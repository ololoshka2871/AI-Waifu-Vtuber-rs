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

    /// Ошибка сброса, нечего очищать
    ResetErrorEmpty,
}

pub trait AIRequest: Send {
    /// Возвращает текст запроса
    fn request(&self) -> String;
    /// Возвращает автора запроса
    fn channel(&self) -> String;
    /// Возвращает язык запроса
    fn lang(&self) -> String;
}

/// Интерфейс ИИ:
///  - ChatGPT
///  - LLaMA
#[async_trait]
pub trait AIinterface: Sync + Send {
    /// Обработать запрос
    async fn process(&mut self, request: Box<dyn AIRequest>) -> Result<String, AIError>;

    /// Сбросить состояние ИИ
    async fn reset(&mut self) -> Result<(), AIError>;
}

pub trait AIBuilder: Send + Sync {
    fn build(&mut self) -> Box<dyn AIinterface>;
}

#[async_trait]
pub trait Dispatcher: Send + Sync {
    /// Обработать запрос
    async fn try_process_request(&mut self, request: Box<dyn AIRequest>)
        -> Result<String, AIError>;

    /// Сбросить состояние ИИ
    async fn reset(&mut self, channel: String) -> Result<(), AIError>;
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
        let channel = self.get_channel(request.channel());

        let mut channel_ai = channel.try_lock().ok_or(AIError::Busy)?;

        if request.request().is_empty() {
            Ok("".to_string())
        } else {
            channel_ai.process(request).await
        }
    }

    /// Сбросить состояние ИИ
    async fn reset(&mut self, channel: String) -> Result<(), AIError> {
        let ch = self.user_map.entry(channel);
        if let std::collections::hash_map::Entry::Occupied(mut entry) = ch {
            let mut channel_ai = entry.get_mut().try_lock().ok_or(AIError::Busy)?;
            channel_ai.reset().await
        } else {
            Err(AIError::ResetErrorEmpty)
        }
    }
}
