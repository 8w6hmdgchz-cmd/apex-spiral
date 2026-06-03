#[cfg(test)]
mod skill_tests {
    use async_trait::async_trait;
    use nano_gpt_claw::skill::{
        SkillCategory, SkillError, SkillMetadata, SkillParameter, SkillRegistry, SkillResult,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Instant;

    struct TestSkill {
        metadata: SkillMetadata,
        should_fail: bool,
    }

    impl TestSkill {
        fn new(id: &str, name: &str, should_fail: bool) -> Self {
            Self {
                metadata: SkillMetadata {
                    id: id.to_string(),
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    description: format!("Test skill: {}", name),
                    author: "Test".to_string(),
                    category: SkillCategory::Utility,
                    enabled: true,
                    parameters: vec![],
                },
                should_fail,
            }
        }
    }

    #[async_trait]
    impl nano_gpt_claw::skill::Skill for TestSkill {
        fn metadata(&self) -> &SkillMetadata {
            &self.metadata
        }

        async fn execute(
            &self,
            _params: HashMap<String, String>,
        ) -> Result<SkillResult, SkillError> {
            if self.should_fail {
                Err(SkillError::ExecutionFailed("Test failure".to_string()))
            } else {
                Ok(SkillResult {
                    success: true,
                    output: "Test executed".to_string(),
                    metadata: Default::default(),
                    execution_time_ms: 0,
                })
            }
        }
    }

    #[tokio::test]
    async fn test_skill_registry_register_and_get() {
        let registry = SkillRegistry::new();
        let skill = Arc::new(TestSkill::new("test-skill", "Test Skill", false));

        registry.register(skill.clone());

        let retrieved = registry.get("test-skill");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata().id, "test-skill");
    }

    #[tokio::test]
    async fn test_skill_registry_list_all() {
        let registry = SkillRegistry::new();

        registry.register(Arc::new(TestSkill::new("skill1", "Skill 1", false)));
        registry.register(Arc::new(TestSkill::new("skill2", "Skill 2", false)));

        let skills = registry.list_all();
        assert_eq!(skills.len(), 2);
    }

    #[tokio::test]
    async fn test_skill_registry_get_nonexistent() {
        let registry = SkillRegistry::new();
        let result = registry.get("nonexistent");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_skill_execution_success() {
        let registry = SkillRegistry::new();
        registry.register(Arc::new(TestSkill::new("success-skill", "Success", false)));

        let result = registry.execute("success-skill", HashMap::new()).await;
        assert!(result.is_ok());

        let skill_result = result.unwrap();
        assert!(skill_result.success);
        assert_eq!(skill_result.output, "Test executed");
    }

    #[tokio::test]
    async fn test_skill_execution_failure() {
        let registry = SkillRegistry::new();
        registry.register(Arc::new(TestSkill::new("fail-skill", "Fail", true)));

        let result = registry.execute("fail-skill", HashMap::new()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SkillError::ExecutionFailed(msg) => {
                assert!(msg.contains("Test failure"));
            }
            _ => panic!("Expected ExecutionFailed error"),
        }
    }

    #[tokio::test]
    async fn test_skill_not_found() {
        let registry = SkillRegistry::new();

        let result = registry.execute("nonexistent", HashMap::new()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SkillError::NotFound(id) => {
                assert_eq!(id, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }
}

#[cfg(test)]
mod auto_fix_tests {
    use nano_gpt_claw::skill::auto_fix::AutoFixSkill;
    use nano_gpt_claw::skill::Skill;
    use std::process::Command;

    #[test]
    fn test_auto_fix_creation() {
        let skill = AutoFixSkill::new();
        assert_eq!(skill.metadata().id, "auto-fix");
        assert_eq!(skill.metadata().name, "Auto Fix");
        assert!(skill.metadata().enabled);
    }

    #[test]
    fn test_auto_fix_with_max_iterations() {
        let skill = AutoFixSkill::with_max_iterations(5);
        assert_eq!(skill.metadata().id, "auto-fix");
    }

    #[tokio::test]
    async fn test_auto_fix_runs_on_clean_project() {
        let skill = AutoFixSkill::new();
        let result = skill.execute(Default::default()).await.unwrap();

        assert!(result.success);
        assert_eq!(
            result.metadata.get("status").map(|s| s.as_str()),
            Some("clean")
        );
    }

    #[test]
    fn test_cargo_available() {
        let output = Command::new("cargo").args(["--version"]).output();

        assert!(output.is_ok());
        assert!(output.unwrap().status.success());
    }
}

#[cfg(test)]
mod evolution_tests {
    use nano_gpt_claw::evolution::apex_akashic::ApexAkashicCalculator;
    use nano_gpt_claw::evolution::self_improve::SelfEvolutionEngine;
    use nano_gpt_claw::evolution::EvolutionEngine;
    use std::path::Path;

    #[test]
    fn test_evolution_engine_creation() {
        let engine = EvolutionEngine::new();
        let stats = engine.get_stats();

        assert_eq!(stats.total_events, 0);
        assert!(stats.current_score >= 0.0);
    }

    #[test]
    fn test_self_evolution_engine_creation() {
        let engine = SelfEvolutionEngine::new();
        let report = engine.generate_final_report();
        assert!(!report.is_empty());
        assert!(report.contains("APEX"));
    }

    #[test]
    fn test_apex_calculator_integration() {
        let calculator = ApexAkashicCalculator::new();
        let result = calculator.calculate();

        assert!(result.final_score >= 0.0);
        assert!(result.final_score <= 1.0);
    }

    #[tokio::test]
    async fn test_evolution_engine_initialize() {
        let config = nano_gpt_claw::evolution::EvolutionConfig {
            benchmark_interval_hours: 24,
            max_iterations_per_day: 10,
            improvement_threshold: 0.05,
            db_path: ":memory:".to_string(),
        };

        let engine = nano_gpt_claw::evolution::EvolutionEngine::with_config(config);
        engine.initialize().await;

        let stats = engine.get_stats();
        assert!(stats.current_score >= 0.0);
    }
}
