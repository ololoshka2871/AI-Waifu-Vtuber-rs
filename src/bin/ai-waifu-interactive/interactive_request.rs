use ai_waifu::dispatcher::AIRequest;


pub struct InteractiveRequest {
    pub request: String,
}

impl AIRequest for InteractiveRequest {
    fn request(&self) -> String {
        self.request.clone()
    }

    fn channel(&self) -> String {
        "interactive".to_string()
    }
}