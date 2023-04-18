use crate::dispatcher::AIRequest;

pub struct TranslatedAIRequest {
    original: Box<dyn AIRequest>,
    en_text: String,
}

impl TranslatedAIRequest {
    pub fn new(original: Box<dyn AIRequest>, en_text: String) -> Self {
        Self { original, en_text }
    }
}

impl AIRequest for TranslatedAIRequest {
    fn request(&self) -> String {
        self.en_text.clone()
    }

    fn channel(&self) -> String {
        self.original.channel()
    }
}
