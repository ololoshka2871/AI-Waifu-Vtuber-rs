use reqwest::Url;
use serde::Deserialize;

fn default_silerio_bridge_url() -> Url {
    Url::parse("http://localhost:8961/say").unwrap()
}

fn default_jp_tts_bridge_url() -> Url {
    Url::parse("http://localhost:8231/say").unwrap()
}

fn default_openai_whisper_url() -> Url {
    Url::parse("http://localhost:3157/transcribe").unwrap()
}

fn auto() -> String {
    "auto".to_string()
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum AIEngineType {
    ChatGPT{
        // OpenAI API token
        #[serde(rename = "OpenAI_Token")]
        openai_token: String, 

        /// The GPT version used Gpt35Turbo, Gpt35Turbo_0301, Gpt4, Gpt4_32k, Gpt4_0314, Gpt4_32k_0314,
        #[serde(rename = "GPT_Version")]
        engine: Option<String>,
    },
    LLaMa {
         /// URL of the /v1/chat/completions endpoint. Can be used to set a proxy
         #[serde(rename = "Url")]
         api_url: Url,
    }
}

#[derive(Deserialize)]
pub struct AIEngine {
    #[serde(rename = "Engine_Type")]
    pub engine_type: AIEngineType,

    /// Controls randomness of the output. Higher valeus means more random
    #[serde(rename = "Temperature")]
    pub temperature: Option<f32>,

    /// Controls diversity via nucleus sampling, not recommended to use with temperature
    #[serde(rename = "Top_p")]
    pub top_p:  Option<f32>,

    /// Determines how much to penalize new tokens pased on their existing presence so far
    #[serde(rename = "Presence_penalty")]
    pub presence_penalty:  Option<f32>,

    /// Determines how much to penalize new tokens based on their existing frequency so far
    #[serde(rename = "Frequency_penalty")]
    pub frequency_penalty:  Option<f32>,

    /// The maximum amount of replies
    #[serde(rename = "Reply_count")]
    pub reply_count:  Option<u32>,
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
#[serde(tag = "type")]
pub enum TTSConfig {
    Disabled,
    SilerioTTSConfig{
        #[serde(rename = "TTS_Service_Url", default = "default_silerio_bridge_url")]
        tts_service_url: Url, // TTS service URL
        #[serde(rename = "Voice_character")]
        voice_character: Option<String>, // Voice character name (like "ksenia")
    },
    JPVoicesTTSConfig{
        #[serde(rename = "TTS_Service_Url", default = "default_jp_tts_bridge_url")]
        tts_service_url: Url, // TTS service URL
        #[serde(rename = "Voice_character")]
        voice_character: Option<u32>, // Voice character id 0, 1... see external_services/jp-voice/voice_synthesizer_dist/app.py
        #[serde(rename = "Voice_duration")]
        voice_duration: Option<f32>, // Voice tempo
    }
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
    #[serde(rename = "TTS_Config")]
    pub tts_config: TTSConfig, // TTS config
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
