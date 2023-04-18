use std::io::Cursor;

use serenity::model::{
    id::{ChannelId, GuildId},
    prelude::MessageId,
    user::User,
};

#[derive(Debug, Clone)]
pub enum DiscordRequest {
    /// Simple text message request
    TextRequest {
        guild_id: Option<GuildId>,
        channel_id: ChannelId,
        msg_id: MessageId,
        user: User,
        text: String,
    },

    /// User joined a voice channel
    VoiceConnected {
        guild_id: Option<GuildId>,
        channel_id: ChannelId,
        user: User,
    },

    /// User leaved a voice channel
    VoiceDisconnected {
        guild_id: Option<GuildId>,
        channel_id: ChannelId,
        user: User,
    },
}

#[derive(Debug, Clone)]
pub enum DiscordResponse {
    /// Simple text message response
    TextResponse {
        req_msg_id: Option<MessageId>,     // Oginal message id
        channel_id: ChannelId,             // Channel to answer to
        text: String,                      // Message
        tts: Option<Cursor<bytes::Bytes>>, // TTS if any
    },
    VoiceResponse {
        req_msg_id: Option<MessageId>, // Oginal message id
        guild_id: GuildId,             // Guild to answer to
        channel_id: ChannelId,         // Text Channel to answer to
        text: Option<String>,          // Text message if any
        tts: Cursor<bytes::Bytes>,     // TTS
    },
}
