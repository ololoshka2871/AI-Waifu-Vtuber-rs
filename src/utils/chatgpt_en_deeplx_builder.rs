use crate::{
    chatgpt::ChatGPT,
    config::Config,
    dispatcher::{AIBuilder, AIinterface},
};

pub struct ChatGPTEnAIBuilder {
    openai_token: String,
    initial_prompt: String,
    src_lang: String,
    dest_lang: String,
}

impl ChatGPTEnAIBuilder {
    pub fn new(openai_token: String, config: &Config) -> Self {
        Self {
            openai_token,
            initial_prompt: config.initial_prompt.clone(),
            src_lang: config.deeplx_translate_config.src_lang.clone(),
            dest_lang: config.deeplx_translate_config.dest_lang.clone(),
        }
    }
}

impl AIBuilder for ChatGPTEnAIBuilder {
    fn build(&mut self) -> Box<dyn AIinterface> {
        let ai = ChatGPT::new(self.openai_token.clone(), self.initial_prompt.clone());

        let en_ai = 

        crate::deeplx_translate_owned::DeepLxTranslatorOwned::new(
            Box::new(ai),
            Some(self.src_lang.clone()),
            Some(self.dest_lang.clone()),
            Some(0.55),
        );

        Box::new(en_ai)
    }
}
