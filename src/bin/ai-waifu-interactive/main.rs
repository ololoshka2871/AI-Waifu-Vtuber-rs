use std::io::Write;

mod interactive_request;

use interactive_request::InteractiveRequest;

use rodio::{decoder::Decoder, OutputStream, Sink};

use cpal::traits::{DeviceTrait, HostTrait};

use clap::Parser;

use ai_waifu::{
    config::Config, silerio_tts::SilerioTTS,
    utils::{audio_input::spawn_audio_input, audio_dev::get_audio_device_by_name},
};

#[allow(unused_imports)]
use tracing::{debug, error, info};

use ai_waifu::utils::audio_input::get_voice_request;

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

fn process_rusty_result(
    rl_res: Result<String, rustyline_async::ReadlineError>,
) -> Result<String, &'static str> {
    use rustyline_async::ReadlineError;
    match rl_res {
        Ok(line) => Ok(line),
        Err(rustyline_async::ReadlineError::Eof) => Err("Exiting..."),
        Err(ReadlineError::Interrupted) => Err("^C"),
        Err(ReadlineError::Closed) => Err("Closed"),
        Err(_) => Err("Unknown input error"),
    }
}

#[tokio::main]
async fn main() {
    use futures_util::future::FutureExt;

    tracing_subscriber::fmt::init();

    let config = Config::load();

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

    let mut dispatcher = ai_waifu::create_ai_dispatcher(&config);

    let tts = SilerioTTS::new(config.silerio_tts_config.tts_service_url);

    let mut audio_request_ctrl = if let Some(ain) = audio_in {
        let (audio_req_tx, audio_req_rx) = tokio::sync::mpsc::channel(1);
        match spawn_audio_input(
            ain,
            audio_req_tx,
            args.noise_gate,
            args.release_time,
            config.stt_config.voice2txt_url,
            config.stt_config.minimal_audio_fragment_length,
            config.stt_config.maximal_audio_fragment_length,
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

    let (mut rl, mut stdout) =
        rustyline_async::Readline::new("> ".to_owned()).expect("Failed to init interactive input!");

    loop {
        let request = if let Some(audio_request_channel) = &mut audio_request_ctrl {
            tokio::select! {
                result = rl.readline().fuse() => {
                    let req = process_rusty_result(result).unwrap_or_else(|e| panic!("{}", e));
                    InteractiveRequest{request: req, lang: "auto".to_string()}
                }
                req = get_voice_request(&mut audio_request_channel.0) => {
                    write!(stdout, "{} ({})\n", req.0, req.1).unwrap();
                    InteractiveRequest{request: req.0, lang: req.1}
                }
            }
        } else {
            let req = process_rusty_result(rl.readline().await).unwrap_or_else(|e| panic!("{}", e));
            InteractiveRequest {
                request: req,
                lang: "auto".to_string(),
            }
        };

        let res = match dispatcher.try_process_request(Box::new(request)).await {
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
        write!(stdout, "< {}\n", res).unwrap();
    }
}
