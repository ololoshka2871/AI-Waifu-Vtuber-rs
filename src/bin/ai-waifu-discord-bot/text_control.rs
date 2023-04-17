use std::io::Cursor;

use serenity::model::{id::ChannelId, prelude::MessageId, user::User};

#[derive(Debug, Clone)]
pub enum TextRequest {
    /// Simple text message request
    TextRequest(MessageId, ChannelId, User, String),

    /// User joined a voice channel
    VoiceConnected(User, ChannelId),

    /// User leaved a voice channel
    VoiceDisconnected(User, ChannelId),
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
