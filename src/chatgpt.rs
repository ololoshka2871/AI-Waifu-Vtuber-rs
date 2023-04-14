use std::collections::HashMap;

use async_trait::async_trait;

use chatgpt::prelude::{ChatGPT as ChatGPTClient, Conversation};

use crate::dispatcher::{AIError, AIRequest, AIinterface};

pub struct ChatGPT {
    _client: ChatGPTClient,
    conversations: HashMap<String, Conversation>,
    prompt: String,
}

impl ChatGPT {
    pub fn new<S: Into<String>>(api_key: S, prompt: S) -> Self {
        let client = match ChatGPTClient::new(api_key) {
            Ok(c) => c,
            Err(e) => panic!("Failed to create ChatGPT client: {e:?}"),
        };

        Self {
            _client: client,
            conversations: HashMap::new(),
            prompt: prompt.into(),
        }
    }
}

#[async_trait]
impl AIinterface for ChatGPT {
    async fn process(&mut self, _request: Box<dyn AIRequest>) -> Result<String, AIError> {
        let request = _request.request();
        let conversation = self
            .conversations
            .entry(_request.author())
            .or_insert_with(|| self._client.new_conversation_directed(self.prompt.clone()));

        conversation
            .send_message(request)
            .await
            .map_err(|e| AIError::AnswerError(format!("ChatGPT error: {:?}", e)))?;

        match conversation.history.last() {
            Some(m) => Ok(m.content.clone()),
            None => Err(AIError::UnknownError),
        }
    }
}
