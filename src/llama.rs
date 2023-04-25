mod completion_req;
mod converse;
mod llama_client;
mod model_config;

use async_trait::async_trait;

use crate::dispatcher::{AIError, AIRequest, AIinterface};

use self::llama_client::LLaMaClient;

pub struct LLaMa {
    conversation: converse::Conversation,
    _client: LLaMaClient,
}

impl LLaMa {
    pub fn new<S: Into<String>>(url: reqwest::Url, prompt: S) -> Self {
        let client = match LLaMaClient::new_with_config(
            model_config::ModelConfigurationBuilder::default()
                .api_url(url)
                .build().unwrap(),
        ) {
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
}
