/// Use DeepLx service to translate text
/// start server https://github.com/ololoshka2871/Voice-2-txt-UrukHan on any port and add it to config as "Voice2txt_Url"
use reqwest::{self, Body, IntoUrl, Url};

#[derive(Clone)]
pub struct UrukHanVoice2Txt {
    voice2txt_url: Url,
}

impl UrukHanVoice2Txt {
    pub fn new<URL: IntoUrl>(url: URL) -> Self {
        Self {
            voice2txt_url: url.into_url().unwrap(),
        }
    }

    pub async fn recognize<V: Into<Body>>(&self, voice_data: V) -> Result<String, String> {
        let client = reqwest::Client::new();
        let resp: serde_json::Value = client
            .post(self.voice2txt_url.clone())
            .header("Content-Type", "audio/wav")
            .body(voice_data)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if let serde_json::Value::Object(result) = resp {
            if let serde_json::Value::String(corrected_text) = &result["corrected_text"] {
                if corrected_text.len() > 0 {
                    return Ok(corrected_text.clone());
                } else {
                    return Err("No text found".to_string());
                }
            }
        }
        Err("Failed to recognize, incorrect result".to_string())
    }
}
