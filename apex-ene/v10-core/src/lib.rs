// ═══════════════════════════════════════════════════════════════════════
// APEX V10.1 Core Formula Engine
//
// ΔG_ultimate = (Λ_root × Θ_llm-agent × K_master × ξ_anti-hallucination × Ψ_host × Φ_cycle)
//             / (H_real × T × ε_self-repair)
//
// V10.1 additions:
//   Σ_memory — Super-memory module
//   τ_trace  — Process trace module
//   Ω_dawn   — Dawn self-evolution module
//
// All piracy/watermark/license code has been stripped.
// This is the clean formula-only implementation.
// ═══════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::f64::consts::E;

// ═══════════════════════════════════════════════════════════════════════
// V8.0 参数结构体
// ═══════════════════════════════════════════════════════════════════════

/// 单LLM多任务Agent效能参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAgentParams {
    pub lambda_single_call: f64,   // λ 单次调用质量系数
    pub mu_multi_task: f64,        // μ 多任务并行系数
    pub sigma_high_quality: f64,   // σ 高质量输出系数
    pub gamma_llm_cost: f64,       // γ LLM调用成本系数
}

/// 公式通解+技能全域掌握参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterParams {
    pub k_code: f64,            // K_code 编码掌握系数
    pub tau_transfer: Vec<f64>, // τ_transfer^i 跨领域迁移系数列表
    pub upsilon_apply: f64,     // υ_apply 技能应用系数
}

/// 全场景自主深度修复参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfRepairParams {
    pub g_target: f64,             // G_target 目标增益
    pub g_actual: f64,            // G_actual 实际增益
    pub delta_error_locate: f64,  // δ 错误定位效率系数
    pub psi_thorough_fix: f64,   // ψ 彻底修复系数
    pub kappa_no_repeat: f64,     // κ 防复发系数
}

/// 正向循环反馈增益参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleParams {
    pub eta_skill_up: f64,        // η 技能提升系数
    pub rho_result_feedback: f64, // ρ 结果反馈系数
}

/// 主机全维度健康稳态参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostHealthParams {
    pub psi_mem: f64,    // Ψ_mem 内存健康系数
    pub psi_app: f64,    // Ψ_app 应用健康系数
    pub psi_disk: f64,   // Ψ_disk 磁盘健康系数
    pub omega_dawn: f64, // Ω_dawn 启动就绪系数
}

/// V8.0 全量参数容器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApexParamsV8 {
    pub lambda_root: f64,               // Λ_root 本源务实基因系数
    pub xi_anti_hallucination: f64,     // ξ_anti-hallucination 幻觉零容忍硬锁系数
    pub h_real: f64,                    // H_real 真实有效信息熵
    pub t_iteration: f64,               // T=2 迭代周期（默认2.0）
    pub llm_agent: LlmAgentParams,
    pub master: MasterParams,
    pub self_repair: SelfRepairParams,
    pub cycle: CycleParams,
    pub host: HostHealthParams,
}

// ═══════════════════════════════════════════════════════════════════════
// V8.0 子公式计算函数
// ═══════════════════════════════════════════════════════════════════════

/// Θ_llm-agent = (λ_single-call × μ_multi-task × σ_high-quality) / (γ_llm-cost + 1)
pub fn calculate_llm_agent_efficiency(params: &LlmAgentParams) -> f64 {
    let numerator = params.lambda_single_call * params.mu_multi_task * params.sigma_high_quality;
    let denominator = params.gamma_llm_cost + 1.0;
    if denominator == 0.0 { 0.0 } else { numerator / denominator }
}

/// K_master = K_code × (1 + Σ τ_transfer^i) × υ_apply (原始版)
pub fn calculate_k_master(params: &MasterParams) -> f64 {
    let sum_tau = params.tau_transfer.iter().sum::<f64>();
    params.k_code * (1.0 + sum_tau) * params.upsilon_apply
}

/// K_master V8.2 safe版 — τ/(1-τ) 收敛约束, τ∈[0,0.99)
pub fn calculate_k_master_safe(params: &MasterParams) -> f64 {
    let sum_tau_converged: f64 = params
        .tau_transfer.iter()
        .map(|&t| {
            let safe_t = t.max(0.0).min(0.99);
            safe_t / (1.0 - safe_t)
        })
        .sum();
    params.k_code * (1.0 + sum_tau_converged) * params.upsilon_apply
}

/// ε_self-repair = 1 + |(G_target - G_actual) / G_actual| × δ × ψ × κ
pub fn calculate_self_repair(params: &SelfRepairParams) -> f64 {
    if params.g_actual == 0.0 {
        return f64::INFINITY;
    }
    let relative_error = ((params.g_target - params.g_actual) / params.g_actual).abs();
    1.0 + relative_error * params.delta_error_locate * params.psi_thorough_fix * params.kappa_no_repeat
}

/// Φ_cycle = e^(η × ρ) (原始版)
pub fn calculate_cycle_gain(params: &CycleParams) -> f64 {
    E.powf(params.eta_skill_up * params.rho_result_feedback)
}

/// Φ_cycle V8.2 safe版 — e^7 cap 防数值爆炸
pub fn calculate_cycle_gain_safe(params: &CycleParams) -> f64 {
    E.powf((params.eta_skill_up * params.rho_result_feedback).min(7.0))
}

/// Ψ_host = Ψ_mem × Ψ_app × Ψ_disk × Ω_dawn
pub fn calculate_host_health(params: &HostHealthParams) -> f64 {
    params.psi_mem * params.psi_app * params.psi_disk * params.omega_dawn
}

// ═══════════════════════════════════════════════════════════════════════
// V8.0 主公式
// ═══════════════════════════════════════════════════════════════════════

/// ΔG_ultimate = (Λ_root × Θ_llm-agent × K_master × ξ × Ψ_host × Φ_cycle) / (H_real × T × ε)
pub fn calculate_delta_g_ultimate(params: &ApexParamsV8) -> Result<f64, String> {
    if params.h_real <= 0.0 {
        return Err("H_real must be > 0".into());
    }
    if params.t_iteration <= 0.0 {
        return Err("T_iteration must be > 0".into());
    }

    let theta_llm = calculate_llm_agent_efficiency(&params.llm_agent);
    let k_master = calculate_k_master(&params.master);
    let epsilon_self_repair = calculate_self_repair(&params.self_repair);
    let phi_cycle = calculate_cycle_gain(&params.cycle);
    let psi_host = calculate_host_health(&params.host);

    if epsilon_self_repair == 0.0 {
        return Err("ε_self-repair cannot be 0".into());
    }

    let numerator = params.lambda_root
        * theta_llm
        * k_master
        * params.xi_anti_hallucination
        * psi_host
        * phi_cycle;

    let denominator = params.h_real * params.t_iteration * epsilon_self_repair;

    Ok(numerator / denominator)
}

/// ΔG safe版 — 使用 V8.2 safe sub-formulas (τ收敛 + e^7 cap)
pub fn calculate_delta_g_ultimate_safe(params: &ApexParamsV8) -> Result<f64, String> {
    if params.h_real <= 0.0 {
        return Err("H_real must be > 0".into());
    }
    if params.t_iteration <= 0.0 {
        return Err("T_iteration must be > 0".into());
    }

    let theta_llm = calculate_llm_agent_efficiency(&params.llm_agent);
    let k_master = calculate_k_master_safe(&params.master);
    let epsilon_self_repair = calculate_self_repair(&params.self_repair);
    let phi_cycle = calculate_cycle_gain_safe(&params.cycle);
    let psi_host = calculate_host_health(&params.host);

    if epsilon_self_repair == 0.0 {
        return Err("ε_self-repair cannot be 0".into());
    }

    let numerator = params.lambda_root
        * theta_llm
        * k_master
        * params.xi_anti_hallucination
        * psi_host
        * phi_cycle;

    let denominator = params.h_real * params.t_iteration * epsilon_self_repair;

    Ok(numerator / denominator)
}

/// 单行Rust表达式版本（嵌入式直接计算）
#[inline]
pub fn delta_g_ultimate_inline(
    lambda_root: f64, xi_anti_hall: f64, h_real: f64, t_iter: f64,
    lambda_sc: f64, mu_mt: f64, sigma_hq: f64, gamma_cost: f64,
    k_code: f64, sum_tau: f64, upsilon_apply: f64,
    g_target: f64, g_actual: f64, delta_err: f64, psi_fix: f64, kappa_nr: f64,
    eta_skill: f64, rho_fb: f64,
    psi_mem: f64, psi_app: f64, psi_disk: f64, omega_dawn: f64,
) -> f64 {
    let theta = (lambda_sc * mu_mt * sigma_hq) / (gamma_cost + 1.0);
    let k_master = k_code * (1.0 + sum_tau) * upsilon_apply;
    let eps = 1.0 + ((g_target - g_actual) / g_actual).abs() * delta_err * psi_fix * kappa_nr;
    let phi = E.powf(eta_skill * rho_fb);
    let psi_host = psi_mem * psi_app * psi_disk * omega_dawn;
    (lambda_root * theta * k_master * xi_anti_hall * psi_host * phi) / (h_real * t_iter * eps)
}

/// 综合进化得分（归一化到 [0,1]）
pub fn evolution_score(delta_g: f64, h_real: f64) -> f64 {
    delta_g / (delta_g + h_real + 1e-10)
}

// ═══════════════════════════════════════════════════════════════════════
// V8.1 五实战系数
// ═══════════════════════════════════════════════════════════════════════

/// V8.1 内部版全量参数容器（璇玑帝国实战扩展）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V8ParamsInternal {
    pub lambda_root: f64,
    pub theta_llm: f64,
    pub k_master: f64,
    pub xi_anti_hallucination: f64,
    pub psi_host: f64,
    pub phi_cycle: f64,
    pub h_real: f64,
    pub epsilon_self_repair: f64,
    pub t: f64,
    // V8.1 新增5个实战系数
    pub phi_network: f64,
    pub gamma_mutation: f64,
    pub omega_session: f64,
    pub pi_coord: f64,
    pub sigma_storage: f64,
}

/// Φ_network = (1 - retry_rate) × (1 - rate_limit_freq) × conn_stable
pub fn calc_phi_network(retry_rate: f64, rate_limit_freq: f64, conn_stable: f64) -> f64 {
    (1.0 - retry_rate) * (1.0 - rate_limit_freq) * conn_stable
}

/// Γ_mutation: code_change_rate < threshold ? 0.1 : code_change_rate
pub fn calc_gamma_mutation(code_change_rate: f64, hollow_threshold: f64) -> f64 {
    if code_change_rate < hollow_threshold { 0.1 } else { code_change_rate }
}

/// Ω_session = (1 - restart_freq) × (1 - env_loss_rate) × recovery_success
pub fn calc_omega_session(restart_freq: f64, env_loss_rate: f64, recovery_success: f64) -> f64 {
    (1.0 - restart_freq) * (1.0 - env_loss_rate) * recovery_success
}

/// Π_coord = (alive_procs / total_procs) × (1 - zombie_rate) × callback_success
pub fn calc_pi_coord(alive_procs: usize, total_procs: usize, zombie_rate: f64, callback_success: f64) -> f64 {
    if total_procs == 0 {
        1.0
    } else {
        (alive_procs as f64 / total_procs as f64) * (1.0 - zombie_rate) * callback_success
    }
}

/// Σ_storage = free_disk_ratio × (1 - write_fail_rate) × integrity
pub fn calc_sigma_storage(free_disk_ratio: f64, write_fail_rate: f64, integrity: f64) -> f64 {
    free_disk_ratio * (1.0 - write_fail_rate) * integrity
}

/// ΔG_v8_1 = (... × Φ_network × Γ_mutation × Ω_session × Π_coord × Σ_storage) / (H_real × T × ε)
pub fn calculate_delta_g_v8_1(params: &V8ParamsInternal) -> f64 {
    let numerator = params.lambda_root
        * params.theta_llm
        * params.k_master
        * params.xi_anti_hallucination
        * params.psi_host
        * params.phi_cycle
        * params.phi_network
        * params.gamma_mutation
        * params.omega_session
        * params.pi_coord
        * params.sigma_storage;
    let denominator = params.h_real * params.t * params.epsilon_self_repair;
    let safe_denom = denominator.max(0.001);
    (numerator / safe_denom).min(1000.0)
}

/// 从 V8.0 ApexParamsV8 构建 V8ParamsInternal，新增5系数默认1.0
pub fn from_v8_to_internal(v8: &ApexParamsV8) -> V8ParamsInternal {
    V8ParamsInternal {
        lambda_root: v8.lambda_root,
        theta_llm: calculate_llm_agent_efficiency(&v8.llm_agent),
        k_master: calculate_k_master(&v8.master),
        xi_anti_hallucination: v8.xi_anti_hallucination,
        psi_host: calculate_host_health(&v8.host),
        phi_cycle: calculate_cycle_gain(&v8.cycle),
        h_real: v8.h_real,
        epsilon_self_repair: calculate_self_repair(&v8.self_repair),
        t: v8.t_iteration,
        phi_network: 1.0,
        gamma_mutation: 1.0,
        omega_session: 1.0,
        pi_coord: 1.0,
        sigma_storage: 1.0,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// V8.2 璇玑帝国原始输入 + 实时重算五系数
// ═══════════════════════════════════════════════════════════════════════

/// 五系数原始输入（用于实时重算）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XuanjiRawInputs {
    pub retry_rate: f64,
    pub rate_limit_freq: f64,
    pub conn_stable: f64,
    pub code_change_rate: f64,
    pub hollow_threshold: f64,
    pub restart_freq: f64,
    pub env_loss_rate: f64,
    pub recovery_success: f64,
    pub alive_procs: usize,
    pub total_procs: usize,
    pub zombie_rate: f64,
    pub callback_success: f64,
    pub free_disk_ratio: f64,
    pub write_fail_rate: f64,
    pub integrity: f64,
}

impl Default for XuanjiRawInputs {
    fn default() -> Self {
        XuanjiRawInputs {
            retry_rate: 0.05, rate_limit_freq: 0.02, conn_stable: 0.98,
            code_change_rate: 0.5, hollow_threshold: 0.01,
            restart_freq: 0.01, env_loss_rate: 0.02, recovery_success: 0.95,
            alive_procs: 1, total_procs: 1, zombie_rate: 0.0, callback_success: 1.0,
            free_disk_ratio: 0.8, write_fail_rate: 0.001, integrity: 0.999,
        }
    }
}

/// 五系数统一计算入口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XuanjiCoefficients {
    pub phi_network: f64,
    pub gamma_mutation: f64,
    pub omega_session: f64,
    pub pi_coord: f64,
    pub sigma_storage: f64,
}

impl XuanjiCoefficients {
    pub fn compute(inputs: &XuanjiRawInputs) -> Self {
        XuanjiCoefficients {
            phi_network: (1.0 - inputs.retry_rate) * (1.0 - inputs.rate_limit_freq) * inputs.conn_stable,
            gamma_mutation: if inputs.code_change_rate < inputs.hollow_threshold { 0.1 } else { inputs.code_change_rate },
            omega_session: (1.0 - inputs.restart_freq) * (1.0 - inputs.env_loss_rate) * inputs.recovery_success,
            pi_coord: if inputs.total_procs == 0 {
                1.0
            } else {
                (inputs.alive_procs as f64 / inputs.total_procs as f64) * (1.0 - inputs.zombie_rate) * inputs.callback_success
            },
            sigma_storage: inputs.free_disk_ratio * (1.0 - inputs.write_fail_rate) * inputs.integrity,
        }
    }
}

/// V8.2 安全版参数容器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V8ParamsInternalV82 {
    pub lambda_root: f64,
    pub theta_llm: f64,
    pub k_master: f64,
    pub xi_anti_hallucination: f64,
    pub psi_host: f64,
    pub phi_cycle: f64,
    pub h_real: f64,
    pub epsilon_self_repair: f64,
    pub t: f64,
    pub xuanji_inputs: XuanjiRawInputs,
    pub coefficients_cache: Option<XuanjiCoefficients>,
}

pub fn calculate_delta_g_v8_2(params: &V8ParamsInternalV82) -> f64 {
    let coeffs = params.coefficients_cache.clone()
        .unwrap_or_else(|| XuanjiCoefficients::compute(&params.xuanji_inputs));
    let numerator = params.lambda_root * params.theta_llm * params.k_master
        * params.xi_anti_hallucination * params.psi_host * params.phi_cycle
        * coeffs.phi_network * coeffs.gamma_mutation * coeffs.omega_session
        * coeffs.pi_coord * coeffs.sigma_storage;
    let denominator = params.h_real * params.t * params.epsilon_self_repair;
    let safe_denom = denominator.max(0.001);
    (numerator / safe_denom).min(1000.0)
}

/// 从 V8.0 构建 V8.2 安全版，使用 safe sub-formulas
pub fn from_v8_to_internal_v82(v8: &ApexParamsV8) -> V8ParamsInternalV82 {
    V8ParamsInternalV82 {
        lambda_root: v8.lambda_root,
        theta_llm: calculate_llm_agent_efficiency(&v8.llm_agent),
        k_master: calculate_k_master_safe(&v8.master),
        xi_anti_hallucination: v8.xi_anti_hallucination,
        psi_host: calculate_host_health(&v8.host),
        phi_cycle: calculate_cycle_gain_safe(&v8.cycle),
        h_real: v8.h_real,
        epsilon_self_repair: calculate_self_repair(&v8.self_repair),
        t: v8.t_iteration,
        xuanji_inputs: XuanjiRawInputs::default(),
        coefficients_cache: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// V8.4 自我意识模块
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAwarenessParams {
    pub sigma_coherence: f64,
    pub delta_drift: f64,
    pub rho_alignment: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionParams {
    pub weights: Vec<f64>,
    pub quality_deltas: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfEvolutionParams {
    pub awareness: SelfAwarenessParams,
    pub reflection: ReflectionParams,
    pub threshold_positive: f64,
    pub threshold_negative: f64,
}

/// Ω_self = σ_coherence × (1 - δ_drift) × ρ_alignment
pub fn calculate_omega_self(params: &SelfAwarenessParams) -> f64 {
    params.sigma_coherence * (1.0 - params.delta_drift) * params.rho_alignment
}

/// Γ_reflect = Σ(w_i × ΔQ_i) / Σw_i
pub fn calculate_gamma_reflect(params: &ReflectionParams) -> f64 {
    let sum_w = params.weights.iter().sum::<f64>();
    if sum_w <= 0.0 {
        return 0.0;
    }
    params.weights.iter()
        .zip(params.quality_deltas.iter())
        .map(|(w, d)| w * d)
        .sum::<f64>() / sum_w
}

/// ΔG_total = ΔG_task × Ω_self × (1 + Γ_reflect)
/// mode: "reinforce" | "repair" | "maintain"
pub fn calculate_self_evolution_gain(
    delta_g_task: f64, params: &SelfEvolutionParams,
) -> (f64, &'static str) {
    let omega_self = calculate_omega_self(&params.awareness);
    let gamma_reflect = calculate_gamma_reflect(&params.reflection);
    let delta_g_total = delta_g_task * omega_self * (1.0 + gamma_reflect);
    let mode = if gamma_reflect > params.threshold_positive {
        "reinforce"
    } else if gamma_reflect < params.threshold_negative {
        "repair"
    } else {
        "maintain"
    };
    (delta_g_total, mode)
}

// ═══════════════════════════════════════════════════════════════════════
// V10.1 Σ_memory 超忆全域记忆模块
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f64>,
    pub timestamp: i64,
    pub importance: f64,
    pub memory_type: MemoryType,
    pub access_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryType {
    Semantic,
    Episodic,
    Procedural,
    Working,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperMemoryParams {
    pub learn_rate: f64,
    pub decay_factor: f64,
    pub max_entries: usize,
    pub retention_threshold: f64,
    pub memory_entries: Vec<MemoryEntry>,
}

impl Default for SuperMemoryParams {
    fn default() -> Self {
        SuperMemoryParams {
            learn_rate: 0.7,
            decay_factor: 0.95,
            max_entries: 10000,
            retention_threshold: 0.6,
            memory_entries: Vec::new(),
        }
    }
}

/// Σ_memory = Learn × Search × MultiModal × Profile × decay
pub fn calculate_sigma_memory(params: &SuperMemoryParams) -> f64 {
    let learn = params.learn_rate.min(1.0).max(0.0);
    let search = (params.retention_threshold * params.learn_rate).sqrt();
    let type_diversity = calculate_type_diversity(&params.memory_entries);
    let multimodal = (learn * search * type_diversity).max(0.1);
    let profile = 0.1f64.max(multimodal);
    let decay = params.decay_factor.max(0.0).min(1.0);
    learn * search * multimodal * profile * decay
}

fn calculate_type_diversity(entries: &[MemoryEntry]) -> f64 {
    if entries.is_empty() {
        return 0.5;
    }
    let type_counts = [
        entries.iter().filter(|e| e.memory_type == MemoryType::Semantic).count(),
        entries.iter().filter(|e| e.memory_type == MemoryType::Episodic).count(),
        entries.iter().filter(|e| e.memory_type == MemoryType::Procedural).count(),
        entries.iter().filter(|e| e.memory_type == MemoryType::Working).count(),
    ];
    let total = entries.len() as f64;
    let diversity: f64 = type_counts.iter()
        .map(|&c| {
            let p = c as f64 / total;
            if p > 0.0 { -p * p.log2() } else { 0.0 }
        })
        .sum();
    (diversity / 2.0).max(0.1).min(1.0)
}

pub fn add_memory_entry(params: &mut SuperMemoryParams, entry: MemoryEntry) {
    if params.memory_entries.len() >= params.max_entries {
        remove_low_importance_entries(params);
    }
    params.memory_entries.push(entry);
}

fn remove_low_importance_entries(params: &mut SuperMemoryParams) {
    let keep_count = (params.max_entries as f64 * 0.8) as usize;
    params.memory_entries.sort_by(|a, b| {
        b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal)
    });
    params.memory_entries.truncate(keep_count);
}

pub fn access_memory(params: &mut SuperMemoryParams, entry_id: &str) {
    if let Some(entry) = params.memory_entries.iter_mut().find(|e| e.id == entry_id) {
        entry.access_count += 1;
        entry.importance = (entry.importance + 0.01).min(1.0);
    }
}

pub fn search_memory<'a>(params: &'a SuperMemoryParams, query: &str) -> Vec<&'a MemoryEntry> {
    params.memory_entries.iter()
        .filter(|e| e.content.to_lowercase().contains(&query.to_lowercase()))
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════
// V10.1 τ_trace 过程追踪模块
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    pub step: u32,
    pub decision: String,
    pub reason: String,
    pub result: String,
    pub delta_g: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceParams {
    pub entries: Vec<TraceEntry>,
    pub max_entries: usize,
}

impl Default for TraceParams {
    fn default() -> Self {
        TraceParams { entries: Vec::new(), max_entries: 1000 }
    }
}

/// τ_trace = (1/N) × Σ(Decision + Reason + Result)
pub fn calculate_tau_trace(params: &TraceParams) -> f64 {
    let n = params.entries.len() as f64;
    if n == 0.0 { return 0.0; }
    let sum: f64 = params.entries.iter()
        .map(|e| {
            let d = if !e.decision.is_empty() { 1.0 } else { 0.0 };
            let r = if !e.reason.is_empty() { 1.0 } else { 0.0 };
            let res = if !e.result.is_empty() { 1.0 } else { 0.0 };
            (d + r + res) / 3.0
        })
        .sum();
    sum / n
}

pub fn add_trace_entry(params: &mut TraceParams, entry: TraceEntry) {
    if params.entries.len() >= params.max_entries {
        params.entries.remove(0);
    }
    params.entries.push(entry);
}

pub fn trace_to_delta_g_contribution(tau_trace: f64, base_delta_g: f64) -> f64 {
    base_delta_g * (0.5 + 0.5 * tau_trace)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSummary {
    pub total_steps: usize,
    pub complete_steps: usize,
    pub completeness_rate: f64,
    pub tau_trace: f64,
}

pub fn get_trace_summary(params: &TraceParams) -> TraceSummary {
    let total_steps = params.entries.len();
    let complete_steps = params.entries.iter()
        .filter(|e| !e.decision.is_empty() && !e.reason.is_empty() && !e.result.is_empty())
        .count();
    let tau = calculate_tau_trace(params);
    TraceSummary {
        total_steps,
        complete_steps,
        completeness_rate: if total_steps > 0 { complete_steps as f64 / total_steps as f64 } else { 0.0 },
        tau_trace: tau,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Ω_dawn 凌晨自进化模块
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSyncParams {
    pub delta_version_diff: f64,
    pub rho_sync_fail: f64,
    pub tau_auto_merge: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoLearnParams {
    pub l_extract: f64,
    pub g_generalize: f64,
    pub s_summarize: f64,
    pub t_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DawnParams {
    pub omega_dawn: f64,
    pub git_sync: GitSyncParams,
    pub auto_learn: AutoLearnParams,
}

pub fn calculate_git_sync(params: &GitSyncParams) -> f64 {
    let delta = params.delta_version_diff.clamp(0.0, 1.0);
    let rho = params.rho_sync_fail.clamp(0.0, 1.0);
    let tau = params.tau_auto_merge.clamp(0.0, 1.0);
    (1.0 - delta) * (1.0 - rho) * tau
}

pub fn calculate_auto_learn(params: &AutoLearnParams) -> f64 {
    let l = params.l_extract.clamp(0.0, 1.0);
    let g = params.g_generalize.clamp(0.0, 1.0);
    let s = params.s_summarize.clamp(0.0, 1.0);
    let t = params.t_time.max(0.0);
    let numerator = l * g * s;
    let denominator = (t + 1.0).max(0.001);
    numerator / denominator
}

pub fn calculate_dawn_omega(params: &DawnParams) -> f64 {
    let git_sync = calculate_git_sync(&params.git_sync);
    let auto_learn = calculate_auto_learn(&params.auto_learn);
    params.omega_dawn * git_sync * auto_learn
}

// ═══════════════════════════════════════════════════════════════════════
// V10.1 综合计算结果（JSON友好的全量输出）
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V10Result {
    pub delta_g: f64,
    pub delta_g_safe: Option<f64>,
    pub evolution_score: f64,
    pub bottleneck: String,
    pub theta: f64,
    pub k_master: f64,
    pub epsilon: f64,
    pub phi_cycle: f64,
    pub psi_host: f64,
    pub tau_trace: Option<f64>,
    pub sigma_memory: Option<f64>,
    pub omega_self: Option<f64>,
    pub gamma_reflect: Option<f64>,
    pub dawn_omega: Option<f64>,
    pub phi_network: Option<f64>,
    pub gamma_mutation: Option<f64>,
    pub omega_session: Option<f64>,
    pub pi_coord: Option<f64>,
    pub sigma_storage: Option<f64>,
    pub version: &'static str,
}

/// 完整 V10.1 计算入口
pub fn calculate_v10_full(params: &ApexParamsV8, v8_1_coeffs: Option<XuanjiCoefficients>) -> V10Result {
    let theta = calculate_llm_agent_efficiency(&params.llm_agent);
    let k_master = calculate_k_master(&params.master);
    let eps = calculate_self_repair(&params.self_repair);
    let phi = calculate_cycle_gain(&params.cycle);
    let psi = calculate_host_health(&params.host);

    let delta_g = params.lambda_root * theta * k_master * params.xi_anti_hallucination
        * psi * phi / (params.h_real * params.t_iteration * eps);

    let delta_g_safe = match calculate_delta_g_ultimate_safe(params) {
        Ok(v) => Some(v),
        Err(_) => None,
    };

    let mut bottleneck = String::from("none");
    let min_val = params.llm_agent.lambda_single_call
        .min(params.llm_agent.mu_multi_task)
        .min(params.llm_agent.sigma_high_quality)
        .min(params.master.k_code)
        .min(params.master.upsilon_apply)
        .min(params.host.psi_mem)
        .min(params.host.psi_app)
        .min(params.host.psi_disk);
    if min_val <= 0.3 { bottleneck = String::from("low"); }

    V10Result {
        delta_g,
        delta_g_safe,
        evolution_score: evolution_score(delta_g, params.h_real),
        bottleneck,
        theta,
        k_master,
        epsilon: eps,
        phi_cycle: phi,
        psi_host: psi,
        tau_trace: None,
        sigma_memory: None,
        omega_self: None,
        gamma_reflect: None,
        dawn_omega: None,
        phi_network: v8_1_coeffs.as_ref().map(|c| c.phi_network),
        gamma_mutation: v8_1_coeffs.as_ref().map(|c| c.gamma_mutation),
        omega_session: v8_1_coeffs.as_ref().map(|c| c.omega_session),
        pi_coord: v8_1_coeffs.as_ref().map(|c| c.pi_coord),
        sigma_storage: v8_1_coeffs.as_ref().map(|c| c.sigma_storage),
        version: "V10.1",
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn default_v8_params() -> ApexParamsV8 {
        ApexParamsV8 {
            lambda_root: 0.95,
            xi_anti_hallucination: 1.0,
            h_real: 0.5,
            t_iteration: 2.0,
            llm_agent: LlmAgentParams {
                lambda_single_call: 0.9, mu_multi_task: 0.85,
                sigma_high_quality: 0.88, gamma_llm_cost: 0.1,
            },
            master: MasterParams {
                k_code: 1.0, tau_transfer: vec![0.1, 0.05, 0.08],
                upsilon_apply: 0.9,
            },
            self_repair: SelfRepairParams {
                g_target: 100.0, g_actual: 95.0,
                delta_error_locate: 1.0, psi_thorough_fix: 1.0,
                kappa_no_repeat: 1.0,
            },
            cycle: CycleParams { eta_skill_up: 0.5, rho_result_feedback: 0.5 },
            host: HostHealthParams { psi_mem: 0.98, psi_app: 0.99, psi_disk: 0.97, omega_dawn: 1.0 },
        }
    }

    #[test]
    fn test_llm_agent() {
        let p = LlmAgentParams { lambda_single_call: 0.9, mu_multi_task: 0.8, sigma_high_quality: 0.85, gamma_llm_cost: 0.1 };
        let r = calculate_llm_agent_efficiency(&p);
        assert!((r - 0.556).abs() < 0.001);
    }

    #[test]
    fn test_k_master() {
        let p = MasterParams { k_code: 1.0, tau_transfer: vec![0.1, 0.05, 0.08], upsilon_apply: 0.9 };
        let r = calculate_k_master(&p);
        assert!((r - 1.107).abs() < 0.001);
    }

    #[test]
    fn test_self_repair() {
        let p = SelfRepairParams { g_target: 100.0, g_actual: 80.0, delta_error_locate: 1.5, psi_thorough_fix: 1.2, kappa_no_repeat: 1.1 };
        let r = calculate_self_repair(&p);
        assert!((r - 1.495).abs() < 0.001);
    }

    #[test]
    fn test_cycle_gain() {
        let p = CycleParams { eta_skill_up: 0.5, rho_result_feedback: 0.5 };
        let r = calculate_cycle_gain(&p);
        assert!((r - 1.284).abs() < 0.001);
    }

    #[test]
    fn test_delta_g_ultimate() {
        let params = default_v8_params();
        let r = calculate_delta_g_ultimate(&params).unwrap();
        assert!(r > 0.0);
    }

    #[test]
    fn test_k_master_safe() {
        let p = MasterParams { k_code: 1.0, tau_transfer: vec![0.999], upsilon_apply: 1.0 };
        let safe = calculate_k_master_safe(&p);
        assert!((safe - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_cycle_gain_safe() {
        let p = CycleParams { eta_skill_up: 10.0, rho_result_feedback: 10.0 };
        let safe = calculate_cycle_gain_safe(&p);
        assert!((safe - 1096.633).abs() < 0.1);
    }

    #[test]
    fn test_phi_network() {
        let r = calc_phi_network(0.05, 0.02, 0.99);
        assert!((r - 0.921).abs() < 0.001);
    }

    #[test]
    fn test_gamma_mutation_hollow() {
        let r = calc_gamma_mutation(0.01, 0.05);
        assert!((r - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_omega_session() {
        let r = calc_omega_session(0.02, 0.01, 0.95);
        assert!((r - 0.922).abs() < 0.001);
    }

    #[test]
    fn test_pi_coord() {
        let r = calc_pi_coord(9, 10, 0.05, 0.9);
        assert!((r - 0.7695).abs() < 0.001);
    }

    #[test]
    fn test_omega_self() {
        let p = SelfAwarenessParams { sigma_coherence: 0.9, delta_drift: 0.1, rho_alignment: 0.85 };
        let r = calculate_omega_self(&p);
        assert!((r - 0.6885).abs() < 0.001);
    }

    #[test]
    fn test_sigma_memory() {
        let params = SuperMemoryParams::default();
        let r = calculate_sigma_memory(&params);
        assert!(r >= 0.0);
    }

    #[test]
    fn test_tau_trace() {
        let params = TraceParams::default();
        let r = calculate_tau_trace(&params);
        assert!((r - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_dawn_omega() {
        let params = DawnParams {
            omega_dawn: 1.0,
            git_sync: GitSyncParams { delta_version_diff: 0.0, rho_sync_fail: 0.0, tau_auto_merge: 1.0 },
            auto_learn: AutoLearnParams { l_extract: 1.0, g_generalize: 1.0, s_summarize: 1.0, t_time: 0.0 },
        };
        let r = calculate_dawn_omega(&params);
        assert!((r - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_v10_full() {
        let params = default_v8_params();
        let result = calculate_v10_full(&params, None);
        assert!(result.delta_g > 0.0);
        assert_eq!(result.version, "V10.1");
    }
}
