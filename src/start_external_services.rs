use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Command, Stdio},
    str::FromStr,
};

use crate::config::Config;

#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

fn start_service<S: AsRef<OsStr>>(script: &str, args: &[S]) -> std::io::Result<Child> {
    let mut binding = Command::new("python");

    // https://stackoverflow.com/a/64569340
    if !cfg!(debug_assertions) {
        binding.stdout(Stdio::null());
        binding.stderr(Stdio::null());
    }

    let command = binding.arg(
        PathBuf::from_str(crate::CARGO_MANIFEST_DIR)
            .unwrap()
            .join("py_services")
            .join(script),
    );

    let command = args
        .into_iter()
        .fold(command, |command, arg| command.arg(arg));

    command.spawn()
}

pub fn start_external_services(config: &Config) -> Result<Vec<Child>, String> {
    let mut external_services = vec![];

    let selirio_host = config.tts_service_url.host_str().unwrap_or("localhost");
    let models_store_path = &config.models_store_path;

    if selirio_host == "localhost" {
        let port = config.tts_service_url.port().unwrap_or(8961);

        external_services.push(
            start_service(
                "Selerio-TTS-server/main.py",
                &[
                    "-m",
                    models_store_path.as_os_str().to_str().unwrap(),
                    "-p",
                    port.to_string().as_str(),
                    config.voice_language.as_str(),
                    config.voice_model.as_str(),
                ],
            )
            .map_err(|e| format!("Failed to start Selerio-TTS-server: {e:?}"))?,
        );
        info!("Selerio-TTS-server started");
    } else {
        warn!("Using external Selerio-TTS-server {}", selirio_host);
    }

    let stt_host = config.voice2txt_url.host_str().unwrap_or("localhost");

    if stt_host == "localhost" {
        let port = config.voice2txt_url.port().unwrap_or(3154);

        external_services.push(
            start_service(
                "Voice-2-txt-UrukHan/main.py",
                &[
                    "-m",
                    models_store_path.as_os_str().to_str().unwrap(),
                    "-p",
                    port.to_string().as_str(),
                ],
            )
            .map_err(|e| format!("Failed to start Voice-2-txt-UrukHan server: {e:?}"))?,
        );

        info!("Voice-2-txt-UrukHan server started");
    } else {
        warn!("Using external Voice-2-txt-UrukHan server {}", stt_host);
    }

    Ok(external_services)
}
