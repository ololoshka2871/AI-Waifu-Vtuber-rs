use std::io::Cursor;

use dasp::sample::ToSample;
use hound::{Error, SampleFormat, WavSpec, WavWriter};

pub fn voice_data_to_wav_buf<T>(
    voice_data: Vec<T>,
    channels: u16,
    sample_rate: u32,
) -> Result<Vec<u8>, Error>
where
    T: cpal::Sample,
    T: ToSample<i16>,
{
    let mut result = Vec::with_capacity(voice_data.len() * 2);
    let coursor = Cursor::new(&mut result);

    let mut writer = WavWriter::new(
        coursor,
        WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        },
    )?;

    for sample in voice_data {
        writer.write_sample(sample.to_sample::<i16>())?;
    }

    writer.finalize()?;

    Ok(result)
}
