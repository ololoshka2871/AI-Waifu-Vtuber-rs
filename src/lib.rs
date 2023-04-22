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

static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn init_python(add_python_path: &Option<Vec<String>>) -> pyo3::PyResult<()> {
    use pyo3::{types, PyErr};

    let python_modules_path = std::path::PathBuf::from(CARGO_MANIFEST_DIR)
        .join("pymodules")
        .to_str()
        .unwrap()
        .to_string();

    pyo3::prepare_freethreaded_python();

    pyo3::Python::with_gil(|py| {
        let syspath: &types::PyList = py.import("sys")?.getattr("path")?.extract()?;
        syspath.insert(0, &python_modules_path)?;

        if let Some(add_python_path) = add_python_path {
            for path in add_python_path {
                syspath.insert(1, path)?;
            }
        }

        // try import all python modules
        let _torch = py.import("torch")?;
        let _numpy = py.import("numpy")?;
        let _librosa = py.import("librosa")?;
        let _wave = py.import("wave")?;

        let sys = py.import("sys")?;
        let version: String = sys.getattr("version")?.extract()?;

        tracing::info!("Python {}", version);
        tracing::debug!("Python import path is: {:?}", syspath);
        Ok(())
    })
    .map_err(|e: PyErr| panic!("Some Python mdules missing check error above:\n{}", e))
}
