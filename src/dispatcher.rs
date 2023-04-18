use std::collections::HashMap;

use async_trait::async_trait;
use serenity::futures::lock::Mutex;

#[derive(Debug, Clone)]
pub enum AIError {
    /// ИИ занят
    Busy,

    /// Ошибка сети
    NetworkError,

    /// Translate error
    TranslateError(String),

    /// Answer error
    AnswerError(String),

    /// Неизвестная ошибка
    UnknownError,
}

pub trait AIRequest: Send {
    /// Возвращает текст запроса
    fn request(&self) -> String;
    /// Возвращает автора запроса
    fn channel(&self) -> String;
}

/// Интерфейс ИИ:
///  - ChatGPT
///  - LLaMA
#[async_trait]
pub trait AIinterface: Sync + Send {
    /// Обработать запрос
    async fn process(&mut self, request: Box<dyn AIRequest>) -> Result<String, AIError>;
}

pub trait AIBuilder {
    fn build(&mut self) -> Box<dyn AIinterface>;
}

pub struct Dispatcher<AIB: AIBuilder> {
    ai_constructor: AIB,
    user_map: HashMap<String, Mutex<Box<dyn AIinterface>>>,
}

impl<AIB: AIBuilder> Dispatcher<AIB> {
    pub fn new(ai_constructor: AIB) -> Self {
        Self {
            ai_constructor,
            user_map: HashMap::new(),
        }
    }

    /// Обработать запрос
    pub async fn try_process_request(
        &mut self,
        request: Box<dyn AIRequest>,
    ) -> Result<String, AIError> {
        let mut channel_ai = self
            .user_map
            .entry(request.channel())
            .or_insert_with(|| Mutex::new(self.ai_constructor.build()))
            .try_lock()
            .ok_or(AIError::Busy)?;

        if request.request().is_empty() {
            Ok("".to_string())
        } else {
            channel_ai.process(request).await
        }
    }
}
