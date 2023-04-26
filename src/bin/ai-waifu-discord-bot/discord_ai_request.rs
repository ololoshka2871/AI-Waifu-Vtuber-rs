use ai_waifu::dispatcher::AIRequest;
use serenity::model::prelude::ChannelId;


pub(crate) struct DiscordAIRequest {
    pub request: String,
    pub channel_id: ChannelId,
}

impl AIRequest for DiscordAIRequest {
    fn request(&self) -> String {
        self.request.clone()
    }

    fn channel(&self) -> String {
        format!("#{}", self.channel_id)
    }

    fn lang(&self) -> String {
        "auto".to_string()
    }
}

impl std::fmt::Display for DiscordAIRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}: {}", self.channel_id, self.request)
    }
}