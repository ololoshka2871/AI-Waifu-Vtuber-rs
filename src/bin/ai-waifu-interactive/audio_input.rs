use std::{
    fs::File,
    io::{BufWriter, Cursor},
};

use ai_waifu::urukhan_voice_recognize::UrukHanVoice2Txt;
use cpal::{traits::StreamTrait, Device, Sample, Stream};

use hound::{WavSpec, WavWriter};

use dasp::Frame;

use reqwest::Url;
use rodio::{buffer, DeviceTrait};
use tokio::{
    runtime::Handle,
    sync::mpsc::{Receiver, Sender},
};

use noise_gate::NoiseGate;

use tracing::{debug, error, info, warn};

//struct WaveBuffer(Vec<u8>);
//
//impl WaveBuffer {
//    fn new() -> Self {
//        WaveBuffer(Vec::new())
//    }
//
//    fn make_coursor(&mut self) -> Cursor<&mut [u8]> {
//        Cursor::new(&mut self.0)
//    }
//}

/// A sink which sends audiodata to spech recognition.
pub struct Sink {
    voice2txt_url: Url,
    spec: WavSpec,
    audio_req_tx: Sender<String>,

    current_buffer: Option<Vec<f32>>,
    fragment_wrier: Option<WavWriter<Cursor<Vec<u8>>>>,
    tokio_handle: Handle,
}

impl Sink {
    pub fn new(
        voice2txt_url: Url,
        audio_req_tx: Sender<String>,
        spec: WavSpec,
        tokio_handle: Handle,
    ) -> Self {
        Sink {
            voice2txt_url: voice2txt_url,
            audio_req_tx,
            spec,
            current_buffer: None,
            fragment_wrier: None,
            tokio_handle,
        }
    }

    fn get_fragment_buffer(&mut self) -> &mut Vec<f32> {
        if self.current_buffer.is_none() {
            self.current_buffer = Some(Vec::new());
        }

        self.current_buffer.as_mut().unwrap()
    }
}

impl<F> noise_gate::Sink<F> for Sink
where
    F: Frame<Sample = f32>,
    //F::Sample: f32,
{
    fn record(&mut self, frame: F) {
        let current_fragment = self.get_fragment_buffer();
        current_fragment.extend(frame.channels());
    }

    fn end_of_transmission(&mut self) {
        if let Some(buf) = self.current_buffer.take() {
            // ready
            let voice2txt_url = self.voice2txt_url.clone();
            let channels = self.spec.channels;
            let sample_rate = self.spec.sample_rate;

            self.tokio_handle.spawn(async move {
                match ai_waifu::audio_halpers::voice_data_to_wav_buf(buf, channels, sample_rate) {
                    Ok(wav_data) => {
                        let voice2txt = UrukHanVoice2Txt::new(voice2txt_url);
                        match voice2txt.recognize(wav_data).await {
                            Ok(text) => {
                                debug!("Said: {}", text);
                            }
                            Err(e) => {
                                error!("Failed to convert voice to text: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to encode voice data to wav: {:?}", e);
                    }
                }
            });
        }
    }
}

pub fn spawn_audio_input(
    ain: Device,
    audio_req_tx: Sender<String>,
    noise_gate: f32,
    release_time: f32,
    voice2txt_url: Url,
    tokio_handle: Handle,
) -> Result<Stream, String> {
    let config = ain.default_input_config().map_err(|e| format!("{e}"))?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    let release_time = (sample_rate as f32 * release_time).round();

    let mut sink = Sink::new(
        voice2txt_url,
        audio_req_tx,
        WavSpec {
            channels: 1,
            sample_rate: sample_rate as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
        tokio_handle,
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
