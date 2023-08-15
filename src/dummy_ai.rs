use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use futures_util::Stream;
use maplit::hashmap;

use crate::dispatcher::{AIError, AIRequest, AIResponseType, AIinterface, ResponseChunk};

pub struct DummyAI;

#[async_trait]
impl AIinterface for DummyAI {
    async fn process(
        &mut self,
        request: Box<dyn AIRequest>,
    ) -> Result<HashMap<AIResponseType, String>, AIError> {
        let res = hashmap! {
            AIResponseType::RawAnswer => request.request(),
        };
        Ok(res)
    }

    async fn process_stream(
        &mut self,
        _request: Box<dyn AIRequest>,
    ) -> Result<Box<dyn Stream<Item = ResponseChunk>>, AIError> {
        Err(AIError::ResetErrorEmpty)
    }

    async fn reset(&mut self) -> Result<(), AIError> {
        Ok(())
    }

    async fn save_context(&mut self, _file: PathBuf) -> Result<(), AIError> {
        Ok(())
    }

    fn load_context(&mut self, _file: PathBuf) -> Result<(), AIError> {
        Ok(())
    }
}
