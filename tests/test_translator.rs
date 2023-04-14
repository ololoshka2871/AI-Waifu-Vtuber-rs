mod tests {
    use ai_waifu::{dispatcher::*, dummy_ai::DummyAI, google_translator::GoogleTranslator};

    struct TestRequest {
        request: String,
        author: String,
    }

    impl AIRequest for TestRequest {
        fn request(&self) -> String {
            self.request.clone()
        }

        fn author(&self) -> String {
            self.author.clone()
        }
    }

    #[tokio::test]
    async fn test_translate_ru() {
        let ai = Box::new(DummyAI);
        let en_ai = GoogleTranslator::new(ai, Some("ru".to_string()), None).await;
    
        let mut dispatcher = Dispatcher::new(Box::new(en_ai));

        let req = TestRequest {
            request: "Мама мыла раму.".to_string(),
            author: "Master".to_string(),
        };

        assert!(dispatcher.try_process_request(Box::new(req)).await.is_ok());
    }
}
