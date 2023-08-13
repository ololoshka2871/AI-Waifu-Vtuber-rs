/// Use silerio python TTS server to generate speech from text
/// start server https://github.com/ololoshka2871/Selerio-TTS-server on any port and add it to config as "TTS_Service_Url"

use std::io::Cursor;

use reqwest::{IntoUrl, Url};

use bytes::Bytes;

pub struct SilerioTTS {
    server_url: Url,
}

impl SilerioTTS {
    pub fn new<URL: IntoUrl>(server_url: URL) -> Self {
        Self {
            server_url: server_url.into_url().unwrap(),
        }
    }

    pub async fn say<S, SP>(&self, text: S, voice_id: Option<SP>) -> Result<Cursor<Bytes>, String>
    where
        S: Into<String>,
        SP: Into<String>,
    {
        let client = reqwest::Client::new();
        let res = client
            .post(self.server_url.clone());

        let res = if let Some(voice_id) = voice_id {
            res.query(&[("voice_id", voice_id.into())])
        } else {
            res
        };

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
