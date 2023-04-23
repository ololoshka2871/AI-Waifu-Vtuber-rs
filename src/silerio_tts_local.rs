use std::io::Cursor;

use bytes::Bytes;

use tch::{TchError, CModule};

#[allow(unused_imports)]
use tracing::{debug, info};

pub struct SilerioTTSLocal(CModule);

impl SilerioTTSLocal {
    pub fn new<T: AsRef<std::path::Path>>(model_file: T) -> Result<Self, TchError> {
        let model_file_str = model_file.as_ref().to_str().unwrap();
        info!("Loading model file: {}", &model_file_str);

        let module = CModule::load(&model_file)?;

        Ok(Self(module))
    }

    pub async fn say<S, SP>(&self, text: S, voice_id: Option<SP>) -> Result<Cursor<Bytes>, String>
    where
        S: Into<String>,
        SP: Into<String>,
    {
        let module = &self.0;

        //Ok(Cursor::new(res.into()))
        Ok(Cursor::new(Bytes::new()))
    }
}
