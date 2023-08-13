mod twitch_request;

use rodio::{decoder::Decoder, OutputStream, Sink};

use cpal::traits::{DeviceTrait, HostTrait};

use clap::Parser;

use ai_waifu::{
    config::Config, silerio_tts::SilerioTTS, utils::audio_dev::get_audio_device_by_name,
};

use tracing::log::warn;
#[allow(unused_imports)]
use tracing::{debug, error, info};

use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use twitch_irc::message::ServerMessage;

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

    /// Audio output device name
    #[clap(short, long)]
    Out: Option<String>,

    /// twitch channel to join
    #[clap(short, long, required = true)]
    channel: Option<String>,
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
    let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    let config = Config::load();

    let args = Cli::parse();

    let ht = cpal::default_host();

    if args.list_audio_devices {
        display_audio_devices(&ht);
        return;
    }

    let audio_out = if let Some(out_d) = args.Out {
        if out_d == "none" {
            None
        } else {
            get_audio_device_by_name(&ht, &out_d, false)
        }
    } else {
        ht.default_output_device()
    };

    if let Some(ao) = &audio_out {
        info!(
            "Using audio output device: {}",
            ao.name().unwrap_or("unknown".to_string())
        );
    } else {
        error!("No audio output device found, only text output will be available!");
    }

    let mut dispatcher = ai_waifu::create_ai_dispatcher(&config);

    let tts = SilerioTTS::new(config.silerio_tts_config.tts_service_url, args.voice_actor);

    let config = twitch_irc::ClientConfig::default();
    let (mut incoming_messages, client) = twitch_irc::TwitchIRCClient::<
        twitch_irc::SecureTCPTransport,
        twitch_irc::login::StaticLoginCredentials,
    >::new(config);

    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Pong(_)
                | ServerMessage::Join(_)
                | ServerMessage::Generic(_)
                | ServerMessage::RoomState(_) => {}
                ServerMessage::Privmsg(m) => {
                    let text = m.message_text.trim();
                    if text.contains("@")
                        | text.contains("http")
                        | text.contains("!")
                        | (text.len() < 5)
                        | !text.contains(" ")
                        | contains_repititions(&m.message_text)
                    {
                        warn!("Skipping: {}", m.message_text);
                        continue;
                    } else {
                        info!("Processing: {}: {}", m.sender.name, m.message_text);
                    }

                    let request = twitch_request::TwitchRequest {
                        request: text.to_owned(),
                        username: m.sender.name,
                    };

                    let res = match dispatcher.try_process_request(Box::new(request)).await {
                        Ok(res) => res,
                        Err(e) => {
                            error!("Error: {:?}", e);
                            continue;
                        }
                    };

                    match tts.say(&res).await {
                        Ok(sound_data) => {
                            if let Some(ao) = &audio_out {
                                if let Ok((_stream, stream_handle)) =
                                    OutputStream::try_from_device(ao)
                                {
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

                    warn!("AI: {}\n", res);
                }
                m => debug!("Unhandled message: {:?}", m),
            }
        }
    });

    client.join(args.channel.unwrap()).unwrap();

    join_handle.await.unwrap();
}

//  return true if text contains repeated words
fn contains_repititions(text: &str) -> bool {
    let mut words = text.split_whitespace().collect::<Vec<&str>>();
    words.sort();
    words.dedup();
    words.len() != text.split_whitespace().count()
}