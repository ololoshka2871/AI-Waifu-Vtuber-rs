use std::str::FromStr;

use derive_builder::Builder;
use reqwest::Url;

/// The struct containing main configuration for the ChatGPT API
#[derive(Debug, Clone, PartialEq, PartialOrd, Builder)]
#[builder(default, setter(into))]
pub struct ModelConfiguration {
    /// Controls randomness of the output. Higher valeus means more random
    pub temperature: f32,
    /// Controls diversity via nucleus sampling, not recommended to use with temperature
    pub top_p: f32,
    /// Determines how much to penalize new tokens pased on their existing presence so far
    pub presence_penalty: f32,
    /// Determines how much to penalize new tokens based on their existing frequency so far
    pub frequency_penalty: f32,
    /// Frequency Penalty for repeated tokens
    pub repeat_penalty: f32,
    /// The maximum amount of replies
    pub reply_count: u32,
    /// URL of the /v1/chat/completions endpoint. Can be used to set a proxy
    pub api_url: Url,
}

impl Default for ModelConfiguration {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            top_p: 0.95,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
            repeat_penalty: 1.1,
            reply_count: 1,
            api_url: Url::from_str("https://localhost:8000/v1/chat/completions").unwrap(),
        }
    }
}
