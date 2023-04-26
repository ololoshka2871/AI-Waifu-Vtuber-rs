use ai_waifu::dispatcher::{AIRequest};

pub struct InteractiveRequest {
    pub request: String,
    pub lang: String,
}

impl AIRequest for InteractiveRequest {
    fn request(&self) -> String {
        self.request.clone()
    }

    fn channel(&self) -> String {
        "interactive".to_string()
    }

    fn lang(&self) -> String {
        self.lang.clone()
    }
}
