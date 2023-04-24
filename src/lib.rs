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
pub mod urukhan_voice_recognize;
pub mod start_external_services;

static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
