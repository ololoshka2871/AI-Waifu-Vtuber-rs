pub mod ai_translated_request;
pub mod chatgpt;
pub mod config;
pub mod deeplx_translate_owned;
pub mod dispatcher;
pub mod dummy_ai;
pub mod num2words;
pub mod whisper_voice_recognize;

pub mod jp_tts;
pub mod silerio_tts;
pub mod tts_engine;

pub mod utils;

#[allow(unused)]
static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn create_ai_dispatcher(config: &config::Config) -> Box<dyn dispatcher::Dispatcher> {
    use dispatcher::AIDispatcher;

    let mut ai_config = ::chatgpt::prelude::ModelConfigurationBuilder::default();

    // common config
    let ai_conf = &config.ai_engine;
    if let Some(temperature) = ai_conf.temperature {
        ai_config.temperature(temperature);
    }
    if let Some(top_p) = ai_conf.top_p {
        ai_config.top_p(top_p);
    }
    if let Some(presence_penalty) = ai_conf.presence_penalty {
        ai_config.presence_penalty(presence_penalty);
    }
    if let Some(frequency_penalty) = ai_conf.frequency_penalty {
        ai_config.frequency_penalty(frequency_penalty);
    }
    if let Some(reply_count) = ai_conf.reply_count {
        ai_config.reply_count(reply_count);
    }

    match &ai_conf.engine_type {
        config::AIEngineType::ChatGPT {
            openai_token,
            engine,
            raw,
        } => {
            if let Some(engine) = engine {
                ai_config.engine(match engine.as_str() {
                    "Gpt35Turbo" => ::chatgpt::prelude::ChatGPTEngine::Gpt35Turbo,
                    "Gpt35Turbo_0301" => ::chatgpt::prelude::ChatGPTEngine::Gpt35Turbo_0301,
                    "Gpt4" => ::chatgpt::prelude::ChatGPTEngine::Gpt4,
                    "Gpt4_32k" => ::chatgpt::prelude::ChatGPTEngine::Gpt4_32k,
                    "Gpt4_0314" => ::chatgpt::prelude::ChatGPTEngine::Gpt4_0314,
                    "Gpt4_32k_0314" => ::chatgpt::prelude::ChatGPTEngine::Gpt4_32k_0314,
                    _ => panic!("Invalid engine: {engine}, supported values Gpt35Turbo, Gpt35Turbo_0301, Gpt4, Gpt4_32k, Gpt4_0314, Gpt4_32k_0314"),
                });
            }

            if *raw {
                Box::new(AIDispatcher::new(
                    utils::chatgpt_raw_builder::ChatGPTRawAIBuilder::new(
                        openai_token.clone(),
                        ai_config.build().unwrap(),
                        config,
                    ),
                    config.ai_engine.context_path.clone(),
                ))
            } else {
                Box::new(AIDispatcher::new(
                    utils::chatgpt_en_deeplx_builder::ChatGPTEnAIBuilder::new(
                        openai_token.clone(),
                        ai_config.build().unwrap(),
                        config,
                    ),
                    config.ai_engine.context_path.clone(),
                ))
            }
        }
        config::AIEngineType::LLaMa { api_url, raw } => {
            ai_config.api_url(api_url.clone()); // set local url (llama server)

            if *raw {
                Box::new(AIDispatcher::new(
                    utils::chatgpt_raw_builder::ChatGPTRawAIBuilder::new(
                        "no-token".to_string(),
                        ai_config.build().unwrap(),
                        config,
                    ),
                    config.ai_engine.context_path.clone(),
                ))
            } else {
                Box::new(AIDispatcher::new(
                    utils::chatgpt_en_deeplx_builder::ChatGPTEnAIBuilder::new(
                        "no-token".to_string(),
                        ai_config.build().unwrap(),
                        config,
                    ),
                    config.ai_engine.context_path.clone(),
                ))
            }
        }
    }
}
