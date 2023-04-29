use std::io::Cursor;

use dasp::sample::ToSample;
use hound::{Error, SampleFormat, WavSpec, WavWriter};

pub fn voice_data_to_wav_buf_gain<T>(
    voice_data: Vec<T>,
    channels: u16,
    sample_rate: u32,
) -> Result<Vec<u8>, Error>
where
    T: cpal::Sample + ToSample<f32> + ToSample<i16>,
{
    let mut result = Vec::new();
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

    voice_data.into_iter().for_each(|x| {
        writer.write_sample(cpal::Sample::to_sample::<i16>(x)).unwrap();
    });

    writer.finalize()?;

    Ok(result)
}
