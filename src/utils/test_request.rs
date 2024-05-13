use crate::dispatcher::AIRequest;

pub struct TestRequest {
    pub request: String,
    pub channel: String,
}

impl AIRequest for TestRequest {
    fn request(&self) -> String {
        self.request.clone()
    }

    fn channel(&self) -> String {
        self.channel.clone()
    }

    fn lang(&self) -> String {
        "auto".to_string()
    }
}