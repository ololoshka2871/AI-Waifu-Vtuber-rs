mod audio_input;
mod interactive_request;

use audio_input::spawn_audio_input;
use interactive_request::InteractiveRequest;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

use rodio::{decoder::Decoder, OutputStream, Sink};

use cpal::traits::{DeviceTrait, HostTrait};

use clap::Parser;

use ai_waifu::{
    audio_dev::get_audio_device_by_name, chatgpt_en_deeplx_builder::ChatGPTEnAIBuilder,
    config::Config, dispatcher::Dispatcher, silerio_tts::SilerioTTS,
};

use tracing::{debug, error, info};

use crate::audio_input::get_voice_request;

/// Ai Waifu interactive mode
#[derive(Parser)]
#[allow(non_snake_case)]
struct Cli {
    /// List audio devices
    #[clap(short, long, default_value_t = false)]
    list_audio_devices: bool,
    /// voice actor name
    #[clap(short, long)]
    voice_actor: Option<String>,

    /// Audio input device name
    #[clap(short, long)]
    In: Option<String>,
    /// Audio output device name
    #[clap(short, long)]
    Out: Option<String>,

    /// Audio noise_gate, 0.0 - 1.0
    #[clap(short, long, default_value_t = 0.1)]
    noise_gate: f32,
    /// Audio release_time, s
    #[clap(short, long, default_value_t = 1.0)]
    release_time: f32,
}

/// print all available devices
fn display_audio_devices(host: &cpal::Host) {
    for device in host.output_devices().unwrap() {
        if let Ok(name) = device.name() {
            if let Ok(oc) = device.supported_output_configs() {
                if oc.count() > 0 {
                    println!("Audio Out: {}", &name);
                }
            }
        }
    }

    for device in host.input_devices().unwrap() {
        if let Ok(name) = device.name() {
            if let Ok(ic) = device.supported_input_configs() {
                if ic.count() > 0 {
                    println!("Audio In: {}", &name);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::load();

    //ai_waifu::init_python(&config.python_path).unwrap();

    let tts_local = ai_waifu::silerio_tts_local::SilerioTTSLocal::new(&config.silerio_tts_model)
        .expect(format!("Failed to load SilerioTTS model {}. Please check if The file exists and it's a TorchScript model file", 
            config.silerio_tts_model.to_str().unwrap()).as_str());
    let d = tts_local.say("Привет мир!", None::<String>).await.unwrap();
    debug!("data: {:?} bytes", d.into_inner().len());

    let args = Cli::parse();

    let ht = cpal::default_host();

    if args.list_audio_devices {
        display_audio_devices(&ht);
        return;
    }

    let audio_in = if let Some(in_d) = args.In {
        if in_d == "none" {
            None
        } else {
            get_audio_device_by_name(&ht, &in_d, true)
        }
    } else {
        ht.default_input_device()
    };

    let audio_out = if let Some(out_d) = args.Out {
        if out_d == "none" {
            None
        } else {
            get_audio_device_by_name(&ht, &out_d, false)
        }
    } else {
        ht.default_output_device()
    };

    if let Some(ai) = &audio_in {
        info!(
            "Using audio input device: {}",
            ai.name().unwrap_or("unknown".to_string())
        );
    } else {
        error!("No audio input device found, only text input will be available!");
    }

    if let Some(ao) = &audio_out {
        info!(
            "Using audio output device: {}",
            ao.name().unwrap_or("unknown".to_string())
        );
    } else {
        error!("No audio output device found, only text output will be available!");
    }

    let mut dispatcher = Dispatcher::new(ChatGPTEnAIBuilder::from(&config));

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin);

    let tts = SilerioTTS::new(config.tts_service_url);

    let mut audio_request_ctrl = if let Some(ain) = audio_in {
        let (audio_req_tx, audio_req_rx) = tokio::sync::mpsc::channel(1);
        match spawn_audio_input(
            ain,
            audio_req_tx,
            args.noise_gate,
            args.release_time,
            config.voice2txt_url,
            tokio::runtime::Handle::current(),
        ) {
            Ok(stream) => Some((audio_req_rx, stream)),
            Err(e) => {
                error!("Failed to init audio input: {}", e);
                None
            }
        }
    } else {
        None
    };

    info!("Type 'STOP' or Ctrl+D (^D) to exit");
    loop {
        // prompt
        stdout.write("> ".as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();

        let (req, result) = if let Some(audio_request_channel) = &mut audio_request_ctrl {
            let mut buffer = String::new();
            tokio::select! {
                result = reader.read_line(&mut buffer) => {
                    (buffer.trim().to_owned(), result)
                }
                req = get_voice_request(&mut audio_request_channel.0) => {
                    let len = req.len();
                    stdout.write(req.as_bytes()).await.unwrap();
                    stdout.write(b"\n").await.unwrap();
                    (req, Ok(len))
                }
            }
        } else {
            let mut buffer = String::new();
            let res = reader.read_line(&mut buffer).await;
            (buffer.trim().to_owned(), res)
        };

        // check if the line is empty
        if let Err(_) = result {
            continue; // try again
        }

        if req == "STOP" || req.starts_with("\x04") {
            break;
        }

        let res = match dispatcher
            .try_process_request(Box::new(InteractiveRequest { request: req }))
            .await
        {
            Ok(res) => res,
            Err(e) => {
                error!("Error: {:?}", e);
                continue;
            }
        };

        // TTS
        match tts.say(&res, args.voice_actor.clone()).await {
            Ok(sound_data) => {
                if let Some(ao) = &audio_out {
                    if let Ok((_stream, stream_handle)) = OutputStream::try_from_device(ao) {
                        // _stream mast exists while stream_handle is used
                        match Sink::try_new(&stream_handle) {
                            Ok(sink) => match Decoder::new_wav(sound_data) {
                                Ok(decoder) => {
                                    sink.append(decoder);
                                    sink.sleep_until_end();
                                }
                                Err(e) => {
                                    error!("Decode wav error: {:?}", e);
                                }
                            },
                            Err(e) => {
                                error!("Sink error: {:?}", e);
                            }
                        }
                    } else {
                        error!("Audio output error");
                    }
                }
            }
            Err(err) => {
                error!("TTS error: {:?}", err);
            }
        }

        // write the line
        stdout
            .write(format!("< {}\n", res).as_bytes())
            .await
            .unwrap();
    }
}
