mod tests {
    use reqwest::Url;
    use serde::Deserialize;

    // https://serde.rs/enum-representations.html
    #[derive(Deserialize, PartialEq, Clone, Debug)]
    #[serde(tag = "type")]
    enum AIEngine {
        ChatGPT {
            #[serde(rename = "OpenAI_Token")]
            openai_token: String, // OpenAI API token
        },
        LLaMa {
            #[serde(rename = "LLaMa_URL")]
            llama_url: Url, // LLaMa API URL
        },
    }

    #[derive(Deserialize, PartialEq, Clone, Debug)]
    struct Cfg {
        pub engine: AIEngine,
    }

    #[tokio::test]
    async fn test_deserialise_json_enum() {
        let req_chatgpt = serde_json::json!(
            {
                "engine": {
                    "type": "ChatGPT",
                    "OpenAI_Token": "1234567890",
                }
            }
        );

        let req_llama = serde_json::json!(
            {
                "engine": {
                    "type": "LLaMa",
                    "LLaMa_URL": "http://localhost:1234/v1/bla-bla-bla",
                }
            }
        );

        let res_chatgpt: Cfg = serde_json::from_value(req_chatgpt).unwrap();
        assert_eq!(
            res_chatgpt.engine,
            AIEngine::ChatGPT {
                openai_token: "1234567890".to_string()
            }
        );

        let res_llama: Cfg = serde_json::from_value(req_llama).unwrap();
        assert_eq!(
            res_llama.engine,
            AIEngine::LLaMa {
                llama_url: Url::parse("http://localhost:1234/v1/bla-bla-bla").unwrap()
            }
        );
    }
}
