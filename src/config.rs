use std::path::PathBuf;

use reqwest::Url;
use serde::Deserialize;

fn default_deeplx_url() -> Url {
    Url::parse("http://localhost:1188/translate").unwrap()
}

fn default_silerio_bridge_url() -> Url {
    Url::parse("http://localhost:8961/say").unwrap()
}

fn default_urukhan_v2t_url() -> Url {
    Url::parse("http://localhost:3154/recognize").unwrap()
}

fn default_models_store_path() -> PathBuf {
    PathBuf::from(crate::CARGO_MANIFEST_DIR).join("models")
}

fn auto() -> String {
    "auto".to_string()
}

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
    #[serde(rename = "DeepLx_Url", default = "default_deeplx_url")]
    pub deeplx_url: Url, // Optional DeepLx translatin service Url
    #[serde(rename = "Speaker_lang", default = "auto")]
    pub src_lang: String, // Optional request language
    #[serde(rename = "Answer_lang")]
    pub dest_lang: String, // Answer langualge
    #[serde(rename = "TTS_Service_Url", default = "default_silerio_bridge_url")]
    pub tts_service_url: Url, // TTS service URL
    #[serde(rename = "Voice_character")]
    pub voice_character: Option<String>, // Voice character name (like "ksenia")
    #[serde(rename = "Voice_language")]
    pub voice_language: String, // Voice language (like "ru")
    #[serde(rename = "Voice_model")]
    pub voice_model: String, // Voice model name (like "standard")
    #[serde(rename = "Busy_messages")]
    pub busy_messages: Vec<String>, // Messages to send when the AI is busy
    #[serde(rename = "Voice2txt_Url", default = "default_urukhan_v2t_url")]
    pub voice2txt_url: Url, // Optional voice to text service URL
    #[serde(rename = "ModelsStorePath", default = "default_models_store_path")]
    pub models_store_path: PathBuf, // Path to the models store
    #[serde(rename = "LLaMa_URL")]
    pub llama_url: Option<Url>, // LLaMa API URL
}

impl Config {
    pub fn load() -> Self {
        let contents =
            std::fs::read_to_string("config.json").expect("Failed to read config.json file!");
        serde_json::from_str::<Config>(&contents).unwrap()
    }
}
