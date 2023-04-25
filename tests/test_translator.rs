mod tests {
    use ai_waifu::{dispatcher::*, dummy_ai::DummyAI, google_translator::GoogleTranslator};

    struct TestRequest {
        request: String,
        channel: String,
    }

    impl AIRequest for TestRequest {
        fn request(&self) -> String {
            self.request.clone()
        }

        fn channel(&self) -> String {
            self.channel.clone()
        }
    }

    struct DummuENAIConstrictor;

    impl AIBuilder for DummuENAIConstrictor {
        fn build(&mut self) -> Box<dyn AIinterface> {
            let ai = Box::new(DummyAI);
            let en_ai = GoogleTranslator::new(ai, Some("ru".to_string()), None);
            Box::new(en_ai)
        }
    }

    #[tokio::test]
    async fn test_translate_ru() {
        let mut dispatcher = AIDispatcher::new(DummuENAIConstrictor{});

        let req = TestRequest {
            request: "Мама мыла раму.".to_string(),
            channel: "Master".to_string(),
        };

        assert!(dispatcher.try_process_request(Box::new(req)).await.is_ok());
    }
}
