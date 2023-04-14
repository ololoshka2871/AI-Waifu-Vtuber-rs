use async_trait::async_trait;
use google_translator;

use tracing::debug;

use crate::dispatcher::{AIError, AIRequest, AIinterface};

/// decorator for AI interface
/// get input from user, translate it to english and send to AI
/// get output from AI, translate it to user language and send to user
/// if user language is english, then send output to user without translation
/// if user language not specified, then detect it automatically
/// if dest language not specified, then the same as input
pub struct GoogleTranslator {
    src_lang: Option<String>,
    dest_lang: Option<String>,

    ai: Box<dyn AIinterface>,
}

impl GoogleTranslator {
    pub async fn new(
        ai: Box<dyn AIinterface>,
        src_lang: Option<String>,
        dest_lang: Option<String>,
    ) -> Self {
        Self {
            src_lang,
            dest_lang,

            ai,
        }
    }

    async fn translate(
        &self,
        text: String,
        src_lang: Option<String>,
        dest_lang: String,
    ) -> Result<(String, Option<String>), String> {
        match google_translator::translate(
            vec![text],
            src_lang.unwrap_or("auto".to_string()),
            dest_lang,
        )
        .await
        {
            Ok(res) => Ok((res.output_text[0][0].clone(), Some(res.input_lang))),
            Err(e) => Err(e),
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
impl AIinterface for GoogleTranslator {
    async fn process(&self, request: Box<dyn AIRequest>) -> Result<String, AIError> {
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

struct TranslatedAIRequest {
    original: Box<dyn AIRequest>,
    en_text: String,
}

impl TranslatedAIRequest {
    fn new(original: Box<dyn AIRequest>, en_text: String) -> Self {
        Self { original, en_text }
    }
}

impl AIRequest for TranslatedAIRequest {
    fn request(&self) -> String {
        self.en_text.clone()
    }

    fn author(&self) -> String {
        self.original.author()
    }
}
