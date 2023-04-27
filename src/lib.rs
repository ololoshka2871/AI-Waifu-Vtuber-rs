pub mod ai_translated_request;
pub mod chatgpt;
pub mod config;
pub mod deeplx_translate_owned;
pub mod dispatcher;
pub mod dummy_ai;
pub mod llama;
pub mod silerio_tts;
pub mod whisper_voice_recognize;
pub mod utils;

#[allow(unused)]
static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn create_ai_dispatcher(config: &config::Config) -> Box<dyn dispatcher::Dispatcher> {
    use dispatcher::AIDispatcher;

    match &config.ai_engine {
        config::AIEngine::ChatGPT { openai_token } => Box::new(AIDispatcher::new(
            utils::chatgpt_en_deeplx_builder::ChatGPTEnAIBuilder::new(openai_token.clone(), config),
        )),
        config::AIEngine::LLaMa { llama_url } => Box::new(AIDispatcher::new(
            utils::llama_en_deeplx_builder::LLaMaEnAIBuilder::new(llama_url.clone(), config),
        )),
        _ => panic!("Unsupported AI engine, please check your config file."),
    }
}
