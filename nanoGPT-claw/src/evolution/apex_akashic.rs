//! APEX·阿卡西融合完整版 - 全新叠加进化总公式
//!
//! 公式定义：
//!
//! - APEX_Akashic = Omega_A * E * V * M * A * B
//!                * T * D * H * L * G * W * B
//!                - Delta_Tok - Delta_Clw - Delta_Agt - Delta_Pan - Delta_Prm - Delta_Soul
//!                - Delta_Run - Delta_Net - Delta_Err - Delta_Mem - Delta_Res - Delta_Log
//!
//! 该公式实现了完整的自进化、自监督、自优化系统，包含：
//! - 阿卡西基础因子 (Ω_A)
//! - 七大维度因子 (E, V, M, A, B, T, D, H, L, G, W, B)
//! - 十二项损失/惩罚项
//!
//! @karpathy - 致敬nanoGPT的简洁哲学

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// APEX·阿卡西计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApexAkashicResult {
    /// 最终融合分数 (0.0-1.0)
    pub final_score: f64,
    /// 阿卡西基础因子
    pub omega_a: f64,
    /// 第一维度乘积
    pub dimension_product_1: f64,
    /// 第二维度乘积
    pub dimension_product_2: f64,
    /// 总惩罚
    pub total_penalty: f64,
    /// 各项惩罚详情
    pub penalties: HashMap<String, f64>,
    /// 各项因子详情
    pub factors: HashMap<String, f64>,
    /// 建议改进项
    pub recommendations: Vec<String>,
    /// 置信度
    pub confidence: f64,
}

/// 七大维度因子配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApexDimensions {
    /// E - Evolution (进化能力)
    pub evolution: f64,
    /// V - Value (价值创造)
    pub value: f64,
    /// M - Memory (记忆能力)
    pub memory: f64,
    /// A - Autonomy (自主能力)
    pub autonomy: f64,
    /// B - Benchmark (基准表现)
    pub benchmark: f64,
    /// T - Thinking (思考深度)
    pub thinking: f64,
    /// D - Decision (决策质量)
    pub decision: f64,
    /// H - Harmony (系统和谐)
    pub harmony: f64,
    /// L - Learning (学习效率)
    pub learning: f64,
    /// G - Growth (成长潜力)
    pub growth: f64,
    /// W - Wisdom (智慧层级)
    pub wisdom: f64,
    /// B - Balance (系统平衡)
    pub balance: f64,
}

impl Default for ApexDimensions {
    fn default() -> Self {
        Self {
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
        }
    }
}

/// 十二项惩罚配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApexPenalties {
    /// Δ_Tok - Token 消耗惩罚
    pub token: f64,
    /// Δ_Clw - Claw 效率损失
    pub claw: f64,
    /// Δ_Agt - Agent 协调成本
    pub agent: f64,
    /// Δ_Pan - Panic 模式惩罚
    pub panic: f64,
    /// Δ_Prm - Prune 修剪损失
    pub prune: f64,
    /// Δ_Soul - 灵魂损耗
    pub soul: f64,
    /// Δ_Run - 运行开销
    pub runtime: f64,
    /// Δ_Net - 网络延迟
    pub network: f64,
    /// Δ_Err - 错误率
    pub error: f64,
    /// Δ_Mem - 内存使用
    pub memory: f64,
    /// Δ_Res - 资源消耗
    pub resource: f64,
    /// Δ_Log - 日志噪音
    pub log: f64,
}

impl Default for ApexPenalties {
    fn default() -> Self {
        Self {
            token: 0.02,
            claw: 0.01,
            agent: 0.015,
            panic: 0.0,
            prune: 0.005,
            soul: 0.001,
            runtime: 0.01,
            network: 0.008,
            error: 0.02,
            memory: 0.012,
            resource: 0.01,
            log: 0.005,
        }
    }
}

/// APEX·阿卡西融合计算器
#[derive(Clone)]
pub struct ApexAkashicCalculator {
    /// 阿卡西基础因子 (Ω_A)
    omega_a: f64,
    /// 维度因子
    dimensions: ApexDimensions,
    /// 惩罚因子
    penalties: ApexPenalties,
}

impl ApexAkashicCalculator {
    /// 创建新的计算器
    pub fn new() -> Self {
        Self {
            omega_a: 0.85, // 默认阿卡西基础因子
            dimensions: ApexDimensions::default(),
            penalties: ApexPenalties::default(),
        }
    }

    /// 配置阿卡西基础因子
    pub fn with_omega_a(mut self, omega: f64) -> Self {
        self.omega_a = omega.clamp(0.1, 1.0);
        self
    }

    /// 配置维度因子
    pub fn with_dimensions(mut self, dimensions: ApexDimensions) -> Self {
        self.dimensions = dimensions;
        self
    }

    /// 配置惩罚因子
    pub fn with_penalties(mut self, penalties: ApexPenalties) -> Self {
        self.penalties = penalties;
        self
    }

    /// 设置单个维度
    pub fn set_dimension(&mut self, name: &str, value: f64) -> Result<(), String> {
        let value_clamped = value.clamp(0.0, 1.0);
        match name.to_lowercase().as_str() {
            "evolution" | "e" => self.dimensions.evolution = value_clamped,
            "value" | "v" => self.dimensions.value = value_clamped,
            "memory" | "m" => self.dimensions.memory = value_clamped,
            "autonomy" | "a" => self.dimensions.autonomy = value_clamped,
            "benchmark" | "b1" => self.dimensions.benchmark = value_clamped,
            "thinking" | "t" => self.dimensions.thinking = value_clamped,
            "decision" | "d" => self.dimensions.decision = value_clamped,
            "harmony" | "h" => self.dimensions.harmony = value_clamped,
            "learning" | "l" => self.dimensions.learning = value_clamped,
            "growth" | "g" => self.dimensions.growth = value_clamped,
            "wisdom" | "w" => self.dimensions.wisdom = value_clamped,
            "balance" | "b2" => self.dimensions.balance = value_clamped,
            _ => return Err(format!("Unknown dimension: {}", name)),
        }
        Ok(())
    }

    /// 设置单个惩罚项
    pub fn set_penalty(&mut self, name: &str, value: f64) -> Result<(), String> {
        let value_clamped = value.clamp(0.0, 0.1); // 惩罚上限10%
        match name.to_lowercase().as_str() {
            "token" | "tok" => self.penalties.token = value_clamped,
            "claw" | "clw" => self.penalties.claw = value_clamped,
            "agent" | "agt" => self.penalties.agent = value_clamped,
            "panic" | "pan" => self.penalties.panic = value_clamped,
            "prune" | "prm" => self.penalties.prune = value_clamped,
            "soul" => self.penalties.soul = value_clamped,
            "runtime" | "run" => self.penalties.runtime = value_clamped,
            "network" | "net" => self.penalties.network = value_clamped,
            "error" | "err" => self.penalties.error = value_clamped,
            "memory_penalty" | "mem" => self.penalties.memory = value_clamped,
            "resource" | "res" => self.penalties.resource = value_clamped,
            "log" => self.penalties.log = value_clamped,
            _ => return Err(format!("Unknown penalty: {}", name)),
        }
        Ok(())
    }

    /// 计算APEX·阿卡西融合分数
    pub fn calculate(&self) -> ApexAkashicResult {
        // 第一维度乘积: E · V · M · A · B（归一化因子）
        let dim_prod_1 = (self.dimensions.evolution
            * self.dimensions.value
            * self.dimensions.memory
            * self.dimensions.autonomy
            * self.dimensions.benchmark)
            * 5.0; // 归一化系数，提升效果

        // 第二维度乘积: T · D · H · L · G · W · B（归一化因子）
        let dim_prod_2 = (self.dimensions.thinking
            * self.dimensions.decision
            * self.dimensions.harmony
            * self.dimensions.learning
            * self.dimensions.growth
            * self.dimensions.wisdom
            * self.dimensions.balance)
            * 5.0; // 归一化系数，提升效果

        // 计算总惩罚
        let penalty_sum = self.penalties.token
            + self.penalties.claw
            + self.penalties.agent
            + self.penalties.panic
            + self.penalties.prune
            + self.penalties.soul
            + self.penalties.runtime
            + self.penalties.network
            + self.penalties.error
            + self.penalties.memory
            + self.penalties.resource
            + self.penalties.log;

        // 最终公式
        let raw_score = self.omega_a * dim_prod_1 * dim_prod_2 - penalty_sum;
        
        // 归一化到0.0-1.0范围
        let final_score = raw_score.clamp(0.0, 1.0);

        // 收集因子详情
        let mut factors = HashMap::new();
        factors.insert("Ω_A (Omega)".to_string(), self.omega_a);
        factors.insert("E (Evolution)".to_string(), self.dimensions.evolution);
        factors.insert("V (Value)".to_string(), self.dimensions.value);
        factors.insert("M (Memory)".to_string(), self.dimensions.memory);
        factors.insert("A (Autonomy)".to_string(), self.dimensions.autonomy);
        factors.insert("B1 (Benchmark)".to_string(), self.dimensions.benchmark);
        factors.insert("T (Thinking)".to_string(), self.dimensions.thinking);
        factors.insert("D (Decision)".to_string(), self.dimensions.decision);
        factors.insert("H (Harmony)".to_string(), self.dimensions.harmony);
        factors.insert("L (Learning)".to_string(), self.dimensions.learning);
        factors.insert("G (Growth)".to_string(), self.dimensions.growth);
        factors.insert("W (Wisdom)".to_string(), self.dimensions.wisdom);
        factors.insert("B2 (Balance)".to_string(), self.dimensions.balance);

        // 收集惩罚详情
        let mut penalties = HashMap::new();
        penalties.insert("Δ_Tok (Token)".to_string(), self.penalties.token);
        penalties.insert("Δ_Clw (Claw)".to_string(), self.penalties.claw);
        penalties.insert("Δ_Agt (Agent)".to_string(), self.penalties.agent);
        penalties.insert("Δ_Pan (Panic)".to_string(), self.penalties.panic);
        penalties.insert("Δ_Prm (Prune)".to_string(), self.penalties.prune);
        penalties.insert("Δ_Soul (Soul)".to_string(), self.penalties.soul);
        penalties.insert("Δ_Run (Runtime)".to_string(), self.penalties.runtime);
        penalties.insert("Δ_Net (Network)".to_string(), self.penalties.network);
        penalties.insert("Δ_Err (Error)".to_string(), self.penalties.error);
        penalties.insert("Δ_Mem (Memory)".to_string(), self.penalties.memory);
        penalties.insert("Δ_Res (Resource)".to_string(), self.penalties.resource);
        penalties.insert("Δ_Log (Log)".to_string(), self.penalties.log);

        // 生成建议
        let recommendations = self.generate_recommendations(&factors, &penalties);

        // 计算置信度
        let confidence = self.calculate_confidence(&factors);

        ApexAkashicResult {
            final_score,
            omega_a: self.omega_a,
            dimension_product_1: dim_prod_1,
            dimension_product_2: dim_prod_2,
            total_penalty: penalty_sum,
            penalties,
            factors,
            recommendations,
            confidence,
        }
    }

    /// 生成改进建议
    fn generate_recommendations(&self, factors: &HashMap<String, f64>, penalties: &HashMap<String, f64>) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 找出最弱的3个维度
        let mut sorted_factors: Vec<_> = factors.iter()
            .filter(|(k, _)| !k.starts_with("Ω_A"))
            .collect();
        sorted_factors.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

        for (name, &score) in sorted_factors.iter().take(3) {
            if score < 0.7 {
                recommendations.push(format!("{} 提升空间: 当前{:.1}%", name, score * 100.0));
            }
        }

        // 找出最重的3个惩罚
        let mut sorted_penalties: Vec<_> = penalties.iter().collect();
        sorted_penalties.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        for (name, &penalty) in sorted_penalties.iter().take(3) {
            if penalty > 0.01 {
                recommendations.push(format!("{} 需优化: 当前惩罚{:.2}%", name, penalty * 100.0));
            }
        }

        recommendations
    }

    /// 计算置信度
    fn calculate_confidence(&self, factors: &HashMap<String, f64>) -> f64 {
        let total: f64 = factors.values().sum();
        let avg = total / factors.len() as f64;
        avg.clamp(0.5, 1.0)
    }

    /// 从系统指标更新维度
    pub fn update_from_metrics(&mut self, metrics: &SystemMetrics) {
        // 根据实际指标更新维度因子
        self.dimensions.evolution = (metrics.evolutions_per_hour / 100.0).min(1.0);
        self.dimensions.value = metrics.task_success_rate;
        self.dimensions.memory = metrics.memory_hit_rate;
        self.dimensions.autonomy = metrics.autonomous_task_rate;
        self.dimensions.thinking = metrics.reasoning_depth / 10.0;
        self.dimensions.learning = metrics.learning_rate;
        self.dimensions.growth = metrics.improvement_trend;
    }

    /// 从运行数据更新惩罚
    pub fn update_penalties_from_runtime(&mut self, runtime: &RuntimeData) {
        // 根据实际运行数据更新惩罚
        self.penalties.token = (runtime.tokens_used / 100000.0).min(0.1);
        self.penalties.error = runtime.error_rate.min(0.1);
        self.penalties.memory = (runtime.memory_used_gb / 32.0).min(0.1);
        self.penalties.network = (runtime.network_latency_ms / 1000.0).min(0.1);
    }
}

impl Default for ApexAkashicCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// 系统指标数据
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    /// 每小时进化次数
    pub evolutions_per_hour: f64,
    /// 任务成功率
    pub task_success_rate: f64,
    /// 记忆命中率
    pub memory_hit_rate: f64,
    /// 自主任务率
    pub autonomous_task_rate: f64,
    /// 推理深度
    pub reasoning_depth: f64,
    /// 学习效率
    pub learning_rate: f64,
    /// 改进趋势
    pub improvement_trend: f64,
}

/// 运行时数据
#[derive(Debug, Clone, Default)]
pub struct RuntimeData {
    /// Token使用量
    pub tokens_used: f64,
    /// 错误率
    pub error_rate: f64,
    /// 内存使用GB
    pub memory_used_gb: f64,
    /// 网络延迟ms
    pub network_latency_ms: f64,
}

/// 格式化显示结果
pub fn format_apex_result(result: &ApexAkashicResult) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("
╔═══════════════════════════════════════════════════════════════╗
"));
    output.push_str(&format!("║          APEX·阿卡西融合完整版 - 进化评估报告                  ║
"));
    output.push_str(&format!("╠═══════════════════════════════════════════════════════════════╣
"));
    output.push_str(&format!("║  最终融合分数: {:.3}    置信度: {:.1}%                    ║
", 
        result.final_score, result.confidence * 100.0));
    output.push_str(&format!("╠═══════════════════════════════════════════════════════════════╣
"));
    output.push_str(&format!("║  阿卡西因子 Ω_A: {:.3}                                          ║
", result.omega_a));
    output.push_str(&format!("║  维度乘积1:     {:.6}                                          ║
", result.dimension_product_1));
    output.push_str(&format!("║  维度乘积2:     {:.6}                                          ║
", result.dimension_product_2));
    output.push_str(&format!("║  总惩罚:        {:.3}%                                         ║
", result.total_penalty * 100.0));
    output.push_str(&format!("╠═══════════════════════════════════════════════════════════════╣
"));
    output.push_str(&format!("║  七大维度因子:                                                ║
"));
    
    let factor_names = [
        "E (Evolution)", "V (Value)", "M (Memory)", "A (Autonomy)", 
        "B1 (Benchmark)", "T (Thinking)", "D (Decision)",
        "H (Harmony)", "L (Learning)", "G (Growth)", 
        "W (Wisdom)", "B2 (Balance)"
    ];
    
    for name in &factor_names {
        if let Some(value) = result.factors.get(*name) {
            output.push_str(&format!("║    {:<15} {:.3}    ", name, value));
            let bar_len = (value * 30.0) as usize;
            output.push_str(&"█".repeat(bar_len));
            output.push_str(&"░".repeat(30 - bar_len));
            output.push_str(" ║
");
        }
    }
    
    output.push_str(&format!("╠═══════════════════════════════════════════════════════════════╣
"));
    output.push_str(&format!("║  十二项惩罚项:                                                ║
"));
    
    let penalty_names = [
        "Δ_Tok (Token)", "Δ_Clw (Claw)", "Δ_Agt (Agent)", "Δ_Pan (Panic)",
        "Δ_Prm (Prune)", "Δ_Soul (Soul)", "Δ_Run (Runtime)", "Δ_Net (Network)",
        "Δ_Err (Error)", "Δ_Mem (Memory)", "Δ_Res (Resource)", "Δ_Log (Log)"
    ];
    
    for name in &penalty_names {
        if let Some(value) = result.penalties.get(*name) {
            output.push_str(&format!("║    {:<15} {:.4}                                        ║
", name, value));
        }
    }
    
    if !result.recommendations.is_empty() {
        output.push_str(&format!("╠═══════════════════════════════════════════════════════════════╣
"));
        output.push_str(&format!("║  改进建议:                                                    ║
"));
        for rec in &result.recommendations {
            output.push_str(&format!("║    • {}
", rec));
        }
    }
    
    output.push_str(&format!("╚═══════════════════════════════════════════════════════════════╝
"));
    output.push_str(&format!("
致敬: nanoGPT @karpathy - \"The most atomic way to train and run inference\"
"));
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apex_calculation() {
        let calculator = ApexAkashicCalculator::new();
        let result = calculator.calculate();
        
        assert!(result.final_score >= 0.0);
        assert!(result.final_score <= 1.0);
        assert!(!result.factors.is_empty());
        assert!(!result.penalties.is_empty());
    }

    #[test]
    fn test_dimension_update() {
        let mut calculator = ApexAkashicCalculator::new();
        
        calculator.set_dimension("evolution", 0.9).unwrap();
        calculator.set_dimension("value", 0.95).unwrap();
        
        let result = calculator.calculate();
        assert_eq!(result.factors.get("E (Evolution)"), Some(&0.9));
        assert_eq!(result.factors.get("V (Value)"), Some(&0.95));
    }

    #[test]
    fn test_penalty_update() {
        let mut calculator = ApexAkashicCalculator::new();
        
        calculator.set_penalty("token", 0.05).unwrap();
        calculator.set_penalty("error", 0.03).unwrap();
        
        let result = calculator.calculate();
        assert_eq!(result.penalties.get("Δ_Tok (Token)"), Some(&0.05));
        assert_eq!(result.penalties.get("Δ_Err (Error)"), Some(&0.03));
    }

    #[test]
    fn test_formatting() {
        let calculator = ApexAkashicCalculator::new();
        let result = calculator.calculate();
        let formatted = format_apex_result(&result);
        
        assert!(!formatted.is_empty());
        assert!(formatted.contains("APEX·阿卡西"));
    }
}
