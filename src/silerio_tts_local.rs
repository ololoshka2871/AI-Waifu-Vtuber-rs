use std::{io::Cursor, path::PathBuf};

use bytes::Bytes;
use pyo3::{types::PyModule, Py, PyAny, PyErr, PyObject, Python};

use tracing::{info, debug};

pub struct SilerioTTSLocal(Py<PyAny>);

impl SilerioTTSLocal {
    pub fn new() -> Result<Self, String> {
        let language = "ru";
        let model = "ru_v3";

        let model_file = PathBuf::from(crate::CARGO_MANIFEST_DIR)
            .join("models")
            .join(format!("{language}_{model}.pt.torchscript.pt"))
            .to_str()
            .unwrap()
            .to_string();

        info!("Loading model file: {}", &model_file);

        let m = tch::jit::CModule::load(&model_file).map_err(|e| e.to_string())?;
        debug!("{:?}", m);

        let (no_null, obj) = Python::with_gil(|py| {
            let tts = PyModule::import(py, "TTS")?;

            let silero_tts_class = tts.getattr("SileroTTS")?;
            let tts_object = silero_tts_class.call1((language, model, model_file))?;

            let obj: PyObject = tts_object.into();

            Ok((!obj.is_none(py), obj))
        })
        .map_err(|e: PyErr| e.to_string())?;

        if no_null {
            Ok(Self(obj))
        } else {
            Err("Error creating TTS object".to_string())
        }
    }

    pub async fn say<S, SP>(&self, text: S, voice_id: Option<SP>) -> Result<Cursor<Bytes>, String>
    where
        S: Into<String>,
        SP: Into<String>,
    {
        let obj = &self.0;
        let res = Python::with_gil(move |py| {
            let voice = voice_id.map(|v| v.into());
            let torch_args = obj.call_method1(py, "prepare_torch_data", (text.into(), voice))?;
            //let wav_data = call_res.extract::<PyObject>(py)?;

            let device = match torch_args.getattr(py, "device")?.extract(py)?  {
                "cpu" => tch::Device::Cpu,
                "cuda" => tch::Device::Cuda(0),
                _ => tch::Device::Cpu,
            };

            Ok((torch_args, device))
        })
        .map_err(|e: PyErr| format!("Error in prepare_torch_args: {}", e))?;

        //Ok(Cursor::new(res.into()))
        Ok(Cursor::new(Bytes::new()))
    }
}
