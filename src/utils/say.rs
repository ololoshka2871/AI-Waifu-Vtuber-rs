use cpal::platform::Device;
use rodio::{OutputStream, Sink, Decoder};

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

pub fn say<R, F>(audio_out: &Option<Device>, sound_data: R, f: F)
where
    R: std::io::Read + std::io::Seek + Send + Sync + 'static,
    F: FnOnce(),
{
    if let Some(ao) = audio_out {
        if let Ok((_stream, stream_handle)) = OutputStream::try_from_device(ao) {
            // _stream mast exists while stream_handle is used
            match Sink::try_new(&stream_handle) {
                Ok(sink) => match Decoder::new_wav(sound_data) {
                    Ok(decoder) => {
                        sink.append(decoder);
                        sink.sleep_until_end();
                        f();
                    }
                    Err(e) => {
                        error!("Decode wav error: {:?}", e);
                    }
                },
                Err(e) => {
                    error!("Sink error: {:?}", e);
                }
            }
        } else {
            error!("Audio output error");
        }
    }
}