use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub openai_token: String, // OpenAI API token
}

impl Config {
    pub fn load() -> Self {
        let contents = std::fs::read_to_string("config.json").expect("Failed to read config.json file!");
        serde_json::from_str::<Config>(&contents).unwrap()
    }
}