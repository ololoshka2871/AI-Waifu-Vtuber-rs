/// Use silerio python TTS server to generate speech from text
/// start server https://github.com/ololoshka2871/Selerio-TTS-server on any port and add it to config as "TTS_Service_Url"
use std::io::Cursor;

use reqwest::IntoUrl;

use bytes::Bytes;

pub struct SilerioTTS {
    _client: reqwest::Client,
    builder: reqwest::RequestBuilder,
}

impl SilerioTTS {
    pub fn new<URL: IntoUrl>(server_url: URL, voice_character: Option<String>) -> Self {
        let client = reqwest::Client::new();
        let builder = client.post(server_url.into_url().unwrap());

        let builder = if let Some(voice_character) = voice_character {
            builder.query(&[("voice_id", voice_character)])
        } else {
            builder
        };

        Self {
            _client: client,
            builder,
        }
    }

    pub async fn say<S>(&self, text: S) -> Result<Cursor<Bytes>, String>
    where
        S: Into<String>,
    {
        let res = self.builder.try_clone().unwrap();

        let res = res
            .body(text.into())
            .send()
            .await
            .map_err(|e| format!("Error: {}", e))?;

        // read response as wav file
        let res = res
            .bytes()
            .await
            .map_err(|e| format!("Http error: {}", e))?;

        Ok(Cursor::new(res))
    }
}
