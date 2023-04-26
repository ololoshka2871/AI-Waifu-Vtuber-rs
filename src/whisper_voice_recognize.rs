use reqwest::{self, Body, IntoUrl, Url};
use serde_json::Value;

#[derive(Clone)]
pub struct OpenAIWhisperVoice2Txt {
    voice2txt_url: Url,
}

impl OpenAIWhisperVoice2Txt {
    pub fn new<URL: IntoUrl>(url: URL) -> Self {
        Self {
            voice2txt_url: url.into_url().unwrap(),
        }
    }

    pub async fn recognize<V: Into<Body>>(
        &self,
        voice_data: V,
    ) -> Result<(String, String), String> {
        let client = reqwest::Client::new();
        let resp: Value = client
            .post(self.voice2txt_url.clone())
            .header("Content-Type", "audio/wav")
            .body(voice_data)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if let Value::Object(result) = resp {
            let lang = if let Value::String(language) = &result["language"] {
                language.clone()
            } else {
                return Err("Incorrect server response: language".to_string());
            };

            let string = if let Value::Array(transcribed_segments) = &result["transcribed_segments"]
            {
                transcribed_segments
                    .into_iter()
                    .map(|s| {
                        if let Value::Object(segment) = s {
                            if let Value::String(text) = &segment["text"] {
                                return text.clone();
                            }
                        }
                        " ".to_string()
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string()
            } else {
                return Err("Incorrect server response: transcribed_segments".to_string());
            };

            return Ok((string, lang));
        }
        Err("Failed to recognize, incorrect result".to_string())
    }
}
