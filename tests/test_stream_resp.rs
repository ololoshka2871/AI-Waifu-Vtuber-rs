mod tests {
    use ai_waifu::{config::Config, utils::test_request::TestRequest};
    use reqwest::Url;

    // не работает с LM Studio
    // Работает с ghcr.io/abetlen/llama-cpp-python:latest и то через раз
    #[tokio::test]
    async fn test_stream_response() {
        let mut chatgpt = ai_waifu::create_streamed_ai(&{
            let mut cfg = Config::default();
            cfg.ai_engine.engine_type = ai_waifu::config::AIEngineType::LLaMa {
                api_url: Url::parse("http://localhost:8000/v1/chat/completions").unwrap(),
                model: Some("TheBloke/zephyr-7B-beta-GGUF".to_owned()),
            };
            cfg
        });

        let req = TestRequest {
            request: "Мама мыла раму.".to_string(),
            channel: "Master".to_string(),
        };
        let _request = chatgpt.process_streamed(Box::new(req)).await;
    }
}
