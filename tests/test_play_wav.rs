mod tests {
    use std::io::BufReader;

    use rodio::OutputStream;

    use ai_waifu::audio_dev::get_audio_device_by_name;

    #[tokio::test]
    async fn test_play_wav() {
        let ht = cpal::default_host();

        let ao = get_audio_device_by_name(
            &ht,
            "Line 1 (Virtual Audio Cable)",
            false,
        )
        .expect("No audio output device found!");

        let file = std::fs::File::open("assets/test.wav").unwrap();

        let (_stream, stream_handle) = OutputStream::try_from_device(&ao).unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        let decoder = rodio::decoder::Decoder::new_wav(BufReader::new(file)).unwrap();
        sink.append(decoder);

        sink.sleep_until_end();
    }
}
