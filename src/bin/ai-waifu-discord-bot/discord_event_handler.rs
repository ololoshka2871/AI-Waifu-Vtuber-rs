use std::{borrow::Cow, io::Cursor, ops::DerefMut, sync::Arc};

use ai_waifu::urukhan_voice_recognize::UrukHanVoice2Txt;
use regex::RegexSet;
use reqwest::Url;
use rodio::Source;
use serenity::{
    async_trait,
    model::{
        channel::Message,
        id::{ChannelId, GuildId},
        prelude::{Guild, MessageId, MessageReference, Ready, UserId},
        user::User,
        voice::VoiceState,
    },
    prelude::{Context, EventHandler, Mutex},
};

use songbird::{error::JoinError, input::Input, Call};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error, info, warn};

use crate::{
    control::{DiscordRequest as Req, DiscordResponse as Resp},
    voice_receiver::{create_voice_control_pair, VoiceEventListenerBuilder, VoiceProcessor},
};

pub struct DiscordEventHandler {
    control_request_channel_tx: Sender<Req>,
    channel_whitelist: RegexSet,
    text_responce_channel_rx: Mutex<Option<Receiver<Resp>>>,

    voice_listener_builder: VoiceEventListenerBuilder,
    voice_processor: Mutex<Option<VoiceProcessor>>,

    voice2txt_url: Url,
}

impl DiscordEventHandler {
    pub fn new(
        control_request_channel_tx: Sender<Req>,
        text_responce_channel_rx: Receiver<Resp>,
        channel_whitelist: Vec<String>,
        voice2txt_url: Url,
    ) -> Self {
        let (voice_listener_builder, voice_processor) = create_voice_control_pair();

        Self {
            control_request_channel_tx,
            channel_whitelist: RegexSet::new(channel_whitelist).unwrap(),
            text_responce_channel_rx: Mutex::new(Some(text_responce_channel_rx)),

            voice_listener_builder,
            voice_processor: Mutex::new(Some(voice_processor)),

            voice2txt_url,
        }
    }

    async fn send_req(&self, req: Req) {
        if self.control_request_channel_tx.send(req).await.is_err() {
            error!("Failed to send request to request handler");
        }
    }

    async fn get_user_name_by_id(ctx: &Context, user_id: UserId) -> String {
        if let Some(user) = Self::get_user_by_id(ctx, user_id).await {
            user.name
        } else {
            user_id.to_string()
        }
    }

    async fn get_user_by_id(ctx: &Context, user_id: UserId) -> Option<User> {
        if let Some(user) = ctx.cache.user(user_id) {
            Some(user)
        } else {
            // user not found in cache - get from discord
            match user_id.to_user(&ctx.http).await {
                Ok(user) => Some(user),
                Err(why) => {
                    error!("Failed to get user {user_id}: {why:?}");
                    None
                }
            }
        }
    }

    async fn get_chanel_name_by_id(ctx: &Context, channel_id: Option<ChannelId>) -> String {
        use serenity::model::prelude::Channel;

        fn extract_name(channel: &Channel) -> String {
            match channel {
                Channel::Category(c) => c.name.clone(),
                Channel::Guild(c) => c.name.clone(),
                Channel::Private(c) => format!("{}'s private channel", c.recipient.name),
                _ => unreachable!(),
            }
        }

        if let Some(channel_id) = channel_id {
            if let Some(channel) = ctx.cache.channel(channel_id) {
                extract_name(&channel)
            } else {
                // channel not found in cache - get from discord
                match channel_id.to_channel(&ctx.http).await {
                    Ok(channel) => extract_name(&channel),
                    Err(why) => {
                        error!("Failed to get channel {channel_id}: {why:?}");
                        channel_id.to_string()
                    }
                }
            }
        } else {
            "None".to_string()
        }
    }

    async fn get_guild_by_id(ctx: &Context, guild_id: GuildId) -> Guild {
        if let Some(guild) = ctx.cache.guild(guild_id) {
            guild
        } else {
            // guild not found in cache - get from discord
            let _ = guild_id
                .to_partial_guild_with_counts(&ctx)
                .await
                .expect("Failed to get guild");
            ctx.cache.guild(guild_id).expect("Error update cache")
        }
    }

    async fn is_channel_allowed(&self, ctx: &Context, channel_id: &ChannelId) -> bool {
        let ch_name = Self::get_chanel_name_by_id(ctx, Some(*channel_id)).await;
        self.channel_whitelist.is_match(ch_name.as_str())
    }

    async fn join_voice_channel<C, G>(
        ctx: &Context,
        guild_id: G,
        channel_id: C,
    ) -> (Arc<Mutex<Call>>, Result<(), JoinError>)
    where
        C: Into<ChannelId>,
        G: Into<GuildId>,
    {
        let manager = songbird::get(ctx).await.unwrap().clone();
        manager.join(guild_id.into(), channel_id.into()).await
    }
}

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        let command_rx = {
            let mut guard = self.text_responce_channel_rx.lock().await;
            let mut rx = None;
            std::mem::swap(guard.deref_mut(), &mut rx);
            rx
        };

        if let Some(mut command_rx) = command_rx {
            info!("Cache ready");

            tokio::spawn(async move {
                async fn send_text_message(
                    ctx: &Context,
                    channel_id: ChannelId,
                    msg_id: Option<MessageId>,
                    text: String,
                    tts: Option<Cursor<bytes::Bytes>>,
                ) {
                    if let Err(e) = channel_id
                        .send_message(&ctx.http, |m| {
                            m.content(text);
                            if let Some(msg_id) = msg_id {
                                m.reference_message(MessageReference::from((channel_id, msg_id)));
                            }
                            m.allowed_mentions(|am| {
                                am.replied_user(false);
                                am
                            });

                            if let Some(tts_data) = tts {
                                m.add_file(serenity::model::prelude::AttachmentType::Bytes {
                                    data: Cow::Owned(tts_data.into_inner().to_vec()),
                                    filename: "TTS.wav".to_string(),
                                });
                            }

                            m
                        })
                        .await
                    {
                        error!("Failed to send message: {e:?}");
                    }
                }

                while let Some(resp) = command_rx.recv().await {
                    match resp {
                        Resp::TextResponse {
                            req_msg_id,
                            channel_id,
                            text,
                            tts,
                        } => send_text_message(&ctx, channel_id, req_msg_id, text, tts).await,
                        Resp::VoiceResponse {
                            req_msg_id,
                            guild_id,
                            channel_id,
                            text,
                            tts,
                        } => {
                            // send text to text channel
                            if let Some(text) = text {
                                send_text_message(&ctx, channel_id, req_msg_id, text, None).await;
                            }

                            // join channel
                            let (handler, join_result) =
                                Self::join_voice_channel(&ctx, guild_id, channel_id).await;
                            if let Err(e) = join_result {
                                error!("Failed to join channel: {:?}", e);
                                continue;
                            }

                            // prepare sound data
                            let decoder = match rodio::Decoder::new_wav(tts) {
                                Ok(decoder) => decoder,
                                Err(e) => {
                                    error!("Failed to decode wav: {:?}", e);
                                    continue;
                                }
                            };
                            let is_stereo = decoder.channels() > 1;

                            // convert audiodata to vec of u8
                            let audiobytes = decoder
                                .into_iter()
                                .map(|x| x.to_le_bytes())
                                .flatten()
                                .collect::<Vec<_>>();

                            // play tts
                            {
                                let mut guard = handler.lock().await;

                                guard.play_only_source(Input::new(
                                    is_stereo,
                                    songbird::input::Reader::from_memory(audiobytes),
                                    songbird::input::Codec::Pcm,
                                    songbird::input::Container::Raw,
                                    None,
                                ));
                            }
                        }
                    }
                }
            });
        } else {
            warn!("Double command_rx ragistarion atempt");
        }

        let voice_processor = {
            let mut guard = self.voice_processor.lock().await;
            let mut p = None;
            std::mem::swap(guard.deref_mut(), &mut p);
            p
        };

        if let Some(mut voice_processor) = voice_processor {
            let voice2txt_url = self.voice2txt_url.clone();

            tokio::spawn(async move {
                let voice2txt = UrukHanVoice2Txt::new(voice2txt_url);

                loop {
                    match voice_processor.try_get_user_voice().await {
                        Ok(Some((user_id, voice_data))) => {
                            debug!(
                                "Voice data: from {} ({} samples)",
                                user_id,
                                voice_data.len()
                            );

                            let voice2txt = voice2txt.clone();
                            tokio::spawn(async move {
                                match ai_waifu::audio_halpers::voice_data_to_wav_buf_gain(voice_data, 2, 48000) {
                                    Ok(wav_data) => match voice2txt.recognize(wav_data).await {
                                        Ok(text) => {
                                            info!("User {} said: {}", user_id, text);
                                        }
                                        Err(e) => {
                                            error!("Failed to convert voice to text: {:?}", e);
                                        }
                                    },
                                    Err(e) => {
                                        error!("Failed to encode voice data to wav: {:?}", e);
                                    }
                                }
                            });
                        }
                        Ok(None) => { /* nothing  */ }
                        Err(_) => {
                            error!("Break voice_processor");
                            break;
                        }
                    }
                }
            });
        } else {
            warn!("Double voice_processor registarion atempt");
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        // check if message is from a bot
        if message.author.bot {
            return;
        }

        // check if channel is whitelisted
        if !self.is_channel_allowed(&ctx, &message.channel_id).await {
            return;
        }

        debug!("{}: {}", message.author.name, message.content);

        self.send_req(Req::TextRequest {
            guild_id: message.guild_id,
            channel_id: message.channel_id,
            msg_id: message.id,
            user: message.author,
            text: message.content,
        })
        .await;
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        if new.user_id == ctx.cache.current_user_id() {
            return; // ignore self
        }

        if let Some(m) = &new.member {
            if m.user.bot {
                return; // ignore bots
            }
        }

        if let Some(old) = &old {
            if old.channel_id.unwrap_or_default() == new.channel_id.unwrap_or_default() {
                return; // same channel - ignore
            }
        }

        // on enter channel
        if let Some(channel_id) = &new.channel_id {
            debug!(
                "{} joined voice channel {}",
                Self::get_user_name_by_id(&ctx, new.user_id).await,
                Self::get_chanel_name_by_id(&ctx, Some(*channel_id)).await
            );

            // check if channel is whitelisted
            if self.is_channel_allowed(&ctx, channel_id).await {
                if let Some(user) = Self::get_user_by_id(&ctx, new.user_id).await {
                    // join channel
                    if let Some(guild_id) = new.guild_id {
                        let (handler, join_result) =
                            Self::join_voice_channel(&ctx, guild_id, channel_id).await;
                        if let Err(e) = join_result {
                            error!("Failed to join channel: {:?}", e);
                        } else {
                            let mut handler = handler.lock().await;

                            handler.add_global_event(
                                songbird::Event::Core(songbird::CoreEvent::SpeakingStateUpdate),
                                self.voice_listener_builder.build_state_update_listener(),
                            );

                            handler.add_global_event(
                                songbird::Event::Core(songbird::CoreEvent::SpeakingUpdate),
                                self.voice_listener_builder.build_speaking_update_listener(),
                            );

                            handler.add_global_event(
                                songbird::Event::Core(songbird::CoreEvent::VoicePacket),
                                self.voice_listener_builder.build_voice_packet_listener(),
                            );
                        }
                    }

                    self.send_req(Req::VoiceConnected {
                        guild_id: new.guild_id,
                        channel_id: *channel_id,
                        user,
                    })
                    .await;
                } else {
                    error!("Ignore voice connected request for unknown user");
                }
            }
        }

        if let Some(before_id) = &old {
            debug!(
                "{} leaved channel {}",
                Self::get_user_name_by_id(&ctx, new.user_id).await,
                Self::get_chanel_name_by_id(&ctx, before_id.channel_id).await
            );
            if let Some(before_ch_id) = before_id.channel_id {
                // check if channel is whitelisted
                if self.is_channel_allowed(&ctx, &before_ch_id).await {
                    if let Some(user) = Self::get_user_by_id(&ctx, new.user_id).await {
                        // leave channel if no one is in it
                        let manager = songbird::get(&ctx).await.unwrap().clone();
                        if let Some(guild_id) = before_id.guild_id {
                            let guild = Self::get_guild_by_id(&ctx, guild_id).await;

                            // На одном сервере бот может быть только в 1 войс канале, поэтому
                            // получим канал где сейчас бот
                            let mut states = guild.voice_states.values();

                            // если в войсе есть кто-то помимо бота
                            let bot_id = ctx.cache.current_user_id();
                            let human_present = states.any(|vs| match vs.channel_id {
                                Some(c_id) => before_ch_id == c_id && vs.user_id != bot_id,
                                None => false,
                            });

                            if !human_present {
                                // если в войсе нет никого помимо бота, то выйдем из канала
                                let guild = manager.get(guild_id).unwrap();
                                let mut lock = guild.lock().await;

                                lock.remove_all_global_events();
                                if let Err(e) = lock.leave().await {
                                    error!("Failed to leave channel: {:?}", e);
                                }
                            }
                        }

                        self.send_req(Req::VoiceDisconnected {
                            guild_id: before_id.guild_id,
                            channel_id: before_ch_id,
                            user,
                        })
                        .await;
                    } else {
                        error!("Ignore voice disconnected request for unknown user");
                    }
                }
            }
        }
    }
}
