#![allow(unused)]

use chatgpt::types::{ChatMessage, CompletionResponse, Role, ServerResponse};
use reqwest::header::HeaderMap;

use super::{
    completion_req::CompletionRequest, converse::Conversation, model_config::ModelConfiguration,
};

/// The client that operates the LLaMa API
#[derive(Debug, Clone)]
pub struct LLaMaClient {
    client: reqwest::Client,
    /// The configuration for this LLaMa client
    pub config: ModelConfiguration,
}

impl LLaMaClient {
    /// Constructs a new LLaMa API client with provided API key and default configuration
    pub fn new() -> chatgpt::Result<Self> {
        Self::new_with_config(ModelConfiguration::default())
    }

    /// Constructs a new LLaMa API client with provided API Key and Configuration
    pub fn new_with_config(config: ModelConfiguration) -> chatgpt::Result<Self> {
        let mut headers = HeaderMap::new();
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;
        Ok(Self { client, config })
    }

    /// Starts a new conversation with a default starting message.
    ///
    /// Conversations record message history.
    pub fn new_conversation(&self) -> Conversation {
        self.new_conversation_directed(
            format!("You are LLaMa, an AI model developed by OpenAI. Answer as concisely as possible. Today is: {0}", 
            chrono::Local::now().format("%d/%m/%Y %H:%M"))
        )
    }

    /// Starts a new conversation with a specified starting message.
    ///
    /// Conversations record message history.
    pub fn new_conversation_directed<S: Into<String>>(&self, direction_message: S) -> Conversation {
        Conversation::new(self.clone(), direction_message.into())
    }

    /// Explicitly sends whole message history to the API.
    ///
    /// In most cases, if you would like to store message history, you should be looking at the [`Conversation`] struct, and
    /// [`Self::new_conversation()`] and [`Self::new_conversation_directed()`]
    pub async fn send_history(
        &self,
        history: &Vec<ChatMessage>,
    ) -> chatgpt::Result<CompletionResponse> {
        let response: ServerResponse = self
            .client
            .post(self.config.api_url.clone())
            .json(&CompletionRequest {
                messages: history,
                stream: false,
                temperature: self.config.temperature,
                top_p: self.config.top_p,
                frequency_penalty: self.config.frequency_penalty,
                presence_penalty: self.config.presence_penalty,
                repeat_penalty: self.config.repeat_penalty,
                reply_count: self.config.reply_count,
            })
            .send()
            .await?
            .json()
            .await?;
        match response {
            ServerResponse::Error { error } => Err(chatgpt::err::Error::BackendError {
                message: error.message,
                error_type: error.error_type,
            }),
            ServerResponse::Completion(completion) => Ok(completion),
        }
    }

    /// Sends a single message to the API without preserving message history.
    pub async fn send_message<S: Into<String>>(
        &self,
        message: S,
    ) -> chatgpt::Result<CompletionResponse> {
        let response: ServerResponse = self
            .client
            .post(self.config.api_url.clone())
            .json(&CompletionRequest {
                //model: self.config.engine.as_ref(),
                messages: &vec![ChatMessage {
                    role: Role::User,
                    content: message.into(),
                }],
                stream: false,
                temperature: self.config.temperature,
                top_p: self.config.top_p,
                frequency_penalty: self.config.frequency_penalty,
                presence_penalty: self.config.presence_penalty,
                repeat_penalty: self.config.repeat_penalty,
                reply_count: self.config.reply_count,
            })
            .send()
            .await?
            .json()
            .await?;
        match response {
            ServerResponse::Error { error } => Err(chatgpt::err::Error::BackendError {
                message: error.message,
                error_type: error.error_type,
            }),
            ServerResponse::Completion(completion) => Ok(completion),
        }
    }
}
