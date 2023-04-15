use async_trait::async_trait;

use reqwest;

use tracing::debug;

use crate::{
    ai_translated_request::TranslatedAIRequest,
    dispatcher::{AIError, AIRequest, AIinterface},
};

/// decorator for AI interface
/// get input from user, translate it to english and send to AI
/// get output from AI, translate it to user language and send to user
/// if user language is english, then send output to user without translation
/// if user language not specified, then detect it automatically
/// if dest language not specified, then the same as input
pub struct DeepLxTranslator {
    src_lang: Option<String>,
    dest_lang: Option<String>,

    deeplx_url: String,

    ai: Box<dyn AIinterface>,
}

impl DeepLxTranslator {
    pub async fn new(
        ai: Box<dyn AIinterface>,
        src_lang: Option<String>,
        dest_lang: Option<String>,
        deeplx_url: String,
    ) -> Self {
        Self {
            src_lang,
            dest_lang,

            deeplx_url,

            ai,
        }
    }

    pub async fn translate<S: Into<String>>(
        &self,
        text: S,
        src_lang: Option<S>,
        dest_lang: S,
    ) -> Result<(String, Option<String>), String> {
        use maplit::hashmap;

        let req = hashmap! {
            "text" => text.into(),
            "source_lang" => match src_lang {
                Some(s) => s.into(),
                None => "auto".to_string(),
            },
            "target_lang" => dest_lang.into(),
        };

        let client = reqwest::Client::new();
        let resp: serde_json::Value = client
            .post(self.deeplx_url.clone())
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if let serde_json::Value::Object(result) = resp {
            Ok((result["data"].to_string(), None))
        } else {
            Err("Failed to reanslate, incorrect result".to_string())
        }
    }

    async fn translate_to_en(&self, text: String) -> Result<(String, Option<String>), AIError> {
        let src_lang = if let Some(src_lang) = &self.src_lang {
            Some(src_lang.clone())
        } else {
            None
        };

        if src_lang.is_none() || src_lang.as_deref().unwrap() != "en" {
            // translate to english
            Ok(self
                .translate(text, src_lang, "en".to_string())
                .await
                .map_err(|e| AIError::TranslateError(e))?)
        } else {
            Ok((text, None))
        }
    }

    async fn translate_from_en(
        &self,
        text: String,
        source_language: Option<String>,
    ) -> Result<String, AIError> {
        let dest_lang = if let Some(dest_lang) = &self.dest_lang {
            dest_lang.clone()
        } else if let Some(source_language) = source_language {
            if source_language == "en" || source_language == "auto" {
                return Ok(text);
            } else {
                source_language
            }
        } else {
            return Ok(text);
        };

        match self
            .translate(text, Some("en".to_string()), dest_lang)
            .await
        {
            Ok((res, _)) => Ok(res),
            Err(e) => Err(AIError::TranslateError(e)),
        }
    }
}

#[async_trait]
impl AIinterface for DeepLxTranslator {
    async fn process(&mut self, request: Box<dyn AIRequest>) -> Result<String, AIError> {
        let r = request.request();

        // translate input to english
        let (translated, lang) = self.translate_to_en(r.clone()).await?;
        debug!("{r} ({lang:?}) => {translated}");

        // preocess AI request
        let answer = self
            .ai
            .process(Box::new(TranslatedAIRequest::new(request, translated)))
            .await?;

        // translate answer to user language
        let res = self.translate_from_en(answer.clone(), lang.clone()).await?;
        debug!("{answer} => {res} ({lang:?})");

        Ok(res)
    }
}
