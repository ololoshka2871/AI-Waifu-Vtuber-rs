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
    //deeplx_url: reqwest::Url,
}

impl From<&Config> for ChatGPTEnAIBuilder {
    fn from(config: &Config) -> Self {
        Self {
            openai_token: config.openai_token.clone(),
            initial_prompt: config.initial_prompt.clone(),
            src_lang: config.src_lang.clone(),
            dest_lang: config.dest_lang.clone(),

            //deeplx_url: config.deeplx_url.clone(),
        }
    }
}

impl AIBuilder for ChatGPTEnAIBuilder {
    fn build(&mut self) -> Box<dyn AIinterface> {
        let ai = ChatGPT::new(self.openai_token.clone(), self.initial_prompt.clone());

        let en_ai = 
        //deeplx_translate::DeepLxTranslator::new(
        //    Box::new(ai),
        //    Some(self.src_lang.clone()),
        //    Some(self.dest_lang.clone()),
        //    self.deeplx_url.clone(),
        //);
        crate::deeplx_translate_owned::DeepLxTranslatorOwned::new(
            Box::new(ai),
            Some(self.src_lang.clone()),
            Some(self.dest_lang.clone()),
            Some(0.55),
        );

        Box::new(en_ai)
    }
}
