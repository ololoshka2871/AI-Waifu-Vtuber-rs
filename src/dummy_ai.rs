use std::collections::HashMap;

use async_trait::async_trait;
use maplit::hashmap;

use crate::{dispatcher::{AIinterface, AIError, AIRequest, AIResponseType}};


pub struct DummyAI;


#[async_trait]
impl AIinterface for DummyAI {
    async fn process(&mut self, request: Box<dyn AIRequest>) -> Result<HashMap<AIResponseType, String>, AIError> {
        let res = hashmap! {
            AIResponseType::RawAnswer => request.request(),
        };
        Ok(res)
    }

    async fn reset(&mut self) -> Result<(), AIError> {
        Ok(())
    }
}