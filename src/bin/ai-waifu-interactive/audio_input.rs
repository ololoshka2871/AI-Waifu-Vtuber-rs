use std::{fs::File, io::BufWriter};

use cpal::{traits::StreamTrait, Device, Stream};

use hound::{WavSpec, WavWriter};

use dasp::Frame;

use rodio::DeviceTrait;
use tokio::sync::mpsc::{Receiver, Sender};

use noise_gate::NoiseGate;

use tracing::{warn, error, info, debug};

/// A sink which sends audiodata to spech recognition.
pub struct Sink {
    spec: WavSpec,
    audio_req_tx: Sender<String>,
    is_recording: bool,
}

impl Sink {
    pub fn new(audio_req_tx: Sender<String>, spec: WavSpec) -> Self {
        Sink {
            audio_req_tx,
            spec,
            is_recording: false,
        }
    }
}

impl<F> noise_gate::Sink<F> for Sink
where
    F: Frame,
    F::Sample: hound::Sample,
{
    fn record(&mut self, _frame: F) {
        if !self.is_recording {
            self.is_recording = true;
            debug!("start_of_transmission");
        }
    }

    fn end_of_transmission(&mut self) {
        debug!("end_of_transmission");
        self.is_recording = false;
    }
}

pub fn spawn_audio_input(
    ain: Device,
    audio_req_tx: Sender<String>,
    noise_gate: f32,
    release_time: f32,
) -> Result<Stream, String> {
    let config = ain.default_input_config().map_err(|e| format!("{e}"))?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    let release_time = (sample_rate as f32 * release_time).round();

    let mut sink = Sink::new(
        audio_req_tx,
        WavSpec {
            channels: channels as u16,
            sample_rate: sample_rate as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    );
    let mut noise_gate = NoiseGate::new(noise_gate, release_time as usize);

    let stream = ain
        .build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // mono signal
                let mut frames = data
                    .chunks(channels as usize)
                    .map(|chank| [chank[0]])
                    .collect::<Vec<_>>();
                noise_gate.process_frames(&mut frames, &mut sink);
            },
            |err| {
                error!("An error occurred on the input stream: {}", err);
            },
            None,
        )
        .map_err(|e| format!("{e}"))?;

    stream.play().map_err(|e| format!("{e}"))?;

    Ok(stream)
}

pub async fn get_voice_request<T>(rx_channel: &mut Receiver<T>) -> String {
    loop {
        if let Some(_s) = rx_channel.recv().await {
            return "".to_string();
        }
    }
}
