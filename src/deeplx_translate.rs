use async_trait::async_trait;

use reqwest::{self, IntoUrl, Url};

use tracing::debug;

use crate::{
    ai_translated_request::TranslatedAIRequest,
    dispatcher::{AIError, AIRequest, AIinterface},
};

/// decorator for AI interface
/// get input from user, translate it to english and send to AI
/// get output from AI, translate it to user language and send to user
pub struct DeepLxTranslator {
    src_lang: Option<String>,
    dest_lang: String,

    deeplx_url: Url,

    ai: Box<dyn AIinterface>,
}

impl DeepLxTranslator {
    /**
     * ai - An AI object implements AIinterface
     * src_lang - Source text language or None (Auto)
     * dest_lang - If None - src lang
     * deeplx_url - DeepLx service URL. See https://hub.docker.com/r/missuo/deeplx docker image
     */
    pub fn new<URL: IntoUrl>(
        ai: Box<dyn AIinterface>,
        mut src_lang: Option<String>,
        dest_lang: Option<String>,
        url: URL,
    ) -> Self {
        assert!(
            src_lang.is_some() || dest_lang.is_some(),
            "Langs mast not be None both in a same time!"
        );

        let dl = if let Some(dest_lang) = dest_lang {
            if src_lang.is_none() {
                src_lang = Some(dest_lang.clone())
            }
            dest_lang
        } else {
            "en".to_string()
        };

        Self {
            src_lang,
            dest_lang: dl,

            deeplx_url: url.into_url().unwrap(),

            ai,
        }
    }

    pub async fn translate<S, SRC, DEST>(
        &self,
        text: S,
        src_lang: Option<SRC>,
        dest_lang: DEST,
    ) -> Result<String, String>
    where
        S: Into<String>,
        SRC: Into<String>,
        DEST: Into<String>,
    {
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
            if let serde_json::Value::String(s) = &result["data"] {
                Ok(s.clone())
            } else {
                Ok(result["data"].to_string())
            }
        } else {
            Err("Failed to reanslate, incorrect result".to_string())
        }
    }
}

#[async_trait]
impl AIinterface for DeepLxTranslator {
    async fn process(&mut self, request: Box<dyn AIRequest>) -> Result<String, AIError> {
        let r = request.request();

        // translate input to english
        let translated = self
            .translate(r.clone(), self.src_lang.clone(), "en")
            .await
            .map_err(|e| AIError::TranslateError(e))?;
        debug!("{r} ({lang:?}) => {translated}", lang = &self.src_lang);

        // preocess AI request
        let answer = self
            .ai
            .process(Box::new(TranslatedAIRequest::new(request, translated)))
            .await?;

        // translate answer to user language
        let res = self
            .translate(answer.clone(), Some("en"), self.dest_lang.clone())
            .await
            .map_err(|e| AIError::TranslateError(e))?;
        debug!("{answer} => {res} ({lang})", lang = &self.dest_lang);

        Ok(res)
    }
}
