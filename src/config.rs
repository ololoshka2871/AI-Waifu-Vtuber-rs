use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "OpenAI_Token")]
    pub openai_token: String, // OpenAI API token
    #[serde(rename = "AI_initial_prompt")]
    pub initial_prompt: String, // Initial prompt for the AI
    #[serde(rename = "Discord_Token")]
    pub discord_token: String, // Discord bot token
    #[serde(rename = "Discord_channel_whitelist")]
    pub channel_whitelist: Vec<String>, // Discord channel whitelist (empty = all channels), supports wildcards
}

impl Config {
    pub fn load() -> Self {
        let contents =
            std::fs::read_to_string("config.json").expect("Failed to read config.json file!");
        serde_json::from_str::<Config>(&contents).unwrap()
    }
}
