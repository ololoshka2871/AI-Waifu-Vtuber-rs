use async_trait::async_trait;
use serenity::{futures::lock::Mutex};

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
    fn author(&self) -> String;
}

/// Интерфейс ИИ:
///  - ChatGPT
///  - LLaMA
#[async_trait]
pub trait AIinterface: Sync + Send {
    /// Обработать запрос
    async fn process(&mut self, request: Box<dyn AIRequest>) -> Result<String, AIError>;
}

pub struct Dispatcher {
    processing_ai: Mutex<Box<dyn AIinterface>>,
}

impl Dispatcher
{
    pub fn new(ai: Box<dyn AIinterface>) -> Self {
        Self {
            processing_ai: Mutex::new(ai),
        }
    }

    /// Обработать запрос
    pub async fn try_process_request(&mut self, request: Box<dyn AIRequest>) -> Result<String, AIError> {
        if let Some(mut ai) = self.processing_ai.try_lock() {
            if request.request().is_empty() {
                Ok("".to_string())
            } else {
                ai.process(request).await
            }
        } else {
            Err(AIError::Busy)
        }
    }
}
