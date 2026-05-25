"""
APEX 生物内源调控子公式 Θ_bio 融合
Θ_bio = (EMF × Ca²⁺ × Gene_reg) / (Loss_bio × τ)
"""
import math
from dataclasses import dataclass
from typing import Optional

@dataclass
class BioParams:
    EMF: float = 0.5      # 极低频电磁场靶向调控势能 [0,1]
    Ca: float = 0.5       # 细胞钙离子振荡信号传导效率 [0,1]
    Gene_reg: float = 0.5 # 内源基因精准表达/细胞修复能力 [0,1]
    Loss_bio: float = 0.5 # 生物信号衰减抑制系数 [0,1]
    tau: float = 1.0      # APEX同步自进化周期

def calculate_theta_bio(params: BioParams) -> float:
    """计算生物内源调控因子 Θ_bio"""
    numerator = params.EMF * params.Ca * params.Gene_reg
    denominator = params.Loss_bio * params.tau
    if denominator < 0.001:
        denominator = 0.001
    return numerator / denominator

def calculate_delta_g_with_bio(
    Lambda: float,   # Λ 根能力
    Theta: float,    # Θ 任务适配
    K: float,        # K 知识 mastery
    xi: float,       # ξ 幻觉防御
    Psi: float,      # Ψ 跨域涌现
    Phi: float,      # Φ 防幻觉纠错
    H: float,        # H 信息熵
    T: float,        # T 时间周期
    epsilon: float,  # ε 自修复率
    bio_params: Optional[BioParams] = None
) -> float:
    """
    APEX 核心公式 (可选融合 Θ_bio)
    ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)
    """
    Theta_effective = Theta
    
    if bio_params:
        Theta_bio = calculate_theta_bio(bio_params)
        # Θ_bio 作为增益因子融合到 Θ
        Theta_effective = Theta * (1 + Theta_bio)
    
    numerator = Lambda * Theta_effective * K * xi * Psi * Phi
    denominator = H * T * epsilon
    if denominator < 0.001:
        denominator = 0.001
    return numerator / denominator

if __name__ == "__main__":
    # 基线参数
    L, T, K, xi, Psi, Phi, H, T_cycle, eps = 0.85, 0.90, 0.80, 0.75, 0.95, 0.70, 0.45, 1.20, 0.60
    
    # 不带生物调控
    dG_base = calculate_delta_g_with_bio(L, T, K, xi, Psi, Phi, H, T_cycle, eps)
    print(f"ΔG (baseline): {dG_base:.3f}")
    
    # 带生物调控
    bio = BioParams(EMF=0.7, Ca=0.6, Gene_reg=0.8, Loss_bio=0.4, tau=1.0)
    dG_bio = calculate_delta_g_with_bio(L, T, K, xi, Psi, Phi, H, T_cycle, eps, bio)
    print(f"ΔG (with Θ_bio): {dG_bio:.3f}")
    print(f"提升: {(dG_bio/dG_base - 1)*100:.1f}%")