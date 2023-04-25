#![allow(unused)]

use chatgpt::types::{ChatMessage, Role, CompletionResponse};

use super::llama_client::LLaMaClient;


/// Stores a single conversation session, and automatically saves message history
pub struct Conversation {
    client: LLaMaClient,
    /// All the messages sent and received, starting with the beginning system message
    pub history: Vec<ChatMessage>,
}

impl Conversation {
    /// Constructs a new conversation from an API client and the introductory message
    pub fn new(client: LLaMaClient, first_message: String) -> Self {
        Self {
            client,
            history: vec![ChatMessage {
                role: Role::System,
                content: first_message,
            }],
        }
    }

    /// Constructs a new conversation from a pre-initialized chat history
    pub fn new_with_history(client: LLaMaClient, history: Vec<ChatMessage>) -> Self {
        Self { client, history }
    }

    /// Rollbacks the history by 1 message, removing the last sent and received message.
    pub fn rollback(&mut self) -> Option<ChatMessage> {
        let last = self.history.pop();
        self.history.pop();
        last
    }

    /// Sends the message to the ChatGPT API and returns the completion response.
    ///
    /// Execution speed depends on API response times.
    pub async fn send_message<S: Into<String>>(
        &mut self,
        message: S,
    ) -> chatgpt::Result<CompletionResponse> {
        self.history.push(ChatMessage {
            role: Role::User,
            content: message.into(),
        });
        let resp = self.client.send_history(&self.history).await?;
        self.history.push(resp.message_choices[0].message.clone());
        Ok(resp)
    }
}
