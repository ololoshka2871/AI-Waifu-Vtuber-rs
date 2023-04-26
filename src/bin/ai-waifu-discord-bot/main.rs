mod control;
mod discord_ai_request;
mod discord_event_handler;
mod voice_ch_map;
mod voice_receiver;

use serenity::{client::Client, framework::StandardFramework, prelude::GatewayIntents};

use songbird::{driver::DecodeMode, Config, SerenityInit};

use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error, info, warn};

use ai_waifu::{
    config::Config as BotConfig,
    dispatcher::{AIError, Dispatcher},
    silerio_tts::SilerioTTS,
};
use control::{DiscordRequest, DiscordResponse};
use discord_event_handler::DiscordEventHandler;

async fn dispatcher_coroutine<F: Fn() -> String>(
    mut dispatcher: Box<dyn Dispatcher>,
    mut control_request_channel_rx: Receiver<DiscordRequest>,
    text_responce_channel_tx: Sender<DiscordResponse>,
    tts_character: Option<String>,
    tts: SilerioTTS,
    busy_messages_generator: F,
) {
    use voice_ch_map::State;

    let mut giuld_ch_user_map = voice_ch_map::VoiceChannelMap::new();

    while let Some(req) = control_request_channel_rx.recv().await {
        match req {
            DiscordRequest::TextRequest {
                guild_id,
                channel_id,
                msg_id,
                user: _,
                text,
            } => {
                let guild_id = if let Some(gi) = guild_id {
                    gi
                } else {
                    error!("Direct message not supported yet!");
                    continue;
                };

                let request = discord_ai_request::DiscordAIRequest {
                    request: text,
                    channel_id,
                };
                info!("{}", request);
                match dispatcher.try_process_request(Box::new(request)).await {
                    Ok(resp) => {
                        let tts_data = match tts.say(&resp, tts_character.clone()).await {
                            Ok(tts) => Some(tts),
                            Err(err) => {
                                error!("TTS error: {:?}", err);
                                None
                            }
                        };

                        let resp = if giuld_ch_user_map.get_voice_state(guild_id, channel_id)
                            == State::Voice
                            && tts_data.is_some()
                        {
                            // Если бот в голосовом канале, то читать сообщени вслух, а отправлять текст без вложения

                            DiscordResponse::VoiceResponse {
                                req_msg_id: Some(msg_id),
                                guild_id: guild_id,
                                channel_id: channel_id,
                                text: Some(resp.clone()),
                                tts: tts_data.unwrap(),
                            }
                        } else {
                            // бот не в голосовом канале, сообщение + вложение

                            DiscordResponse::TextResponse {
                                req_msg_id: Some(msg_id),
                                channel_id: channel_id,
                                text: resp.clone(),
                                tts: tts_data,
                            }
                        };

                        if let Err(err) = text_responce_channel_tx.send(resp).await {
                            error!("Error send discord responce: {:?}", err);
                        }
                    }
                    Err(AIError::Busy) => {
                        let resp = if giuld_ch_user_map.get_voice_state(guild_id, channel_id)
                            == State::Voice
                        {
                            // Если бот в голосовом канале, то возмутиться вслух, а текст не отправлять
                            match tts
                                .say(&busy_messages_generator(), tts_character.clone())
                                .await
                            {
                                Ok(tts) => DiscordResponse::VoiceResponse {
                                    req_msg_id: Some(msg_id),
                                    guild_id: guild_id,
                                    channel_id: channel_id,
                                    text: None,
                                    tts,
                                },
                                Err(err) => {
                                    error!("TTS error: {:?}", err);
                                    DiscordResponse::TextResponse {
                                        req_msg_id: Some(msg_id),
                                        channel_id: channel_id,
                                        text: "TTS error!".to_string(),
                                        tts: None,
                                    }
                                }
                            }
                        } else {
                            // бот не в голосовом канале, сообщение без вложения
                            DiscordResponse::TextResponse {
                                req_msg_id: Some(msg_id),
                                channel_id: channel_id,
                                text: busy_messages_generator(),
                                tts: None,
                            }
                        };

                        if let Err(err) = text_responce_channel_tx.send(resp).await {
                            error!("Error send discord responce: {:?}", err);
                        }
                    }
                    Err(err) => {
                        error!("AI Error: {:?}", err);
                    }
                }
            }
            DiscordRequest::VoiceConnected {
                guild_id,
                channel_id,
                ..
            } => {
                if let Some(gid) = guild_id {
                    giuld_ch_user_map.set_voice_state(gid, channel_id, State::Voice);
                    debug!("{:?}", &giuld_ch_user_map);
                } else {
                    warn!("Not a guild event, ignore!");
                }
            }
            DiscordRequest::VoiceDisconnected {
                guild_id,
                channel_id,
                ..
            } => {
                if let Some(gid) = guild_id {
                    giuld_ch_user_map.set_voice_state(gid, channel_id, State::TextOnly);
                    debug!("{:?}", &giuld_ch_user_map);
                } else {
                    warn!("Not a guild event, ignore!");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = BotConfig::load();

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let (control_request_channel_tx, control_request_channel_rx) =
        tokio::sync::mpsc::channel::<DiscordRequest>(5);

    let (text_responce_channel_tx, text_responce_channel_rx) =
        tokio::sync::mpsc::channel::<DiscordResponse>(1);

    let dispatcher = ai_waifu::create_ai_dispatcher(&config);

    let tts = SilerioTTS::new(config.tts_service_url);

    let busy_messages = config.busy_messages;

    tokio::spawn(dispatcher_coroutine(
        dispatcher,
        control_request_channel_rx,
        text_responce_channel_tx,
        config.voice_character,
        tts,
        move || {
            use rand::Rng;

            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0..busy_messages.len());
            busy_messages[idx].clone()
        },
    ));

    let framework = StandardFramework::new();

    // Here, we need to configure Songbird to decode all incoming voice packets.
    // If you want, you can do this on a per-call basis---here, we need it to
    // read the audio data that other people are sending us!
    let songbird_config = Config::default().decode_mode(DecodeMode::Decode);

    let mut bot = Client::builder(&config.discord_token, intents)
        .event_handler(DiscordEventHandler::new(
            control_request_channel_tx,
            text_responce_channel_rx,
            config.channel_whitelist,
            config.voice2txt_url,
        ))
        .framework(framework)
        .register_songbird_from_config(songbird_config)
        .await
        .expect("Error creating bot");

    let _ = bot
        .start()
        .await
        .map_err(|why| info!("Bot ended: {:?}", why));
}
