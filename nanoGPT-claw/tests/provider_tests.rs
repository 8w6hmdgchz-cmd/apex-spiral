//! Test suite for LLM provider system with mockall

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::mock;
    use nano_gpt_claw::scheduler::provider::{LLMProvider, LLMResponse, LLMResult};

    // Mock LLMProvider
    mock! {
        pub TestProvider {}

        #[async_trait]
        impl LLMProvider for TestProvider {
            fn provider_name(&self) -> &str;
            fn default_model(&self) -> &str;
            async fn complete(&self, prompt: &str) -> LLMResult<LLMResponse>;
            async fn complete_with_messages(&self, messages: Vec<nano_gpt_claw::scheduler::provider::ChatMessage>) -> LLMResult<LLMResponse>;
        }
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let mut mock = MockTestProvider::new();

        mock.expect_provider_name()
            .return_const("test-provider".to_string());

        mock.expect_default_model()
            .return_const("test-model".to_string());

        mock.expect_complete().returning(|prompt| {
            Ok(LLMResponse {
                content: format!("Mock response to: {}", prompt),
                model: "test-model".to_string(),
                provider: "test-provider".to_string(),
                usage: None,
                finish_reason: Some("stop".to_string()),
            })
        });

        assert_eq!(mock.provider_name(), "test-provider");
        assert_eq!(mock.default_model(), "test-model");

        let result = mock.complete("Hello").await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.content.contains("Mock response"));
    }

    #[test]
    fn test_retry_config() {
        use nano_gpt_claw::scheduler::retry::RetryConfig;

        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);

        let custom_config = RetryConfig::new().with_max_retries(5);
        assert_eq!(custom_config.max_retries, 5);
    }

    #[test]
    fn test_apex_fitness() {
        use nano_gpt_claw::evolution::bench::calculate_apex_fitness;

        // Perfect scenario
        let fitness = calculate_apex_fitness(1.0, 1.0, 0.0);
        assert!((fitness - 1.0).abs() < 0.001);

        // Good scenario
        let fitness = calculate_apex_fitness(0.9, 0.85, 0.05);
        assert!(fitness > 0.7);

        // Bad scenario
        let fitness = calculate_apex_fitness(0.5, 0.4, 0.5);
        assert!(fitness < 0.5);

        // Clamping
        let fitness = calculate_apex_fitness(2.0, 1.5, -0.1);
        assert!((fitness - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_env_interpolation() {
        use nano_gpt_claw::config::env::interpolate_env_vars;

        std::env::set_var("TEST_VAR", "test_value");

        let result = interpolate_env_vars("Hello ${TEST_VAR}");
        assert_eq!(result, "Hello test_value");

        let missing = interpolate_env_vars("${MISSING_VAR}");
        assert_eq!(missing, "${MISSING_VAR}");

        std::env::remove_var("TEST_VAR");
    }
}
