use async_trait::async_trait;

use crate::{
    dispatcher::{AIError, AIRequest, AIinterface},
    llama::{converse, llama_client::LLaMaClient, model_config::ModelConfiguration},
};

pub struct LLaMa {
    conversation: converse::Conversation,
    _client: LLaMaClient,
}

impl LLaMa {
    pub fn new<S: Into<String>>(config: ModelConfiguration, prompt: S) -> Self {
        let client = match LLaMaClient::new_with_config(config) {
            Ok(c) => c,
            Err(e) => panic!("Failed to create ChatGPT client: {e:?}"),
        };

        Self {
            conversation: client.new_conversation_directed(prompt.into()),
            _client: client,
        }
    }
}

#[async_trait]
impl AIinterface for LLaMa {
    async fn process(&mut self, _request: Box<dyn AIRequest>) -> Result<String, AIError> {
        let request = _request.request();

        self.conversation
            .send_message(request)
            .await
            .map_err(|e| AIError::AnswerError(format!("LLaMa error: {:?}", e)))?;

        match self.conversation.history.last() {
            Some(m) => Ok(m.content.clone()),
            None => Err(AIError::UnknownError),
        }
    }

    async fn reset(&mut self) -> Result<(), AIError> {
        if self.conversation.history.len() == 0 {
            Err(AIError::ResetErrorEmpty)
        } else {
            let first_message = self.conversation.history.remove(0);
            self.conversation.history.clear();
            self.conversation.history.push(first_message);
            Ok(())
        }
    }
}
