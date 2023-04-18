use async_trait::async_trait;

use chatgpt::prelude::{ChatGPT as ChatGPTClient, Conversation};

use crate::dispatcher::{AIError, AIRequest, AIinterface};

pub struct ChatGPT {
    _client: ChatGPTClient,
    conversation: Conversation,
}

impl ChatGPT {
    pub fn new<S: Into<String>>(api_key: S, prompt: S) -> Self {
        let client = match ChatGPTClient::new(api_key) {
            Ok(c) => c,
            Err(e) => panic!("Failed to create ChatGPT client: {e:?}"),
        };

        Self {
            conversation: client.new_conversation_directed(prompt),
            _client: client,
        }
    }
}

#[async_trait]
impl AIinterface for ChatGPT {
    async fn process(&mut self, _request: Box<dyn AIRequest>) -> Result<String, AIError> {
        let request = _request.request();

        self.conversation
            .send_message(request)
            .await
            .map_err(|e| AIError::AnswerError(format!("ChatGPT error: {:?}", e)))?;

        match self.conversation.history.last() {
            Some(m) => Ok(m.content.clone()),
            None => Err(AIError::UnknownError),
        }
    }
}
