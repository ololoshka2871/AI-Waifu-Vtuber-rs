use cpal::{traits::StreamTrait, Device, Stream};

use hound::WavSpec;

use dasp::Frame;

use reqwest::Url;
use rodio::DeviceTrait;
use tokio::{
    runtime::Handle,
    sync::mpsc::{Receiver, Sender},
};

use noise_gate::NoiseGate;

use tracing::{debug, error};

use crate::whisper_voice_recognize::OpenAIWhisperVoice2Txt;

/// A sink which sends audiodata to spech recognition.
pub struct Sink {
    voice2txt_url: Url,
    minimal_fragment_length: f32,
    maximal_fragment_length: f32,
    spec: WavSpec,
    audio_req_tx: Sender<(String, String)>,

    current_buffer: Option<Vec<f32>>,
    tokio_handle: Handle,
}

impl Sink {
    pub fn new(
        voice2txt_url: Url,
        minimal_fragment_length: f32,
        maximal_fragment_length: f32,
        audio_req_tx: Sender<(String, String)>,
        spec: WavSpec,
        tokio_handle: Handle,
    ) -> Self {
        Sink {
            voice2txt_url: voice2txt_url,
            minimal_fragment_length,
            maximal_fragment_length,
            audio_req_tx,
            spec,
            current_buffer: None,
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
{
    fn record(&mut self, frame: F) {
        let current_fragment = self.get_fragment_buffer();
        current_fragment.extend(frame.channels());
    }

    fn end_of_transmission(&mut self) {
        if let Some(buf) = std::mem::replace(&mut self.current_buffer, None) {
            // ready
            let channels = self.spec.channels;
            let sample_rate = self.spec.sample_rate;

            let length = buf.len() as f32 / channels as f32 / sample_rate as f32;

            if length < self.minimal_fragment_length {
                debug!(
                    "Voice fragment too short ({length}s < {min}s), skipping...",
                    min = self.minimal_fragment_length,
                    length = length
                );
                return;
            } else if length > self.maximal_fragment_length {
                debug!(
                    "Voice fragment too long ({length}s > {max}s), skipping...",
                    max = self.maximal_fragment_length,
                    length = length
                );
                return;
            } else {
                debug!("Got voice fragment length: {}s", length);
            }

            let voice2txt_url = self.voice2txt_url.clone();
            let audio_req_tx = self.audio_req_tx.clone();

            self.tokio_handle.spawn(async move {
                match super::audio_halpers::voice_data_to_wav_buf_gain(buf, channels, sample_rate) {
                    Ok(wav_data) => {
                        let voice2txt = OpenAIWhisperVoice2Txt::new(voice2txt_url);
                        match voice2txt.recognize(wav_data).await {
                            Ok(text) => {
                                if text.0.len() > 1 {
                                    if let Err(e) = audio_req_tx.send(text).await {
                                        error!("Failed to send voice request: {:?}", e)
                                    }
                                } else {
                                    debug!("No words recognized...");
                                }
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
    audio_req_tx: Sender<(String, String)>,
    noise_gate: f32,
    release_time: f32,
    voice2txt_url: Url,
    minimal_fragment_length: f32,
    maximal_fragment_length: f32,
    tokio_handle: Handle,
) -> Result<Stream, String> {
    let config = ain.default_input_config().map_err(|e| format!("{e}"))?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    let release_time = (sample_rate as f32 * release_time).round();

    let mut sink = Sink::new(
        voice2txt_url,
        minimal_fragment_length,
        maximal_fragment_length,
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
    let mut dagc = dagc::MonoAgc::new(0.001, 0.0001).expect("unreachable");

    let stream = ain
        .build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // mono signal
                let mut frames = data
                    .chunks(channels as usize)
                    .map(|chank| chank[0])
                    .collect::<Vec<_>>();
                dagc.process(&mut frames);
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

pub async fn get_voice_request<T>(rx_channel: &mut Receiver<T>) -> T {
    loop {
        if let Some(s) = rx_channel.recv().await {
            return s;
        }
    }
}
