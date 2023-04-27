mod control;
mod discord_ai_request;
mod discord_event_handler;
mod process_request;
mod voice_ch_map;
mod voice_receiver;

use serenity::{client::Client, framework::StandardFramework, prelude::GatewayIntents, model::prelude::ChannelId};

use songbird::{driver::DecodeMode, Config, SerenityInit};

use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error, info, warn};

use ai_waifu::{config::Config as BotConfig, dispatcher::Dispatcher, silerio_tts::SilerioTTS};
use control::{DiscordRequest, DiscordResponse};
use discord_event_handler::DiscordEventHandler;

use crate::{
    discord_ai_request::DiscordAIRequest,
    voice_ch_map::{State, VoiceChannelMap},
};

use process_request::{process_text_request, process_voice_request};

async fn dispatcher_coroutine<F: Fn() -> String>(
    mut dispatcher: Box<dyn Dispatcher>,
    mut control_request_channel_rx: Receiver<DiscordRequest>,
    text_responce_channel_tx: Sender<DiscordResponse>,
    tts_character: Option<String>,
    tts: SilerioTTS,
    busy_messages_generator: F,
) {
    // грязный хак
    fn convert_user_to_pseudo_channel_id(user: &serenity::model::prelude::User) -> ChannelId {
        ChannelId(user.id.0)
    }

    let mut giuld_ch_user_map = VoiceChannelMap::new();

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

                let request = DiscordAIRequest {
                    request: text,
                    channel_id,
                };

                process_text_request(
                    request,
                    dispatcher.as_mut(),
                    &tts,
                    tts_character.as_ref(),
                    &mut giuld_ch_user_map,
                    &text_responce_channel_tx,
                    busy_messages_generator(),
                    guild_id,
                    channel_id,
                    msg_id,
                )
                .await;
            }
            DiscordRequest::VoiceRequest {
                guild_id,
                channel_id,
                user,
                text,
            } => {
                let request = DiscordAIRequest {
                    request: text,
                    channel_id: convert_user_to_pseudo_channel_id(&user), 
                };

                process_voice_request(
                    request,
                    dispatcher.as_mut(),
                    &tts,
                    tts_character.as_ref(),
                    &mut giuld_ch_user_map,
                    &text_responce_channel_tx,
                    busy_messages_generator(),
                    guild_id,
                    channel_id,
                )
                .await;
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
            DiscordRequest::ResetConversation { guild_id: _, channel_id, user } => {
                let ch = if channel_id.0 != 0 {
                    channel_id
                } else {
                    convert_user_to_pseudo_channel_id(&user)
                };

                if let Err(e) = dispatcher.reset(format!("#{}", ch.0)).await {
                    error!("Failed to reset conversation: {:#?}", e);
                } else {
                    info!("Reset conversation by {}", user.name); 
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

    let tts = SilerioTTS::new(config.silerio_tts_config.tts_service_url);

    let busy_messages = config.busy_messages;

    tokio::spawn(dispatcher_coroutine(
        dispatcher,
        control_request_channel_rx,
        text_responce_channel_tx,
        config.silerio_tts_config.voice_character,
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

    let mut bot = Client::builder(&config.discord_config.discord_token, intents)
        .event_handler(DiscordEventHandler::new(
            control_request_channel_tx,
            text_responce_channel_rx,
            config.discord_config.channel_whitelist,
            config.stt_config.voice2txt_url,
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
