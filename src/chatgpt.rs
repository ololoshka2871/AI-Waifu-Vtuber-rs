use std::{collections::HashMap, io::Write, path::PathBuf};

use async_trait::async_trait;

use chatgpt::{
    prelude::{ChatGPT as ChatGPTClient, Conversation, ModelConfiguration},
    types::ChatMessage,
};
use futures_util::StreamExt;
use maplit::hashmap;

use tracing::error;

use crate::dispatcher::{AIError, AIRequest, AIResponseType, AIinterface, AIinterfaceStreamed};

pub struct ChatGPT {
    _client: ChatGPTClient,
    conversation: Conversation,
}

impl ChatGPT {
    pub fn new<S: Into<String>>(api_key: S, config: ModelConfiguration, prompt: S) -> Self {
        let client = match ChatGPTClient::new_with_config(api_key, config) {
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
    async fn process(
        &mut self,
        _request: Box<dyn AIRequest>,
    ) -> Result<HashMap<AIResponseType, String>, AIError> {
        let request = _request.request();

        self.conversation
            .send_message(request)
            .await
            .map_err(|e| AIError::AnswerError(format!("ChatGPT error: {:?}", e)))?;

        match self.conversation.history.last() {
            Some(m) => {
                let res = hashmap! {
                    AIResponseType::RawAnswer => m.content.clone(),
                };
                Ok(res)
            }
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

    async fn save_context(&mut self, file: PathBuf) -> Result<(), AIError> {
        self.conversation
            .save_history_json(file)
            .await
            .map_err(|_| AIError::ContextError)
    }

    fn load_context(&mut self, file: PathBuf) -> Result<(), AIError> {
        if !file.exists() {
            error!("File \"{file:?}\" not exists");
            return Err(AIError::ContextError);
        }

        self.conversation.history = serde_json::from_reader::<_, Vec<ChatMessage>>(
            std::fs::File::open(file).map_err(|_| AIError::ContextError)?,
        )
        .map_err(|_| AIError::ContextError)?;
        Ok(())
    }
}

#[async_trait]
impl AIinterfaceStreamed for ChatGPT {
    async fn process_streamed(&mut self, request: Box<dyn AIRequest>) -> Result<(), AIError> {
        let request = request.request();

        let mut r = self
            .conversation
            .send_message_streaming(request)
            .await
            .map_err(|e| AIError::AnswerError(format!("ChatGPT error: {:?}", e)))?;

        /*
        let chanks = r
            .map(|chank| {
                match &chank {
                    chatgpt::types::ResponseChunk::Content { delta, .. } => {
                        println!("Token: {:?}", delta);
                    }
                    _ => {}
                };
                chank
            })
            .collect::<Vec<_>>().await;
        */
        
        let mut chanks = vec![];
        while let Some(chunk) = r.next().await{
            match chunk {
                chatgpt::types::ResponseChunk::Content {
                    delta,
                    response_index,
                } => {
                    // Printing part of response without the newline
                    print!("{delta}");
                    // Manually flushing the standard output, as `print` macro does not do that
                    std::io::stdout().lock().flush().unwrap();
                    chanks.push(chatgpt::types::ResponseChunk::Content {
                        delta,
                        response_index,
                    });
                }
                // We don't really care about other types, other than parsing them into a ChatMessage later
                other => chanks.push(other),
            }
        }

        let messages = ChatMessage::from_response_chunks(chanks);
        for m in messages {
            println!("{:?}: {}", m.role, m.content);
            self.conversation.history.push(m);
        }
        Ok(())
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

    async fn save_context(&mut self, file: PathBuf) -> Result<(), AIError> {
        self.conversation
            .save_history_json(file)
            .await
            .map_err(|_| AIError::ContextError)
    }

    fn load_context(&mut self, file: PathBuf) -> Result<(), AIError> {
        if !file.exists() {
            error!("File \"{file:?}\" not exists");
            return Err(AIError::ContextError);
        }

        self.conversation.history = serde_json::from_reader::<_, Vec<ChatMessage>>(
            std::fs::File::open(file).map_err(|_| AIError::ContextError)?,
        )
        .map_err(|_| AIError::ContextError)?;
        Ok(())
    }
}
