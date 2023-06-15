use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;

use maplit::hashmap;
use reqwest::header;
use serde_json::Value;
use tracing::{debug, trace};

use crate::{
    ai_translated_request::TranslatedAIRequest,
    dispatcher::{AIError, AIRequest, AIResponseType, AIinterface},
};

static DEEPLX_URL: &str = "https://www2.deepl.com/jsonrpc";

fn get_i_count(translate_text: &str) -> u64 {
    translate_text.chars().filter(|c| *c == 'i').count() as u64
}

fn get_time_stamp(mut i_count: u64) -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let ts = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;
    if i_count != 0 {
        i_count = i_count.wrapping_add(1);
        return ts - ts % i_count + i_count;
    } else {
        return ts;
    }
}

/// Rust port of https://github.com/OwO-Network/DeepLX/blob/main/main.go
pub struct DeepLxTranslatorOwned {
    src_lang: Option<String>,
    dest_lang: String,
    ai: Box<dyn AIinterface>,
    id: i64,
    drop_nonconfident_result: Option<f64>,

    pub headers: header::HeaderMap,
}

impl DeepLxTranslatorOwned {
    fn to_header(h: &mut header::HeaderMap, key: &'static str, value: &'static str) {
        h.insert(key, header::HeaderValue::from_static(value));
    }

    /**
     * ai - An AI object implements AIinterface
     * src_lang - Source text language or None (Auto)
     * dest_lang - If None - src lang
     */
    pub fn new(
        ai: Box<dyn AIinterface>,
        mut src_lang: Option<String>,
        dest_lang: Option<String>,
        drop_nonconfident_result: Option<f64>,
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

        let random_start_id = (rand::random::<i64>() % 99999 + 8300000) * 1000;

        let mut headers = header::HeaderMap::new();
        Self::to_header(&mut headers, "Content-Type", "application/json");
        Self::to_header(&mut headers, "Accept", "*/*");
        Self::to_header(&mut headers, "x-app-os-name", "iOS");
        Self::to_header(&mut headers, "x-app-os-version", "16.3.0");
        Self::to_header(&mut headers, "Accept-Language", "en-US,en;q=0.9");
        Self::to_header(&mut headers, "Accept-Encoding", "gzip, deflate, br");
        Self::to_header(&mut headers, "x-app-device", "iPhone13,2");
        Self::to_header(
            &mut headers,
            "User-Agent",
            "DeepL-iOS/2.6.0 iOS 16.3.0 (iPhone13,2)",
        );
        Self::to_header(&mut headers, "x-app-build", "353933");
        Self::to_header(&mut headers, "x-app-version", "2.6");
        Self::to_header(&mut headers, "Connection", "keep-alive");

        Self {
            src_lang,
            dest_lang: dl,
            ai,
            // generate random id
            id: random_start_id,
            drop_nonconfident_result,

            headers,
        }
    }

    pub async fn translate<S, SRC, DEST>(
        &mut self,
        text: S,
        src_lang: Option<SRC>,
        dest_lang: DEST,
        override_drop_nonconfident_result: Option<f64>,
    ) -> Result<String, String>
    where
        S: Into<String>,
        SRC: Into<String>,
        DEST: Into<String>,
    {
        let current_id = self.id.wrapping_add(1);

        let drop_nonconfident_result = match override_drop_nonconfident_result {
            Some(v) => Some(v),
            None => self.drop_nonconfident_result,
        };

        let text = text.into();
        let req = serde_json::json!(
            {
                "jsonrpc": "2.0",
                "method":  "LMT_handle_texts",
                "id":      current_id,
                "params": {
                    "timestamp": get_time_stamp(get_i_count(&text)),
                    "texts": [{
                        "text": text.clone(),
                        "requestAlternatives": 1 // 3
                    }],
                    "splitting": "newlines",
                    "lang": {
                        "source_lang_user_selected": match src_lang {
                            Some(s) => s.into(),
                            None => "auto".to_string(),
                        },
                        "target_lang": dest_lang.into(),
                    },
                    "commonJobParams": {
                        "WasSpoken":    false,
                        "TranscribeAS": "",
                        // RegionalVariant: "en-US",
                    },
                },
            }
        );
        self.id = current_id;

        // serialise json
        let post_str = serde_json::to_string(&req).map_err(|e| e.to_string())?;

        // add space if necessary
        let post_str = if (self.id + 5) % 29 == 0 || (self.id + 3) % 13 == 0 {
            post_str.replace("\"method\":\"", "\"method\" : \"")
        } else {
            post_str.replace("\"method\":\"", "\"method\": \"")
        };

        let client = reqwest::Client::new();
        let resp: Value = client
            .post(DEEPLX_URL)
            .headers(self.headers.clone())
            .body(post_str)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        // Ok resp:
        //  Object {"id": Number(8373055001), "jsonrpc": String("2.0"), "result": Object {"detectedLanguages": Object {}, "lang": String("EN"), "lang_is_confident": Bool(false), "texts": Array [Object {"alternatives": Array [], "text": String("Hi")}]}}
        // Unknown language:
        //  Object {"id": Number(8375246001), "jsonrpc": String("2.0"), "result": Object {"detectedLanguages": Object {"BG": Number(0.130183), "CS": Number(0.000067), "DA": Number(0.000018), "DE": Number(0.000025), "EL": Number(0.000014), "EN": Number(0.000028), "ES": Number(0.00003), "ET": Number(0.000371), "FI": Number(0.000025), "FR": Number(0.000029), "HU": Number(0.000217), "ID": Number(0.000018), "IT": Number(0.000033), "JA": Number(0.000035), "KO": Number(0.000019), "LT": Number(0.000047), "LV": Number(0.000204), "NB": Number(0.000019), "NL": Number(0.000019), "PL": Number(0.000039), "PT": Number(0.000084), "RO": Number(0.000108), "RU": Number(0.160944), "SK": Number(0.000045), "SL": Number(0.00023), "SV": Number(0.000023), "TR": Number(0.000011), "UK": Number(0.165217), "ZH": Number(0.000155), "unsupported": Number(0.541743)}, "lang": String("UK"), "lang_is_confident": Bool(false), "texts": Array [Object {"alternatives": Array [Object {"text": String("Goodbye.")}], "text": String("Bye.")}]}}
        // Error:
        //  Object {"error": Object {"code": Number(1042911), "message": String("Too many requests")}, "jsonrpc": String("2.0")}

        trace!("Translate responce: {}", resp);
        if let Value::Object(result) = &resp {
            if let Some(Value::Object(error)) = result.get("error") {
                if let Value::Number(e) = &error["code"] {
                    if e.eq(&Into::<serde_json::Number>::into(-32600)) {
                        return Err("Invalid target Lang".to_string());
                    } else {
                        return Err(format!(
                            "Failed to translate, error {}: {}",
                            e, &error["message"]
                        ));
                    }
                }
            } else if let Some(Value::Number(id)) = result.get("id") {
                if id.eq(&current_id.into()) {
                    if let Some(Value::Object(result)) = result.get("result") {
                        let is_confident =
                            Some(&Value::Bool(true)) == result.get("lang_is_confident");
                        if !is_confident {
                            if let Some(drop_nonconfident_result) = drop_nonconfident_result {
                                if let Some(Value::Object(detected_langs)) =
                                    result.get("detectedLanguages")
                                {
                                    if let Some(Value::Number(unsupported)) =
                                        detected_langs.get("unsupported")
                                    {
                                        let unsupported_p = unsupported.as_f64().unwrap();
                                        if unsupported_p > drop_nonconfident_result {
                                            return Err(format!("Translate input is not confident ({unsupported_p}), posibly gabrage input '{text}'"));
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(Value::Array(texts)) = result.get("texts") {
                            if let Some(Value::Object(first_res)) = texts.first() {
                                if let Some(Value::String(text)) = first_res.get("text") {
                                    return Ok(text.clone());
                                }
                            }
                        }
                        return Err("Missing text results".to_string());
                    } else {
                        return Err("Missing translate result".to_string());
                    }
                } else {
                    return Err(format!("Invaid response id {}, actual: {}", id, current_id));
                }
            }
        }
        Err(format!("Failed to parse result ({})", resp))
    }
}

#[async_trait]
impl AIinterface for DeepLxTranslatorOwned {
    async fn process(
        &mut self,
        request: Box<dyn AIRequest>,
    ) -> Result<HashMap<AIResponseType, String>, AIError> {
        let r = request.request();
        let req_lang = if let Some(l) = &self.src_lang {
            l.clone()
        } else {
            request.lang()
        };

        // translate input to english
        let translated = self
            .translate(r.clone(), Some(req_lang), "en", None)
            .await
            .map_err(|e| AIError::TranslateError(e))?;
        debug!("{r} ({lang:?}) => {translated}", lang = &self.src_lang);

        // preocess AI request
        let raw_answer = &self
            .ai
            .process(Box::new(TranslatedAIRequest::new(request, translated)))
            .await?[&AIResponseType::RawAnswer];

        let no_digits_answer = crate::num2words::convert_numbers2words(raw_answer.clone());

        // translate answer to user language
        let translated_answer = self
            .translate(
                no_digits_answer.clone(),
                Some("en"),
                self.dest_lang.clone(),
                Some(1.0), // do not drop non-confident results
            )
            .await
            .map_err(|e| AIError::TranslateError(e))?;
        debug!(
            "{raw_answer} => {translated_answer} ({lang})",
            lang = &self.dest_lang
        );

        let res = hashmap! {
            AIResponseType::RawAnswer => raw_answer.clone(),
            AIResponseType::NoDigits => no_digits_answer,
            AIResponseType::Translated => translated_answer,
        };

        Ok(res)
    }

    async fn reset(&mut self) -> Result<(), AIError> {
        self.ai.reset().await
    }
}
