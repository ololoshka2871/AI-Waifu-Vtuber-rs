pub mod ai_translated_request;
pub mod chatgpt;
pub mod config;
pub mod deeplx_translate_owned;
pub mod dispatcher;
pub mod dummy_ai;
pub mod llama;
pub mod whisper_voice_recognize;
pub mod silerio_tts;

pub mod utils;

static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn create_ai_dispatcher(config: &config::Config) -> Box<dyn dispatcher::Dispatcher> {
    use dispatcher::AIDispatcher;

    if config.openai_token.is_some() {
        Box::new(AIDispatcher::new(
            utils::chatgpt_en_deeplx_builder::ChatGPTEnAIBuilder::from(config),
        ))
    } else {
        Box::new(AIDispatcher::new(
            utils::llama_en_deeplx_builder::LLaMaEnAIBuilder::from(config),
        ))
    }
}
