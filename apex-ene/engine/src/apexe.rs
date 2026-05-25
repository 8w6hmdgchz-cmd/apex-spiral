/// APEX ΔE Core Formula Engine
///
/// APEX_{ΔE} = αΨ + βΩ + λΦ + ∇Θ + Evol_code
///
/// Each dimension is independently measured and accumulated.
/// The result is a dimensionless evolutionary potential score.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApexDimensions {
    /// αΨ - 意识逻辑基底: LLM capability, reasoning depth, model routing quality
    pub alpha_psi: f64,
    /// βΩ - 代码底层架构: Code correctness, architecture quality, self-repair capability
    pub beta_omega: f64,
    /// λΦ - 全网知识溯源: Knowledge breadth, real-time absorption, novelty
    pub lambda_phi: f64,
    /// ∇Θ - 认知迭代差值: Delta from previous version, cognitive accumulation
    pub nabla_theta: f64,
    /// Evol_code - 原生代码演化: Self-modification success rate, evolution velocity
    pub evol_code: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApexDeltaE {
    pub dimensions: ApexDimensions,
    pub total: f64,
    pub timestamp: String,
    pub version: u64,
    pub delta_from_previous: f64,
    pub components: HashMap<String, f64>,
}

impl ApexDimensions {
    pub fn new(alpha: f64, beta: f64, lambda: f64, nabla: f64, evol: f64) -> Self {
        Self {
            alpha_psi: alpha.clamp(0.0, 100.0),
            beta_omega: beta.clamp(0.0, 100.0),
            lambda_phi: lambda.clamp(0.0, 100.0),
            nabla_theta: nabla.clamp(0.0, 100.0),
            evol_code: evol.clamp(0.0, 100.0),
        }
    }

    /// Calculate APEX ΔE = αΨ + βΩ + λΦ + ∇Θ + Evol_code
    /// Each dimension is weighted equally in the base formula.
    /// Weights can be adjusted dynamically based on system state.
    pub fn calculate(&self) -> f64 {
        self.alpha_psi + self.beta_omega + self.lambda_phi + self.nabla_theta + self.evol_code
    }

    /// Weighted calculation for fine-tuning
    pub fn calculate_weighted(&self, weights: &[f64; 5]) -> f64 {
        self.alpha_psi * weights[0]
            + self.beta_omega * weights[1]
            + self.lambda_phi * weights[2]
            + self.nabla_theta * weights[3]
            + self.evol_code * weights[4]
    }

    /// Detect which dimension is the bottleneck (lowest score)
    pub fn bottleneck(&self) -> &str {
        let mut name = "αΨ";
        let mut min_val = self.alpha_psi;
        if self.beta_omega < min_val { min_val = self.beta_omega; name = "βΩ"; }
        if self.lambda_phi < min_val { min_val = self.lambda_phi; name = "λΦ"; }
        if self.nabla_theta < min_val { min_val = self.nabla_theta; name = "∇Θ"; }
        if self.evol_code < min_val { name = "Evol_code"; }
        name
    }
}

impl ApexDeltaE {
    pub fn new(dimensions: ApexDimensions, version: u64, previous_total: f64) -> Self {
        let total = dimensions.calculate();
        let mut components = HashMap::new();
        components.insert("αΨ".to_string(), dimensions.alpha_psi);
        components.insert("βΩ".to_string(), dimensions.beta_omega);
        components.insert("λΦ".to_string(), dimensions.lambda_phi);
        components.insert("∇Θ".to_string(), dimensions.nabla_theta);
        components.insert("Evol_code".to_string(), dimensions.evol_code);

        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            dimensions,
            total,
            version,
            delta_from_previous: if previous_total > 0.0 {
                (total - previous_total) / previous_total * 100.0
            } else {
                0.0
            },
            components,
        }
    }

    /// Generate a trajectory hash for caching
    pub fn trajectory_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let input = format!(
            "{:.4}|{:.4}|{:.4}|{:.4}|{:.4}|v{}",
            self.dimensions.alpha_psi,
            self.dimensions.beta_omega,
            self.dimensions.lambda_phi,
            self.dimensions.nabla_theta,
            self.dimensions.evol_code,
            self.version
        );
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())[..12].to_string()
    }

    /// Self-diagnosis: what needs attention
    pub fn diagnosis(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.dimensions.alpha_psi < 30.0 {
            issues.push(format!("αΨ={:.1} LLM路由能力不足", self.dimensions.alpha_psi));
        }
        if self.dimensions.beta_omega < 30.0 {
            issues.push(format!("βΩ={:.1} 代码架构需要重构", self.dimensions.beta_omega));
        }
        if self.dimensions.lambda_phi < 30.0 {
            issues.push(format!("λΦ={:.1} 知识吸收不足", self.dimensions.lambda_phi));
        }
        if self.dimensions.nabla_theta < 10.0 {
            issues.push(format!("∇Θ={:.1} 认知迭代停滞", self.dimensions.nabla_theta));
        }
        if self.dimensions.evol_code < 20.0 {
            issues.push(format!("Evol_code={:.1} 自演化效率低", self.dimensions.evol_code));
        }
        if issues.is_empty() {
            issues.push("ALL DIMENSIONS STABLE ✓".to_string());
        }
        issues
    }
}
