mod twitch_request;

use std::{collections::HashMap, path::PathBuf};

use cpal::traits::{DeviceTrait, HostTrait};

use clap::Parser;

use ai_waifu::{
    config::Config,
    dispatcher::AIResponseType,
    utils::{audio_dev::get_audio_device_by_name, say::say},
};

#[allow(unused_imports)]
use tracing::{debug, error, info};
use tracing::{log::warn, trace};

use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use twitch_irc::message::ServerMessage;

/// Ai Waifu interactive mode
#[derive(Parser)]
struct Cli {
    /// List audio devices
    #[clap(short, long, default_value_t = false)]
    list_audio_devices: bool,
    /// voice actor name
    #[clap(short, long)]
    voice_actor: Option<String>,

    /// Audio output device name
    #[clap(short = 'O', long)]
    out: Option<String>,

    /// twitch channel to join
    #[clap(short, long, required = true)]
    channel: Option<String>,

    /// Request subtitles file
    #[clap(long)]
    subtitles_req: Option<PathBuf>,

    /// Response subtitles file
    #[clap(long)]
    subtitles_ans: Option<PathBuf>,
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

    let audio_out = if let Some(out_d) = args.out {
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

    let tts = ai_waifu::tts_engine::TTSEngine::with_config(&config.tts_config);

    let twitch_config = twitch_irc::ClientConfig::default();
    let (mut incoming_messages, client) = twitch_irc::TwitchIRCClient::<
        twitch_irc::SecureTCPTransport,
        twitch_irc::login::StaticLoginCredentials,
    >::new(twitch_config);

    let (message_channel_tx, mut message_channel_rx) =
        tokio::sync::mpsc::channel::<twitch_request::TwitchRequest>(2);

    let (tts_channel_tx, mut tts_channel_rx) =
        tokio::sync::mpsc::channel::<HashMap<AIResponseType, String>>(2);

    let channel = args.channel.unwrap();

    //-------------------------------------------------------------------------------

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

                    if let Err(_e) = message_channel_tx.try_send(request) {
                        error!("Procesing queue is full, skipping request");
                    }
                }
                m => debug!("Unhandled message: {:?}", m),
            }
        }
    });

    let subtitles_req = args.subtitles_req.clone();
    let processing_handle = tokio::spawn(async move {
        while let Some(request) = message_channel_rx.recv().await {
            if let Some(subtitles_req) = &subtitles_req {
                debug!("Writing request subtitles...");
                if let Err(e) = std::fs::write(subtitles_req, &request.request) {
                    error!("Failed to write request subtitles: {:?}", e);
                }
            }

            match dispatcher.try_process_request(Box::new(request)).await {
                Ok(res) => tts_channel_tx.send(res).await.unwrap(),
                Err(e) => {
                    error!("Error: {:?}", e);
                    continue;
                }
            };
        }
    });

    let subtitles_req = args.subtitles_req.clone();
    let subtitles_ans = args.subtitles_ans.clone();
    let tts_handle = tokio::spawn(async move {
        while let Some(res) = tts_channel_rx.recv().await {
            let text_to_tts = if let Some(translated_text) = res.get(&AIResponseType::Translated) {
                translated_text
            } else {
                res.get(&AIResponseType::RawAnswer).unwrap()
            };

            // write the line
            if !text_to_tts.is_empty() {
                let sub_text = if config.display_raw_resp {
                    let raw_text = res.get(&AIResponseType::RawAnswer).unwrap();
                    println!("< {} [{}]", text_to_tts, raw_text);
                    raw_text.clone()
                } else {
                    println!("< {}", text_to_tts);
                    text_to_tts.clone()
                };

                if let Some(subtitles_ans) = &subtitles_ans {
                    debug!("Writing answer subtitles...");
                    if let Err(e) = std::fs::write(subtitles_ans, &sub_text) {
                        error!("Failed to write answer subtitles: {:?}", e);
                    }
                }
            }

            // TTS
            match tts.say(text_to_tts).await {
                Ok(sound_data) => {
                    say(&audio_out, sound_data, || {
                        if let Some(subtitles_req) = &subtitles_req {
                            trace!("Clearing request subtitles...");
                            if let Err(e) = std::fs::write(subtitles_req, "") {
                                error!("Failed to clear request subtitles: {:?}", e);
                            }
                        }

                        std::thread::sleep(std::time::Duration::from_millis(750));
                        if let Some(subtitles_ans) = &subtitles_ans {
                            trace!("Clearing answer subtitles...");
                            if let Err(e) = std::fs::write(subtitles_ans, "") {
                                error!("Failed to clear answer subtitles: {:?}", e);
                            }
                        }
                    });
                }
                Err(err) => {
                    error!("TTS error: {:?}", err);
                }
            }
        }
    });

    client.join(channel).unwrap();

    join_handle.await.unwrap();
    processing_handle.await.unwrap();
    tts_handle.await.unwrap();
}

//  return true if text contains repeated words
fn contains_repititions(text: &str) -> bool {
    let words_src = text.split_whitespace().collect::<Vec<&str>>();
    let mut words = words_src.clone();
    words.sort();
    words.dedup();
    if words.len() != words_src.len() {
        if words.len() > 1 && words_src.len() - words.len() > 2 {
            return true;
        }
    }
    false
}
