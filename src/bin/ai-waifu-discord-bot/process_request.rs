use std::io::Cursor;

use ai_waifu::{silerio_tts::SilerioTTS, dispatcher::{Dispatcher, AIError}};


use bytes::Bytes;
use serenity::model::prelude::{GuildId, ChannelId, MessageId};
use tokio::sync::mpsc::Sender;

use tracing::{error, info};

use crate::{discord_ai_request::DiscordAIRequest, voice_ch_map::{VoiceChannelMap, State}, control::DiscordResponse};

async fn generate_tts<T: Into<String>>(
    resp: T,
    tts: &SilerioTTS,
    tts_character: Option<&String>,
) -> Option<Cursor<Bytes>> {
    match tts.say(resp, tts_character.clone()).await {
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
    tts: &SilerioTTS,
    tts_character: Option<&String>,
    giuld_ch_user_map: &mut VoiceChannelMap,
    text_responce_channel_tx: &Sender<DiscordResponse>,
    busy_message: String,
    guild_id: GuildId,
    channel_id: ChannelId,
    msg_id: MessageId,
) {
    info!("{}", request);
    match dispatcher.try_process_request(Box::new(request)).await {
        Ok(resp) => {
            let tts_data = generate_tts(&resp, tts, tts_character).await;

            let resp = if giuld_ch_user_map.get_voice_state(guild_id, channel_id) == State::Voice
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
            let resp = if giuld_ch_user_map.get_voice_state(guild_id, channel_id) == State::Voice {
                // Если бот в голосовом канале, то возмутиться вслух, а текст не отправлять
                match tts.say(&busy_message, tts_character.clone()).await {
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
    tts: &SilerioTTS,
    tts_character: Option<&String>,
    giuld_ch_user_map: &mut VoiceChannelMap,
    text_responce_channel_tx: &Sender<DiscordResponse>,
    busy_message: String,
    guild_id: GuildId,
    channel_id: ChannelId,
) {
    info!("{}", request);
    match dispatcher.try_process_request(Box::new(request)).await {
        Ok(resp) => {
            let tts_data = generate_tts(&resp, tts, tts_character).await;

            if giuld_ch_user_map.get_voice_state(guild_id, channel_id) == State::Voice
                && tts_data.is_some()
            {
                // Если бот в голосовом канале, то читать сообщени вслух, а отправлять текст без вложения

                let resp = DiscordResponse::VoiceResponse {
                    req_msg_id: None,
                    guild_id: guild_id,
                    channel_id: channel_id,
                    text: None,
                    tts: tts_data.unwrap(),
                };
                if let Err(err) = text_responce_channel_tx.send(resp).await {
                    error!("Error send discord responce: {:?}", err);
                }
            }
        }
        Err(AIError::Busy) => {
            if giuld_ch_user_map.get_voice_state(guild_id, channel_id) == State::Voice {
                // Если бот в голосовом канале, то возмутиться вслух
                match tts.say(&busy_message, tts_character.clone()).await {
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
            }
        }
        Err(err) => {
            error!("AI Error: {:?}", err);
        }
    }
}
