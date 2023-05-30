mod tests {
    use ai_waifu::silerio_tts::SilerioTTS;

    #[tokio::test]
    async fn test_tts() {
        let tts = SilerioTTS::new("http://localhost:8961/say", Some("kseniya".to_string()));

        let res = tts.say("Привет, мир!").await;

        assert!(res.is_ok());
    }
}
