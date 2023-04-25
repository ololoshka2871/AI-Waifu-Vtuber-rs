use chatgpt::types::ChatMessage;
use serde::Serialize;


/// A request struct sent to the API to request a message completion
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct CompletionRequest<'a> {
    /// The message history, including the message that requires completion, which should be the last one
    pub messages: &'a Vec<ChatMessage>,
    /// Whether the message response should be gradually streamed
    pub stream: bool,
    /// The extra randomness of response
    pub temperature: f32,
    /// Controls diversity via nucleus sampling, not recommended to use with temperature
    pub top_p: f32,
    /// Determines how much to penalize new tokens based on their existing frequency so far
    pub frequency_penalty: f32,
    /// Determines how much to penalize new tokens pased on their existing presence so far
    pub presence_penalty: f32,
    /// Frequency Penalty for repeated tokens
    pub repeat_penalty: f32,
    /// Determines the amount of output responses
    #[serde(rename = "n")]
    pub reply_count: u32,
}