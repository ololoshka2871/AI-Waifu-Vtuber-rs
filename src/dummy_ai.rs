use async_trait::async_trait;

use crate::{dispatcher::{AIinterface, AIError, AIRequest}};


pub struct DummyAI;


#[async_trait]
impl AIinterface for DummyAI {
    async fn process(&mut self, request: Box<dyn AIRequest>) -> Result<String, AIError> {
        Ok(request.request())
    }

    async fn reset(&mut self) -> Result<(), AIError> {
        Ok(())
    }
}