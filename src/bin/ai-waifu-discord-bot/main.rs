mod discord_ai_request;
mod discord_event_handler;
mod text_control;
mod voice_ch_map;
mod voice_receiver;

use serenity::{client::Client, framework::StandardFramework, prelude::GatewayIntents};

use songbird::{driver::DecodeMode, Config, SerenityInit};

use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error, info, warn};

use ai_waifu::{
    chatgpt::ChatGPT,
    config::Config as BotConfig,
    deeplx_translate::DeepLxTranslator,
    dispatcher::Dispatcher,
    silerio_tts::SilerioTTS,
    //google_translator::GoogleTranslator,
};
use discord_event_handler::DiscordEventHandler;
use text_control::{TextRequest, TextResponse};

async fn dispatcher_coroutine(
    mut dispatcher: Dispatcher,
    mut control_request_channel_rx: Receiver<TextRequest>,
    text_responce_channel_tx: Sender<TextResponse>,
    tts_character: Option<String>,
    tts: SilerioTTS,
) {
    use voice_ch_map::State;

    let mut giuld_ch_user_map = voice_ch_map::VoiceChannelMap::new();

    while let Some(req) = control_request_channel_rx.recv().await {
        match req {
            TextRequest::TextRequest {
                guild_id,
                channel_id,
                msg_id,
                user,
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
                    user,
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

                        let text_resp = TextResponse {
                            req_msg_id: Some(msg_id),
                            channel_id: channel_id,
                            text: resp.clone(),
                            tts: if giuld_ch_user_map.get_voice_state(guild_id, channel_id)
                                == State::TextOnly
                            {
                                tts_data
                            } else {
                                None
                            },
                        };
                        match text_responce_channel_tx.send(text_resp).await {
                            Ok(_) => {
                                debug!("Response: {}", resp)
                            }
                            Err(err) => {
                                error!("Error send discord responce: {:?}", err);
                            }
                        }
                    }
                    Err(err) => {
                        error!("AI Error: {:?}", err);
                    }
                }
            }
            TextRequest::VoiceConnected {
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
            TextRequest::VoiceDisconnected {
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
        tokio::sync::mpsc::channel::<TextRequest>(1);

    let (text_responce_channel_tx, text_responce_channel_rx) =
        tokio::sync::mpsc::channel::<TextResponse>(1);

    let ai = ChatGPT::new(config.openai_token, config.initial_prompt);

    let en_ai = DeepLxTranslator::new(
        Box::new(ai),
        Some(config.src_lang),
        Some(config.dest_lang),
        config.deeplx_url,
    );

    let tts = SilerioTTS::new(config.tts_service_url);

    let dispatcher = Dispatcher::new(Box::new(en_ai));

    tokio::spawn(dispatcher_coroutine(
        dispatcher,
        control_request_channel_rx,
        text_responce_channel_tx,
        config.voice_character,
        tts,
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
        ))
        .framework(framework)
        .register_songbird_from_config(songbird_config)
        .await
        .expect("Err creating client");

    let _ = bot
        .start()
        .await
        .map_err(|why| info!("Client ended: {:?}", why));
}
