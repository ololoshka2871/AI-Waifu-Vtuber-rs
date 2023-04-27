use cpal::traits::HostTrait;
use rodio::DeviceTrait;

pub fn get_audio_device_by_name(
    host: &cpal::Host,
    name: &str,
    input: bool,
) -> Option<cpal::Device> {
    let devices = if input {
        host.input_devices()
    } else {
        host.output_devices()
    };

    if let Ok(mut d) = devices {
        d.find(|d| {
            if let Ok(n) = d.name() {
                n == name
            } else {
                false
            }
        })
    } else {
        if input {
            host.default_input_device()
        } else {
            host.default_output_device()
        }
    }
}
