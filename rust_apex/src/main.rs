//! Rust APEX Calculator CLI

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApexResult {
    pub final_score: f64,
    pub omega_a: f64,
    pub dimension_product_1: f64,
    pub dimension_product_2: f64,
    pub total_penalty: f64,
    pub confidence: f64,
}

pub struct ApexCalculator {
    omega_a: f64,
    evolution: f64,
    value: f64,
    memory: f64,
    autonomy: f64,
    benchmark: f64,
    thinking: f64,
    decision: f64,
    harmony: f64,
    learning: f64,
    growth: f64,
    wisdom: f64,
    balance: f64,
    penalties: Vec<f64>,
}

impl ApexCalculator {
    pub fn new() -> Self {
        Self {
            omega_a: 0.85,
            evolution: 0.7,
            value: 0.75,
            memory: 0.8,
            autonomy: 0.65,
            benchmark: 0.7,
            thinking: 0.72,
            decision: 0.68,
            harmony: 0.75,
            learning: 0.78,
            growth: 0.8,
            wisdom: 0.7,
            balance: 0.72,
            penalties: vec![0.02, 0.01, 0.015, 0.0, 0.005, 0.001, 0.01, 0.008, 0.02, 0.012, 0.01, 0.005],
        }
    }
    
    pub fn calculate(&self) -> ApexResult {
        let dim_prod_1 = self.evolution * self.value * self.memory * self.autonomy * self.benchmark;
        let dim_prod_2 = self.thinking * self.decision * self.harmony * self.learning 
            * self.growth * self.wisdom * self.balance;
        let penalty_sum: f64 = self.penalties.iter().sum();
        let base_score = self.omega_a * 0.4;
        let dim_contribution = (dim_prod_1.powf(0.5) * dim_prod_2.powf(0.5)).min(0.5);
        let raw_score = base_score + dim_contribution - penalty_sum;
        let final_score = raw_score.clamp(0.0, 1.0);
        let confidence = (self.omega_a + penalty_sum).clamp(0.5, 1.0);
        
        ApexResult {
            final_score,
            omega_a: self.omega_a,
            dimension_product_1: dim_prod_1,
            dimension_product_2: dim_prod_2,
            total_penalty: penalty_sum,
            confidence,
        }
    }
}

impl Default for ApexCalculator {
    fn default() -> Self { Self::new() }
}

fn main() {
    let calc = ApexCalculator::new();
    let result = calc.calculate();
    
    println!("{{\"final_score\": {}, \"omega_a\": {}, \"dimension_product_1\": {}, \"dimension_product_2\": {}, \"total_penalty\": {}, \"confidence\": {}}}",
        result.final_score, result.omega_a, result.dimension_product_1, 
        result.dimension_product_2, result.total_penalty, result.confidence);
}
