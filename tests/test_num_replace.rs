mod tests {
    #[test]
    fn test_num_replace() {
        let text = ai_waifu::num2words::convert_numbers2words("I have 42 apples".to_string());
        assert_eq!(text, "I have forty-two apples");
    }

    #[test]
    fn test_num_replace_precision() {
        let text = ai_waifu::num2words::convert_numbers2words("I have 42.53 apples".to_string());
        assert_eq!(text, "I have about forty-three apples");
    }

    #[test]
    fn test_num_replace_multy() {
        let text = ai_waifu::num2words::convert_numbers2words("I have 42.43 apples and 1.5 bananas".to_string());
        assert_eq!(text, "I have about forty-two apples and about two bananas");
    }
}
