use ai_waifu::dispatcher::AIRequest;


pub(crate) struct DiscordAIRequest {
    pub request: String,
    pub user: serenity::model::user::User,
}

impl AIRequest for DiscordAIRequest {
    fn request(&self) -> String {
        self.request.clone()
    }

    fn author(&self) -> String {
        self.user.name.clone()
    }
}

impl std::fmt::Display for DiscordAIRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.user.name, self.request)
    }
}