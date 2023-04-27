use clap::Parser;
use cpal::traits::HostTrait;
use rodio::DeviceTrait;

use tracing::info;

use ai_waifu::{
    utils::audio_dev::get_audio_device_by_name,
    config::Config,
    utils::audio_input::{get_voice_request, spawn_audio_input},
};

/// TTS testing tool
#[derive(Parser)]
#[allow(non_snake_case)]
struct Cli {
    /// List audio devices
    #[clap(short, long, default_value_t = false)]
    list_audio_devices: bool,

    /// Audio input device name
    #[clap(short, long)]
    In: Option<String>,

    /// Audio noise_gate, 0.0 - 1.0
    #[clap(short, long, default_value_t = 0.1)]
    noise_gate: f32,
    /// Audio release_time, s
    #[clap(short, long, default_value_t = 1.0)]
    release_time: f32,
}

/// print all available devices
fn display_audio_devices(host: &cpal::Host) {
    for device in host.input_devices().unwrap() {
        if let Ok(name) = device.name() {
            if let Ok(ic) = device.supported_input_configs() {
                if ic.count() > 0 {
                    println!("Audio In: {}", &name);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::load();

    let args = Cli::parse();

    let ht = cpal::default_host();

    if args.list_audio_devices {
        display_audio_devices(&ht);
        return;
    }

    let audio_in = if let Some(in_d) = args.In {
        get_audio_device_by_name(&ht, &in_d, true)
    } else {
        ht.default_input_device()
    };

    let (mut audio_request_channel, _stream) = if let Some(audio_in) = audio_in {
        info!(
            "Using audio input device: {}",
            audio_in.name().unwrap_or("unknown".to_string())
        );

        let (audio_req_tx, audio_req_rx) = tokio::sync::mpsc::channel(1);
        let stream = spawn_audio_input(
            audio_in,
            audio_req_tx,
            args.noise_gate,
            args.release_time,
            config.voice2txt_url,
            config.minimal_audio_fragment_length,
            config.maximal_audio_fragment_length,
            tokio::runtime::Handle::current(),
        )
        .expect("Failed to init audio input");

        (audio_req_rx, stream)
    } else {
        panic!("No audio input device found, only text input will be available!");
    };

    info!("Say something... (Ctrl-C to exit)");

    // sleep for 1 scond
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    loop {
        let req = get_voice_request(&mut audio_request_channel).await;
        info!("Lang {}: {}", req.1, req.0);
    }
}
