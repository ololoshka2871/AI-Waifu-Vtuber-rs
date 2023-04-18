use std::io::Cursor;

use serenity::model::{
    id::{ChannelId, GuildId},
    prelude::MessageId,
    user::User,
};

#[derive(Debug, Clone)]
pub enum TextRequest {
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
    VoiceDisconnected{
        guild_id: Option<GuildId>,
        channel_id: ChannelId,
        user: User,
    },
}

#[derive(Debug, Clone)]
pub struct TextResponse {
    // Oginal message id
    pub req_msg_id: Option<MessageId>,

    // Channel to answer to
    pub channel_id: ChannelId,

    // Message
    pub text: String,

    // TTS if any
    pub tts: Option<Cursor<bytes::Bytes>>,
}
