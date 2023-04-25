use crate::{
    config::Config,
    dispatcher::{AIBuilder, AIinterface},
    llama::LLaMa,
};

pub struct LLaMaEnAIBuilder {
    initial_prompt: String,
    src_lang: String,
    dest_lang: String,
    llama_url: reqwest::Url,
}

impl From<&Config> for LLaMaEnAIBuilder {
    fn from(config: &Config) -> Self {
        Self {
            initial_prompt: config.initial_prompt.clone(),
            src_lang: config.src_lang.clone(),
            dest_lang: config.dest_lang.clone(),
            llama_url: config.llama_url.clone(),
        }
    }
}

impl AIBuilder for LLaMaEnAIBuilder {
    fn build(&mut self) -> Box<dyn AIinterface> {
        let ai = LLaMa::new(self.llama_url.clone(), self.initial_prompt.clone());

        let en_ai = crate::deeplx_translate_owned::DeepLxTranslatorOwned::new(
            Box::new(ai),
            Some(self.src_lang.clone()),
            Some(self.dest_lang.clone()),
            Some(0.55),
        );

        Box::new(en_ai)
    }
}
