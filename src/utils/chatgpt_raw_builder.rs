use chatgpt::prelude::ModelConfiguration;

use crate::{
    chatgpt::ChatGPT,
    config::Config,
    dispatcher::{AIBuilder, AIinterface},
};

pub struct ChatGPTRawAIBuilder {
    openai_token: String,
    config: ModelConfiguration,
    initial_prompt: String,
}

impl ChatGPTRawAIBuilder {
    pub fn new(openai_token: String, model_config: ModelConfiguration, config: &Config) -> Self {
        Self {
            openai_token,
            config: model_config,
            initial_prompt: config.initial_prompt.clone(),
        }
    }
}

impl AIBuilder for ChatGPTRawAIBuilder {
    fn build(&mut self) -> Box<dyn AIinterface> {
        let ai = ChatGPT::new(
            self.openai_token.clone(),
            self.config.clone(),
            self.initial_prompt.clone(),
        );
        Box::new(ai)
    }
}
