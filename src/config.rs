use reqwest::Url;
use serde::Deserialize;

fn default_silerio_bridge_url() -> Url {
    Url::parse("http://localhost:8961/say").unwrap()
}

fn default_openai_whisper_url() -> Url {
    Url::parse("http://localhost:3157/transcribe").unwrap()
}

fn auto() -> String {
    "auto".to_string()
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum AIEngine {
    ChatGPT {
        #[serde(rename = "OpenAI_Token")]
        openai_token: String, // OpenAI API token
    },
    LLaMa {
        #[serde(rename = "LLaMa_URL")]
        llama_url: Url, // LLaMa API URL
    },
}

#[derive(Deserialize)]
pub struct DiscordConfig {
    #[serde(rename = "Discord_Token")]
    pub discord_token: String, // Discord bot token
    #[serde(rename = "Discord_channel_whitelist")]
    pub channel_whitelist: Vec<String>, // Discord channel whitelist (empty = all channels), supports wildcards
}

#[derive(Deserialize)]
pub struct DeepLxTranslateConfig {
    #[serde(rename = "Speaker_lang", default = "auto")]
    pub src_lang: String, // Optional request language
    #[serde(rename = "Answer_lang")]
    pub dest_lang: String, // Answer langualge
}

#[derive(Deserialize)]
pub struct SilerioTTSConfig {
    #[serde(rename = "TTS_Service_Url", default = "default_silerio_bridge_url")]
    pub tts_service_url: Url, // TTS service URL
    #[serde(rename = "Voice_character")]
    pub voice_character: Option<String>, // Voice character name (like "ksenia")
    #[serde(rename = "Voice_language")]
    pub voice_language: String, // Voice language (like "ru")
    #[serde(rename = "Voice_model")]
    pub voice_model: String, // Voice model name (like "ru_v3")
}

#[derive(Deserialize)]
pub struct STTConfig {
    #[serde(rename = "STT_Url", default = "default_openai_whisper_url")]
    pub voice2txt_url: Url, // Optional voice to text service URL
    #[serde(rename = "Drop_Nonconfident_Translate_lvl")]
    pub drop_nonconfident_translate_result: Option<f64>, // drop translate it confidens < value
    #[serde(rename = "Minimal_audio_fragment_length")]
    pub minimal_audio_fragment_length: f32, // Minimal audio fragment length in seconds
    #[serde(rename = "Maximal_audio_fragment_length")]
    pub maximal_audio_fragment_length: f32, // Maximal audio fragment length in seconds
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "AIEngine")]
    pub ai_engine: AIEngine, // AI engine
    #[serde(rename = "AI_initial_prompt")]
    pub initial_prompt: String, // Initial prompt for the AI
    #[serde(rename = "Discord_Config")]
    pub discord_config: DiscordConfig, // Discord bot config
    #[serde(rename = "DeepLx_Translate_Config")]
    pub deeplx_translate_config: DeepLxTranslateConfig, // DeepLx translate config
    #[serde(rename = "Silerio_TTS_Config")]
    pub silerio_tts_config: SilerioTTSConfig, // Silerio TTS config
    #[serde(rename = "Busy_messages")]
    pub busy_messages: Vec<String>, // Messages to send when the AI is busy
    #[serde(rename = "STT_Config")]
    pub stt_config: STTConfig, // STT config
}

impl Config {
    pub fn load() -> Self {
        let contents =
            std::fs::read_to_string("config.json").expect("Failed to read config.json file!");
        serde_json::from_str::<Config>(&contents).unwrap()
    }
}
