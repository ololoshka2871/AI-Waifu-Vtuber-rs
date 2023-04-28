pub mod ai_translated_request;
pub mod chatgpt;
pub mod config;
pub mod deeplx_translate_owned;
pub mod dispatcher;
pub mod dummy_ai;
pub mod llama;
pub mod llama_client;
pub mod num2words;
pub mod silerio_tts;
pub mod whisper_voice_recognize;

pub mod utils;

#[allow(unused)]
static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn create_ai_dispatcher(config: &config::Config) -> Box<dyn dispatcher::Dispatcher> {
    use dispatcher::AIDispatcher;

    match &config.ai_engine {
        config::AIEngine::ChatGPT {
            openai_token,
            engine,
            temperature,
            top_p,
            presence_penalty,
            frequency_penalty,
            reply_count,
            api_url,
        } => {
            let mut model_config = ::chatgpt::prelude::ModelConfigurationBuilder::default();
            
            if let Some(engine) = engine {
                model_config.engine(match engine.as_str() {
                    "Gpt35Turbo" => ::chatgpt::prelude::ChatGPTEngine::Gpt35Turbo,
                    "Gpt35Turbo_0301" => ::chatgpt::prelude::ChatGPTEngine::Gpt35Turbo_0301,
                    "Gpt4" => ::chatgpt::prelude::ChatGPTEngine::Gpt4,
                    "Gpt4_32k" => ::chatgpt::prelude::ChatGPTEngine::Gpt4_32k,
                    "Gpt4_0314" => ::chatgpt::prelude::ChatGPTEngine::Gpt4_0314,
                    "Gpt4_32k_0314" => ::chatgpt::prelude::ChatGPTEngine::Gpt4_32k_0314,
                    _ => panic!("Invalid engine: {engine}, supported values Gpt35Turbo, Gpt35Turbo_0301, Gpt4, Gpt4_32k, Gpt4_0314, Gpt4_32k_0314"),
                });
            }
            if let Some(temperature) = temperature {
                model_config.temperature(*temperature);
            }
            if let Some(top_p) = top_p {
                model_config.top_p(*top_p);
            }
            if let Some(presence_penalty) = presence_penalty {
                model_config.presence_penalty(*presence_penalty);
            }
            if let Some(frequency_penalty) = frequency_penalty {
                model_config.frequency_penalty(*frequency_penalty);
            }
            if let Some(reply_count) = reply_count {
                model_config.reply_count(*reply_count);
            }
            if let Some(api_url) = api_url {
                model_config.api_url(api_url.clone());
            }

            Box::new(AIDispatcher::new(
                utils::chatgpt_en_deeplx_builder::ChatGPTEnAIBuilder::new(
                    openai_token.clone(),
                    model_config.build().unwrap(),
                    config,
                ),
            ))
        }
        config::AIEngine::LLaMa {
            api_url,
            temperature,
            top_p,
            presence_penalty,
            frequency_penalty,
            repeat_penalty,
            reply_count,
        } => {
            let mut model_config = llama::model_config::ModelConfigurationBuilder::default();
            model_config.api_url(api_url.clone());

            if let Some(temperature) = temperature {
                model_config.temperature(*temperature);
            }
            if let Some(top_p) = top_p {
                model_config.top_p(*top_p);
            }
            if let Some(presence_penalty) = presence_penalty {
                model_config.presence_penalty(*presence_penalty);
            }
            if let Some(frequency_penalty) = frequency_penalty {
                model_config.frequency_penalty(*frequency_penalty);
            }
            if let Some(repeat_penalty) = repeat_penalty {
                model_config.repeat_penalty(*repeat_penalty);
            }
            if let Some(reply_count) = reply_count {
                model_config.reply_count(*reply_count);
            }

            Box::new(AIDispatcher::new(
                utils::llama_en_deeplx_builder::LLaMaEnAIBuilder::new(
                    model_config.build().unwrap(),
                    config,
                ),
            ))
        }
    }
}
