use std::io::Cursor;

use bytes::Bytes;

use crate::{config, jp_tts::JpTTS, silerio_tts::SilerioTTS};

pub enum TTSEngine {
    NullTTS,
    SilerioTTS(SilerioTTS),
    JpTTS(JpTTS),
}

impl TTSEngine {
    pub fn with_config(config: &config::TTSConfig) -> Self {
        match config {
            config::TTSConfig::Disabled => Self::NullTTS,
            config::TTSConfig::SilerioTTSConfig {
                tts_service_url,
                voice_character,
            } => Self::SilerioTTS(SilerioTTS::new(
                tts_service_url.clone(),
                voice_character.clone(),
            )),
            config::TTSConfig::JPVoicesTTSConfig {
                tts_service_url,
                voice_character,
                voice_duration,
            } => Self::JpTTS(JpTTS::new(
                tts_service_url.clone(),
                voice_character.clone(),
                voice_duration.clone(),
            )),
        }
    }

    pub async fn say<S>(&self, text: S) -> Result<Cursor<Bytes>, String>
    where
        S: Into<String>,
    {
        match self {
            TTSEngine::NullTTS => Ok(Cursor::new(Bytes::new())),
            TTSEngine::SilerioTTS(tts) => tts.say(text).await,
            TTSEngine::JpTTS(tts) => tts.say(text).await,
        }
    }
}
