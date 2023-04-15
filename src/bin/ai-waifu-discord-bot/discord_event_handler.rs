use std::sync::Arc;

use regex::Regex;
use serenity::{
    async_trait,
    model::{
        channel::Message,
        id::{ChannelId, GuildId},
        prelude::{MessageReference, Ready, UserId},
        user::User,
        voice::VoiceState,
    },
    prelude::{Context, EventHandler, Mutex},
};

use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error, info};

use crate::text_control::{TextRequest as Req, TextResponse as Resp};

pub struct DiscordEventHandler {
    control_request_channel_tx: Sender<Req>,
    text_responce_channel_rx: Arc<Mutex<Receiver<Resp>>>,

    channel_whitelist: Vec<Regex>,
}

impl DiscordEventHandler {
    pub fn new(
        control_request_channel_tx: Sender<Req>,
        text_responce_channel_rx: Receiver<Resp>,
        channel_whitelist: Vec<String>,
    ) -> Self {
        Self {
            control_request_channel_tx,
            text_responce_channel_rx: Arc::new(Mutex::new(text_responce_channel_rx)),
            channel_whitelist: channel_whitelist
                .iter()
                .map(|s| Regex::new(&s).unwrap())
                .collect(),
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
}

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Cache ready");

        let ctx_clone = ctx.clone();
        let response_receiver = self.text_responce_channel_rx.clone();

        tokio::spawn(async move {
            while let Some(resp) = response_receiver.lock().await.recv().await {
                if let Err(e) = resp.channel_id.send_message(&ctx_clone.http, |m| {
                    m.content(resp.text);
                    if let Some(msg_id) = resp.req_msg_id {
                        m.reference_message(MessageReference::from((resp.channel_id, msg_id)));
                    }
                    m.allowed_mentions(|am| {
                        am.replied_user(false);
                        am
                    });
                    m
                }).await {
                    error!("Failed to send message: {:?}", e);
                }
            }
        });
    }

    async fn message(&self, ctx: Context, message: Message) {
        // check if message is from a bot
        if message.author.bot {
            return;
        }

        let channel_name = Self::get_chanel_name_by_id(&ctx, Some(message.channel_id)).await;

        // check if channel is whitelisted
        if !self
            .channel_whitelist
            .iter()
            .any(|r| r.is_match(&channel_name))
        {
            return;
        }

        debug!("{}: {}", message.author.name, message.content);

        self.send_req(Req::TextRequest(
            message.id,
            message.channel_id,
            message.author,
            message.content,
        ))
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

            let channel = Self::get_chanel_name_by_id(&ctx, Some(*channel_id)).await;

            // check if channel is whitelisted
            if self.channel_whitelist.iter().any(|r| r.is_match(&channel)) {
                if let Some(user) = Self::get_user_by_id(&ctx, new.user_id).await {
                    self.send_req(Req::VoiceConnected(user, *channel_id)).await;
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
            if let Some(before_id) = before_id.channel_id {
                if let Some(user) = Self::get_user_by_id(&ctx, new.user_id).await {
                    self.send_req(Req::VoiceDisconnected(user, before_id)).await;
                } else {
                    error!("Ignore voice disconnected request for unknown user");
                }
            }
        }
    }
}
