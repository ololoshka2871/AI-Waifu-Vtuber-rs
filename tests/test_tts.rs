mod tests {
    use ai_waifu::silerio_tts::SilerioTTS;

    #[tokio::test]
    async fn test_translate_ru() {
        let tts = SilerioTTS::new("http://localhost:8961/say");

        let res = tts.say("Привет, мир!", "kseniya").await;

        assert!(res.is_ok());
    }
}
