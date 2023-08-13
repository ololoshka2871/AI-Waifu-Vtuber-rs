use std::{collections::HashMap, io::Cursor};

use ai_waifu::{
    dispatcher::{AIError, AIResponseType, Dispatcher},
    tts_engine::TTSEngine,
};

use bytes::Bytes;
use serenity::model::prelude::{ChannelId, GuildId, MessageId};
use tokio::sync::mpsc::Sender;

use tracing::{error, info};

use crate::{
    control::DiscordResponse,
    discord_ai_request::DiscordAIRequest,
    voice_ch_map::{State, VoiceChannelMap},
};

fn get_texts<'a>(
    resp: &'a HashMap<AIResponseType, String>,
    display_raw_resp: bool,
) -> (&'a String, String) {
    let text_to_tts = if let Some(translated_text) = resp.get(&AIResponseType::Translated) {
        translated_text
    } else {
        resp.get(&AIResponseType::RawAnswer).unwrap()
    };

    let text_to_send = if display_raw_resp {
        format!(
            "{} [{}]",
            text_to_tts,
            resp.get(&AIResponseType::RawAnswer).unwrap()
        )
    } else {
        text_to_tts.clone()
    };

    (text_to_tts, text_to_send)
}

async fn generate_tts<T: Into<String>>(resp: T, tts: &TTSEngine) -> Option<Cursor<Bytes>> {
    match tts.say(resp).await {
        Ok(tts) => Some(tts),
        Err(err) => {
            error!("TTS error: {:?}", err);
            None
        }
    }
}

pub async fn process_text_request(
    request: DiscordAIRequest,
    dispatcher: &mut dyn Dispatcher,
    tts: &TTSEngine,
    giuld_ch_user_map: &mut VoiceChannelMap,
    text_responce_channel_tx: &Sender<DiscordResponse>,
    busy_message: String,
    guild_id: GuildId,
    channel_id: ChannelId,
    msg_id: MessageId,
    display_raw_resp: bool,
) {
    info!("{}", request);
    match dispatcher.try_process_request(Box::new(request)).await {
        Ok(resp) => {
            let (text_to_tts, text_to_send) = get_texts(&resp, display_raw_resp);

            let tts_data = generate_tts(text_to_tts, tts).await;

            let resp = if giuld_ch_user_map.get_voice_state(guild_id, channel_id) == State::Voice
                && tts_data.is_some()
            {
                // Если бот в голосовом канале, то читать сообщени вслух, а отправлять текст без вложения
                DiscordResponse::VoiceResponse {
                    req_msg_id: Some(msg_id),
                    guild_id: guild_id,
                    channel_id: channel_id,
                    text: Some(text_to_send.clone()),
                    tts: tts_data.unwrap(),
                }
            } else {
                // бот не в голосовом канале, сообщение + вложение
                DiscordResponse::TextResponse {
                    req_msg_id: Some(msg_id),
                    channel_id: channel_id,
                    text: text_to_send.clone(),
                    tts: tts_data,
                }
            };

            if let Err(err) = text_responce_channel_tx.send(resp).await {
                error!("Error send discord responce: {:?}", err);
            }
        }
        Err(AIError::Busy) => {
            let resp = if giuld_ch_user_map.get_voice_state(guild_id, channel_id) == State::Voice {
                // Если бот в голосовом канале, то возмутиться вслух, а текст не отправлять
                match tts.say(&busy_message).await {
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
                    text: busy_message,
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

pub async fn process_voice_request(
    request: DiscordAIRequest,
    dispatcher: &mut dyn Dispatcher,
    tts: &TTSEngine,
    giuld_ch_user_map: &mut VoiceChannelMap,
    text_responce_channel_tx: &Sender<DiscordResponse>,
    busy_message: String,
    guild_id: GuildId,
    channel_id: ChannelId,
    display_raw_resp: bool,
) {
    info!("{}", request);
    match dispatcher.try_process_request(Box::new(request)).await {
        Ok(resp) => {
            let (text_to_tts, text_to_send) = get_texts(&resp, display_raw_resp);

            let tts_data = generate_tts(text_to_tts, tts).await;

            if giuld_ch_user_map.get_voice_state(guild_id, channel_id) == State::Voice
                && tts_data.is_some()
            {
                // Если бот в голосовом канале, то читать сообщени вслух, а отправлять текст без вложения
                let resp = DiscordResponse::VoiceResponse {
                    req_msg_id: None,
                    guild_id: guild_id,
                    channel_id: channel_id,
                    text: if display_raw_resp {
                        Some(text_to_send.clone())
                    } else {
                        None
                    },
                    tts: tts_data.unwrap(),
                };
                if let Err(err) = text_responce_channel_tx.send(resp).await {
                    error!("Error send discord responce: {:?}", err);
                }
            } else {
                // send text response
                let resp = DiscordResponse::TextResponse {
                    req_msg_id: None,
                    channel_id: channel_id,
                    text: text_to_send.clone(),
                    tts: None,
                };
                if let Err(err) = text_responce_channel_tx.send(resp).await {
                    error!("Error send discord responce: {:?}", err);
                }
            }
        }
        Err(AIError::Busy) => {
            if giuld_ch_user_map.get_voice_state(guild_id, channel_id) == State::Voice {
                // Если бот в голосовом канале, то возмутиться вслух
                match tts.say(&busy_message).await {
                    Ok(tts) => {
                        let resp = DiscordResponse::VoiceResponse {
                            req_msg_id: None,
                            guild_id: guild_id,
                            channel_id: channel_id,
                            text: None,
                            tts,
                        };
                        if let Err(err) = text_responce_channel_tx.send(resp).await {
                            error!("Error send discord responce: {:?}", err);
                        }
                    }
                    Err(err) => {
                        error!("TTS error: {:?}", err);
                    }
                }
            } else {
                // бот не в голосовом канале, сообщение без вложения
                let resp = DiscordResponse::TextResponse {
                    req_msg_id: None,
                    channel_id: channel_id,
                    text: busy_message,
                    tts: None,
                };
                if let Err(err) = text_responce_channel_tx.send(resp).await {
                    error!("Error send discord responce: {:?}", err);
                }
            }
        }
        Err(err) => {
            error!("AI Error: {:?}", err);
        }
    }
}
