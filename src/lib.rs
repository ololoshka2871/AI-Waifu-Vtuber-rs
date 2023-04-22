pub mod ai_translated_request;
pub mod audio_dev;
pub mod audio_halpers;
pub mod chatgpt;
pub mod chatgpt_en_deeplx_builder;
pub mod config;
pub mod deeplx_translate;
pub mod deeplx_translate_owned;
pub mod dispatcher;
pub mod dummy_ai;
pub mod google_translator;
pub mod silerio_tts;
pub mod silerio_tts_local;
pub mod urukhan_voice_recognize;

pub fn check_python() -> pyo3::PyResult<()> {
    use pyo3::{types, PyErr};

    let path = std::path::Path::new("./pymodules/");

    pyo3::prepare_freethreaded_python();

    pyo3::Python::with_gil(|py| {
        // try import all python modules
        let _torch = py.import("torch")?;
        let _numpy = py.import("numpy")?;
        let _librosa = py.import("librosa")?;
        let _wave = py.import("wave")?;

        let syspath: &types::PyList = py.import("sys")?.getattr("path")?.extract()?;
        syspath.insert(0, &path)?;
        tracing::info!("Python import path is: {:?}", syspath);

        let sys = py.import("sys")?;
        let version: String = sys.getattr("version")?.extract()?;

        tracing::info!("Python {}", version);
        Ok(())
    }).map_err(|e: PyErr| {
        panic!("Some Python mdules missing check error above:\n{}", e)
    })
}
