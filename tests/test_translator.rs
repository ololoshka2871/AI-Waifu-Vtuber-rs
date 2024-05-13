mod tests {
    use ai_waifu::{
        deeplx_translate_owned::DeepLxTranslatorOwned, dispatcher::*, dummy_ai::DummyAI,
        utils::test_request::TestRequest,
    };

    struct DummuENAIConstrictor;

    impl AIBuilder for DummuENAIConstrictor {
        fn build(&mut self) -> Box<dyn AIinterface> {
            let ai = Box::new(DummyAI);
            let en_ai = DeepLxTranslatorOwned::new(ai, Some("ru".to_string()), None, None);
            Box::new(en_ai)
        }
    }

    #[tokio::test]
    async fn test_translate_ru() {
        let mut dispatcher = AIDispatcher::new(DummuENAIConstrictor {}, None);

        let req = TestRequest {
            request: "Мама мыла раму.".to_string(),
            channel: "Master".to_string(),
        };

        assert!(dispatcher.try_process_request(Box::new(req)).await.is_ok());
    }
}
