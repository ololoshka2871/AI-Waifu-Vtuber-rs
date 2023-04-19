use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::mpsc::{Receiver, Sender},
};

use rodio::{decoder::Decoder, OutputStream, Sink};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device,
};

use clap::Parser;

use ai_waifu::{
    audio_dev::get_audio_device_by_name,
    chatgpt_en_deeplx_builder::ChatGPTEnAIBuilder,
    config::Config,
    dispatcher::{AIRequest, Dispatcher},
    silerio_tts::SilerioTTS,
};

use tracing::{error, info, warn};

struct InteractiveRequest {
    request: String,
}

impl AIRequest for InteractiveRequest {
    fn request(&self) -> String {
        self.request.clone()
    }

    fn channel(&self) -> String {
        "interactive".to_string()
    }
}

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
}

/// print all available devices
fn display_audio_devices(host: &cpal::Host) {
    for device in host.output_devices().unwrap() {
        if let Ok(name) = device.name() {
            if let Ok(oc) = device.supported_output_configs() {
                if oc.count() > 0 {
                    print!("Audio Out: {}", &name);
                }
            }
        }
    }

    for device in host.input_devices().unwrap() {
        if let Ok(name) = device.name() {
            if let Ok(ic) = device.supported_input_configs() {
                if ic.count() > 0 {
                    print!("Audio In: {}", &name);
                }
            }
        }
    }
}

fn spawn_audio_input(ain: Device, audio_req_tx: Sender<String>) -> Result<(), String> {
    let config = ain.default_input_config().map_err(|e| format!("{e}"))?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    let stream = ain
        .build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut up_detected = false;
                // Обработка аудио-данных
                for &sample in data {
                    // Выделение фрагментов, отличных от тишины
                    if sample.abs() > 0.1 && !up_detected {
                        warn!("up_detected");
                        up_detected = true;
                    } else if sample.abs() < 0.05 && up_detected {
                        warn!("down_detected");
                        up_detected = false;
                    }
                }
            },
            |err| {
                // Обработка ошибок
                eprintln!("An error occurred on the input stream: {}", err);
            },
            None,
        )
        .map_err(|e| format!("{e}"))?;

    stream.play().map_err(|e| format!("{e}"))?;

    Ok(())
}

async fn get_voice_request<T>(rx_channel: &mut Receiver<T>) -> String {
    loop {
        if let Some(_s) = rx_channel.recv().await {
            return "".to_string();
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();

    let ht = cpal::default_host();

    if args.list_audio_devices {
        display_audio_devices(&ht);
        return;
    }

    let audio_in = if let Some(in_d) = args.In {
        get_audio_device_by_name(&ht, &in_d, true)
    } else {
        ht.default_input_device()
    };

    let audio_out = if let Some(out_d) = args.Out {
        get_audio_device_by_name(&ht, &out_d, false)
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

    let config = Config::load();

    let mut dispatcher = Dispatcher::new(ChatGPTEnAIBuilder::from(&config));

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin);

    let tts = SilerioTTS::new(config.tts_service_url);

    let mut audio_request_channel = if let Some(ain) = audio_in {
        let (audio_req_tx, audio_req_rx) = tokio::sync::mpsc::channel(1);
        if let Err(e) = spawn_audio_input(ain, audio_req_tx) {
            error!("Failed to init audio input: {}", e);
            None
        } else {
            Some(audio_req_rx)
        }
    } else {
        None
    };

    loop {
        // prompt
        stdout.write("> ".as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();

        let (req, result) = if let Some(audio_request_channel) = &mut audio_request_channel {
            let mut buffer = String::new();
            tokio::select! {
                result = reader.read_line(&mut buffer) => {
                    (buffer.trim().to_owned(), result)
                }
                req = get_voice_request(audio_request_channel) => {
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
        if result.is_err() || unsafe { result.unwrap_unchecked() == 0 } {
            continue;
        }

        if req == "STOP" {
            break;
        }

        let res = dispatcher
            .try_process_request(Box::new(InteractiveRequest { request: req }))
            .await
            .unwrap();

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
