use async_trait::async_trait;

use crate::{dispatcher::{AIinterface, AIError, AIRequest}};


pub struct ChatGPT {
    //model: GPT2,
    //tokenizer: GPT2Tokenizer,
    //device: Device,
}

impl ChatGPT {
    pub fn new() -> Self {
        Self {
            //model: GPT2::new(gpt2::GPT2Config::from_file("gpt2/config.json"), Default::default()),
            //tokenizer: GPT2Tokenizer::from_file("gpt2/vocab.json", true),
            //device: Device::cuda_if_available(),
        }
    }
}

#[async_trait]
impl AIinterface for ChatGPT {
    async fn process(&self, _request: Box<dyn AIRequest>) -> Result<String, AIError> {
        //let input_ids = self.tokenizer.encode(request.request(), true);
        //let input_ids = Tensor::of_slice(&input_ids).to_kind(Kind::Int64).to_device(self.device);
        //let output = self.model.generate(Some(
        //    [input_ids.size()[0], 1].as_ref(),
        //), Some(100),
        //    Some(&input_ids),
        //    None,
        //    None,
        //    None,
        //    None,
        //    None,
        Err(AIError::UnknownError)
    }
}