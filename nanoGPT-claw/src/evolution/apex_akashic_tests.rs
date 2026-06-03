#[cfg(test)]
mod tests {
    use crate::evolution::apex_akashic::{
        format_apex_result, ApexAkashicCalculator, ApexDimensions, ApexPenalties, RuntimeData,
        SystemMetrics,
    };

    #[test]
    fn test_apex_calculation_basic() {
        let calculator = ApexAkashicCalculator::new();
        let result = calculator.calculate();

        assert!(result.final_score >= 0.0 && result.final_score <= 1.0);
        assert!(result.omega_a > 0.0 && result.omega_a <= 1.0);
        assert!(!result.factors.is_empty());
        assert!(!result.penalties.is_empty());
    }

    #[test]
    fn test_apex_dimension_update() {
        let mut calculator = ApexAkashicCalculator::new();

        assert!(calculator.set_dimension("evolution", 0.9).is_ok());
        assert!(calculator.set_dimension("value", 0.95).is_ok());
        assert!(calculator.set_dimension("invalid", 0.5).is_err());

        let result = calculator.calculate();
        assert_eq!(result.factors.get("E (Evolution)"), Some(&0.9));
        assert_eq!(result.factors.get("V (Value)"), Some(&0.95));
    }

    #[test]
    fn test_apex_penalty_update() {
        let mut calculator = ApexAkashicCalculator::new();

        assert!(calculator.set_penalty("token", 0.05).is_ok());
        assert!(calculator.set_penalty("error", 0.03).is_ok());
        assert!(calculator.set_penalty("invalid", 0.01).is_err());

        let result = calculator.calculate();
        assert_eq!(result.penalties.get("Δ_Tok (Token)"), Some(&0.05));
        assert_eq!(result.penalties.get("Δ_Err (Error)"), Some(&0.03));
    }

    #[test]
    fn test_apex_score_normalization() {
        let mut calculator = ApexAkashicCalculator::new();

        calculator.set_dimension("evolution", 1.5).ok();
        calculator.set_dimension("value", 1.5).ok();

        let result = calculator.calculate();
        assert!(result.final_score <= 1.0);
    }

    #[test]
    fn test_apex_builder_pattern() {
        let calculator = ApexAkashicCalculator::new()
            .with_omega_a(0.9)
            .with_dimensions(ApexDimensions {
                evolution: 0.95,
                value: 0.90,
                memory: 0.85,
                autonomy: 0.80,
                benchmark: 0.88,
                thinking: 0.82,
                decision: 0.78,
                harmony: 0.92,
                learning: 0.88,
                growth: 0.90,
                wisdom: 0.85,
                balance: 0.87,
            })
            .with_penalties(ApexPenalties {
                token: 0.01,
                claw: 0.005,
                agent: 0.008,
                panic: 0.0,
                prune: 0.003,
                soul: 0.0005,
                runtime: 0.004,
                network: 0.003,
                error: 0.006,
                memory: 0.004,
                resource: 0.003,
                log: 0.002,
            });

        let result = calculator.calculate();
        assert!(result.final_score > 0.0);
    }

    #[test]
    fn test_apex_recommendations_generated() {
        let calculator = ApexAkashicCalculator::new();
        let result = calculator.calculate();

        assert!(!result.recommendations.is_empty());
    }

    #[test]
    fn test_apex_format_result() {
        let calculator = ApexAkashicCalculator::new();
        let result = calculator.calculate();
        let formatted = format_apex_result(&result);

        assert!(!formatted.is_empty());
        assert!(formatted.contains("APEX"));
    }

    #[test]
    fn test_system_metrics_default() {
        let metrics = SystemMetrics::default();
        assert_eq!(metrics.evolutions_per_hour, 0.0);
        assert_eq!(metrics.task_success_rate, 0.0);
        assert_eq!(metrics.memory_hit_rate, 0.0);
    }

    #[test]
    fn test_runtime_data_default() {
        let runtime = RuntimeData::default();
        assert_eq!(runtime.tokens_used, 0.0);
        assert_eq!(runtime.error_rate, 0.0);
        assert_eq!(runtime.memory_used_gb, 0.0);
    }

    #[test]
    fn test_apex_update_from_metrics() {
        let mut calculator = ApexAkashicCalculator::new();
        let metrics = SystemMetrics {
            evolutions_per_hour: 50.0,
            task_success_rate: 0.95,
            memory_hit_rate: 0.88,
            autonomous_task_rate: 0.75,
            reasoning_depth: 8.0,
            learning_rate: 0.9,
            improvement_trend: 0.85,
        };

        calculator.update_from_metrics(&metrics);
        let result = calculator.calculate();

        assert!(result.factors.get("E (Evolution)").is_some());
    }

    #[test]
    fn test_apex_update_penalties_from_runtime() {
        let mut calculator = ApexAkashicCalculator::new();
        let runtime = RuntimeData {
            tokens_used: 50000.0,
            error_rate: 0.02,
            memory_used_gb: 8.0,
            network_latency_ms: 150.0,
        };

        calculator.update_penalties_from_runtime(&runtime);
        let result = calculator.calculate();

        assert!(result.total_penalty > 0.0);
    }

    #[test]
    fn test_apex_score_comparison_after_improvement() {
        let mut calculator = ApexAkashicCalculator::new();

        let before = calculator.calculate();

        calculator
            .set_dimension(
                "evolution",
                (before.factors.get("E (Evolution)").unwrap() + 0.1).min(1.0),
            )
            .ok();
        calculator
            .set_dimension(
                "value",
                (before.factors.get("V (Value)").unwrap() + 0.1).min(1.0),
            )
            .ok();
        calculator.set_penalty("error", 0.005).ok();

        let after = calculator.calculate();
        assert!(after.final_score >= before.final_score);
    }
}
