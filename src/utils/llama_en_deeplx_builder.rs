use crate::{
    config::Config,
    dispatcher::{AIBuilder, AIinterface},
    llama::model_config::{ModelConfiguration},
    llama_client::LLaMa,
};

pub struct LLaMaEnAIBuilder {
    initial_prompt: String,
    src_lang: String,
    dest_lang: String,
    llama_config: ModelConfiguration,
    drop_nonconfident_result: Option<f64>,
}

impl LLaMaEnAIBuilder {
    pub fn new(model_config: ModelConfiguration, config: &Config) -> Self {

        Self {
            initial_prompt: config.initial_prompt.clone(),
            src_lang: config.deeplx_translate_config.src_lang.clone(),
            dest_lang: config.deeplx_translate_config.dest_lang.clone(),
            llama_config: model_config,
            drop_nonconfident_result: config.stt_config.drop_nonconfident_translate_result,
        }
    }
}

impl AIBuilder for LLaMaEnAIBuilder {
    fn build(&mut self) -> Box<dyn AIinterface> {
        let ai = LLaMa::new(self.llama_config.clone(), self.initial_prompt.clone());

        let en_ai = crate::deeplx_translate_owned::DeepLxTranslatorOwned::new(
            Box::new(ai),
            Some(self.src_lang.clone()),
            Some(self.dest_lang.clone()),
            self.drop_nonconfident_result,
        );

        Box::new(en_ai)
    }
}
