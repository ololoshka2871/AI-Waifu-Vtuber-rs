use ai_waifu::dispatcher::AIRequest;

pub struct TwitchRequest {
    pub request: String,
    pub username: String,
}

impl AIRequest for TwitchRequest {
    fn request(&self) -> String {
        self.request.clone()
    }

    fn channel(&self) -> String {
        format!("{}", self.username)
    }

    fn lang(&self) -> String {
        "auto".to_string()
    }
}

impl std::fmt::Display for TwitchRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.username, self.request)
    }
}