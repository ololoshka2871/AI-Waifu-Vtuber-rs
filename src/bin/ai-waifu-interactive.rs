use rodio::{decoder::Decoder, OutputStream, Sink};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

use cpal::traits::{DeviceTrait, HostTrait};

use clap::Parser;

use ai_waifu::{
    audio_dev::get_audio_device_by_name,
    //google_translator::GoogleTranslator,
    chatgpt::ChatGPT,
    config::Config,
    deeplx_translate::DeepLxTranslator,
    dispatcher::{AIRequest, Dispatcher},
    silerio_tts::SilerioTTS,
};

use tracing::{error, info};

struct InteractiveRequest {
    request: String,
}

impl AIRequest for InteractiveRequest {
    fn request(&self) -> String {
        self.request.clone()
    }

    fn author(&self) -> String {
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

    let ai = ChatGPT::new(config.openai_token, config.initial_prompt);

    let en_ai = DeepLxTranslator::new(
        Box::new(ai),
        Some(config.src_lang),
        Some(config.dest_lang),
        config.deeplx_url,
    );

    let mut dispatcher = Dispatcher::new(Box::new(en_ai));

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut buffer = String::new();

    let tts = SilerioTTS::new(config.tts_service_url);

    loop {
        // prompt
        stdout.write("> ".as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();

        // read a line
        let size = reader.read_line(&mut buffer).await.unwrap();

        // check if the line is empty
        if size == 0 {
            continue;
        }

        let res = dispatcher
            .try_process_request(Box::new(InteractiveRequest {
                request: buffer.trim().to_string(),
            }))
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

        // clear buffer
        buffer.clear();
    }
}
