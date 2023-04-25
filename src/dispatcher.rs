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

pub trait AIBuilder: Send + Sync {
    fn build(&mut self) -> Box<dyn AIinterface>;
}

#[async_trait]
pub trait Dispatcher: Send + Sync {
    /// Обработать запрос
    async fn try_process_request(&mut self, request: Box<dyn AIRequest>)
        -> Result<String, AIError>;
}

pub struct AIDispatcher<AIB: AIBuilder> {
    ai_constructor: AIB,
    user_map: HashMap<String, Mutex<Box<dyn AIinterface>>>,
}

impl<AIB: AIBuilder> AIDispatcher<AIB> {
    pub fn new(ai_constructor: AIB) -> Self {
        Self {
            ai_constructor,
            user_map: HashMap::new(),
        }
    }

    fn get_channel(&mut self, channel: String) -> &Mutex<Box<dyn AIinterface>> {
        self.user_map
            .entry(channel)
            .or_insert_with(|| Mutex::new(self.ai_constructor.build()))
    }
}

#[async_trait]
impl<AIB: AIBuilder> Dispatcher for AIDispatcher<AIB> {
    /// Обработать запрос
    async fn try_process_request(
        &mut self,
        request: Box<dyn AIRequest>,
    ) -> Result<String, AIError> {
        let user = self.get_channel(request.channel());

        let mut channel_ai = user.try_lock().ok_or(AIError::Busy)?;

        if request.request().is_empty() {
            Ok("".to_string())
        } else {
            channel_ai.process(request).await
        }
    }
}
