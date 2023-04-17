use reqwest::{IntoUrl, Url};
use wav;

use tracing::debug;

pub struct SilerioTTS {
    server_url: Url,
}

impl SilerioTTS {
    pub fn new<URL: IntoUrl>(server_url: URL) -> Self {
        Self {
            server_url: server_url.into_url().unwrap(),
        }
    }

    pub async fn say<S, SP>(&self, text: S, voice_id: SP) -> Result<(wav::Header, wav::BitDepth), String>
    where
        S: Into<String>,
        SP: Into<String>,
    {
        let client = reqwest::Client::new();
        let res = client
            .post(self.server_url.clone())
            .query(&[("voice_id", voice_id.into())])
            .body(text.into())
            .send()
            .await
            .map_err(|e| format!("Error: {}", e))?;

        // read response as wav file
        let res = res
            .bytes()
            .await
            .map_err(|e| format!("Http error: {}", e))?;

        let mut res = std::io::Cursor::new(res);
        let (header, wav_data) = wav::read(&mut res).map_err(|e| format!("Wav error: {}", e))?;

        debug!("Got audio file with header: {:?}", header);

        Ok((header, wav_data))
    }
}
