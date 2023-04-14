use serenity::model::{id::ChannelId, user::User};


#[derive(Debug, Clone)]
pub enum Request {
    /// Simple text message request
    TextRequest(User, String),

    /// User joined a voice channel
    VoiceConnected(User, ChannelId),

    /// User leaved a voice channel
    VoiceDisconnected(User, ChannelId),
}