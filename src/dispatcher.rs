use std::{collections::HashMap, path::PathBuf};

use std::collections::hash_map::Entry;

use async_trait::async_trait;
use maplit::hashmap;
use serenity::futures::lock::Mutex;
use tracing::{error, info};

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

    /// Ошибка контекста
    ContextError,
}

pub trait AIRequest: Send {
    /// Возвращает текст запроса
    fn request(&self) -> String;
    /// Возвращает автора запроса
    fn channel(&self) -> String;
    /// Возвращает язык запроса
    fn lang(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AIResponseType {
    RawAnswer,
    NoDigits,
    Translated,
}

/// Интерфейс ИИ:
///  - ChatGPT
///  - LLaMA
#[async_trait]
pub trait AIinterface: Sync + Send {
    /// Обработать запрос
    async fn process(
        &mut self,
        request: Box<dyn AIRequest>,
    ) -> Result<HashMap<AIResponseType, String>, AIError>;

    /// Сбросить состояние ИИ
    async fn reset(&mut self) -> Result<(), AIError>;

    /// Сохранить контекст
    async fn save_context(&mut self, file: PathBuf) -> Result<(), AIError>;

    /// Загрузить контекст
    fn load_context(&mut self, file: PathBuf) -> Result<(), AIError>;
}

pub trait AIBuilder: Send + Sync {
    fn build(&mut self) -> Box<dyn AIinterface>;
}

#[async_trait]
pub trait Dispatcher: Send + Sync {
    /// Обработать запрос
    async fn try_process_request(
        &mut self,
        request: Box<dyn AIRequest>,
    ) -> Result<HashMap<AIResponseType, String>, AIError>;

    /// Сбросить состояние ИИ
    async fn reset(&mut self, channel: String) -> Result<(), AIError>;
}

pub struct AIDispatcher<AIB: AIBuilder> {
    ai_constructor: AIB,
    user_map: HashMap<String, Mutex<Box<dyn AIinterface>>>,
    context_path: Option<PathBuf>,
}

impl<AIB: AIBuilder> AIDispatcher<AIB> {
    pub fn new(ai_constructor: AIB, context_path: Option<PathBuf>) -> Self {
        Self {
            ai_constructor,
            user_map: HashMap::new(),
            context_path,
        }
    }

    fn context_path(&self, channel: String) -> Option<PathBuf> {
        if let Some(context_path) = &self.context_path {
            Some(context_path.clone().join(channel))
        } else {
            None
        }
    }

    fn get_channel(&mut self, channel: String) -> &Mutex<Box<dyn AIinterface>> {
        let context_filename = self.context_path(channel.clone());
        let entry = self.user_map.entry(channel.clone());
        match entry {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let mut ai_context = self.ai_constructor.build();

                if let Some(filename) = context_filename {
                    if filename.exists() {
                        info!("Loading context from {:?}", filename);
                        if let Err(_) = ai_context.load_context(filename) {
                            error!("Failed to load context, skipping...");
                        }
                    }
                }

                entry.insert(Mutex::new(ai_context))
            }
        }
    }
}

#[async_trait]
impl<AIB: AIBuilder> Dispatcher for AIDispatcher<AIB> {
    /// Обработать запрос
    async fn try_process_request(
        &mut self,
        request: Box<dyn AIRequest>,
    ) -> Result<HashMap<AIResponseType, String>, AIError> {
        let channel_name = request.channel();
        let context_path = self.context_path(channel_name.clone());
        let channel = self.get_channel(channel_name.clone());

        let mut channel_ai = channel.try_lock().ok_or(AIError::Busy)?;

        if request.request().is_empty() {
            let res = hashmap! {
                AIResponseType::RawAnswer => "".to_string(),
            };
            Ok(res)
        } else {
            let result = channel_ai.process(request).await;

            match result {
                Ok(result) => {
                    if let Some(filename) = context_path {
                        if let Err(_) = channel_ai.save_context(filename).await {
                            error!("Failed to save context, skipping...");
                        }
                    }
                    Ok(result)
                }
                Err(e) => Err(e),
            }
        }
    }

    /// Сбросить состояние ИИ
    async fn reset(&mut self, channel: String) -> Result<(), AIError> {
        let context_path = self.context_path(channel.clone());
        let ch = self.user_map.entry(channel);
        let reset_result = if let Entry::Occupied(mut entry) = ch {
            let mut channel_ai = entry.get_mut().try_lock().ok_or(AIError::Busy)?;
            channel_ai.reset().await
        } else {
            Ok(())
        };

        if let Some(filename) = context_path {
            // delete context file
            if filename.exists() {
                if let Err(e) = std::fs::remove_file(filename) {
                    error!("Failed to remove context file ({e}), skipping...");
                }
            }
        }

        reset_result
    }
}
