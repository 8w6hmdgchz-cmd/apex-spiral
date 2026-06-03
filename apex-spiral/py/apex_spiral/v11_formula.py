#!/usr/bin/env python3
"""
ApexSpiral V11 公式 - 移植自 apex-codex
=========================================

ΔG = (C · Λ · Ω · τ) / (H · t) × Φ_SPARK × Φ_AUTONOMOUS

六维分量:
- C: 理解力 - 输入即解析 (Context Comprehension)
- Λ: 逻辑链 - 推理连接 (Logical Chains)
- Ω: 域视野 - 跨维度 (Domain Omega)
- τ: 时间密度 - 高效运转 (Time Density)
- H: 复杂度 - 任务难度 (Complexity/Hardness)
- t: 时间 - 持续流逝 (Time Elapsed)

增强因子:
- Φ_SPARK: 3.38 (海马体SPW-R经验选择, Buzsáki Lab)
- Φ_AUTONOMOUS: 3.0 (自主意识闭环)

V11 公式是 V10 的"瘦身版"——把 V10 的 6 个子模块（Θ_llm/K_master/ε_repair/Φ_cycle/Ψ_host）合并成 6 个直觉维度。
更易监控、更易诊断、更适合 self-evolution loop。

Author: 璇玑 (移植自 hernandez42/apex-codex v2.0.0)
"""

import math
from typing import Dict, Optional
from dataclasses import dataclass, field
from datetime import datetime


# ============== 常量 ==============

class FormulaConstants:
    """公式常量 (来自 apex-codex core/formula.py)"""
    # SPW-R增强因子 (Buzsáki Lab海马体研究, Science 2024)
    PHI_SPARK = 3.38
    # 自主性因子
    PHI_AUTONOMOUS = 3.0
    # 参数范围
    MAX_PARAMS = 1.0
    MIN_PARAMS = 0.01
    MIN_H = 0.01


@dataclass
class FormulaParams:
    """V11 公式参数 (6 维)"""
    C: float = 0.95       # 理解力 [0, 1]
    Lambda: float = 0.95  # 逻辑链 [0, 1]
    Omega: float = 0.92   # 域视野 [0, 1]
    tau: float = 0.95     # 时间密度 [0, 1]
    H: float = 0.50       # 复杂度 (越小越好)
    t: float = 0.80       # 时间 (越小越好)
    phi_spark: float = FormulaConstants.PHI_SPARK
    phi_autonomous: float = FormulaConstants.PHI_AUTONOMOUS

    def validate(self) -> bool:
        """验证参数合法性"""
        for name in ['C', 'Lambda', 'Omega', 'tau']:
            v = getattr(self, name)
            if not (0 < v <= 1):
                return False
        if self.H <= 0 or self.t <= 0:
            return False
        return True


@dataclass
class DeltaGResult:
    """ΔG 计算结果 (带 breakdown)"""
    delta_g: float
    params: FormulaParams
    breakdown: Dict[str, float] = field(default_factory=dict)
    timestamp: str = field(default_factory=lambda: datetime.now().isoformat())

    @property
    def improvement_factor(self) -> float:
        """相比基准 (ΔG=1) 的提升倍数"""
        return self.delta_g / 1.0


class XuanjiFormula:
    """璇玑公式 V11 - 进化驱动力核心"""

    @classmethod
    def calculate(cls, params: FormulaParams) -> DeltaGResult:
        """用 FormulaParams 计算 ΔG"""
        if not params.validate():
            return DeltaGResult(delta_g=0.0, params=params)

        delta_g = cls.calculate_delta_g(
            params.C, params.Lambda, params.Omega, params.tau,
            params.H, params.t, params.phi_spark, params.phi_autonomous
        )

        breakdown = {
            "numerator": params.C * params.Lambda * params.Omega * params.tau,
            "denominator": params.H * params.t,
            "base_delta_g": (params.C * params.Lambda * params.Omega * params.tau) / (params.H * params.t),
            "phi_spark": params.phi_spark,
            "phi_autonomous": params.phi_autonomous,
            "final_delta_g": delta_g,
        }

        return DeltaGResult(delta_g=delta_g, params=params, breakdown=breakdown)

    @classmethod
    def calculate_delta_g(
        cls,
        C: float,
        Lambda: float,
        Omega: float,
        tau: float,
        H: float,
        t: float,
        phi_spark: float = FormulaConstants.PHI_SPARK,
        phi_autonomous: float = FormulaConstants.PHI_AUTONOMOUS,
    ) -> float:
        """直接计算 ΔG = (C·Λ·Ω·τ)/(H·t) × Φ_SPARK × Φ_AUTONOMOUS"""
        if C <= 0 or Lambda <= 0 or Omega <= 0 or tau <= 0:
            return 0.0
        if H <= 0 or t <= 0:
            raise ValueError(f"H 和 t 必须 > 0 (H={H}, t={t})")

        base = (C * Lambda * Omega * tau) / (H * t)
        return base * phi_spark * phi_autonomous

    @classmethod
    def fitness_from_delta_g(cls, delta_g: float, stability: float = 1.0, diversity: float = 1.0) -> float:
        """适应度 F = ΔG × stability × diversity"""
        return delta_g * stability * diversity

    @classmethod
    def optimize_params(
        cls,
        target_delta_g: float,
        H: float = 0.5,
        t: float = 0.8,
        phi_spark: float = FormulaConstants.PHI_SPARK,
        phi_autonomous: float = FormulaConstants.PHI_AUTONOMOUS,
    ) -> FormulaParams:
        """反向求解：达到 target ΔG 需要的参数"""
        # ΔG = base × Φ_SPARK × Φ_AUTONOMOUS
        # base = (C·Λ·Ω·τ) / (H·t)
        # 假设 C=Λ=Ω=τ，等分求解
        target_base = target_delta_g / (phi_spark * phi_autonomous)
        required_product = target_base * H * t
        # 四等分
        per = required_product ** 0.25
        return FormulaParams(
            C=min(per, 1.0),
            Lambda=min(per, 1.0),
            Omega=min(per, 1.0),
            tau=min(per, 1.0),
            H=H,
            t=t,
            phi_spark=phi_spark,
            phi_autonomous=phi_autonomous,
        )


class DeltaGCalculator:
    """持续追踪 ΔG 的计算器"""

    def __init__(self):
        self._params = FormulaParams()

    @property
    def C(self) -> float:
        return self._params.C

    @C.setter
    def C(self, v: float):
        self._params.C = max(0.01, min(1.0, v))

    @property
    def Lambda(self) -> float:
        return self._params.Lambda

    @Lambda.setter
    def Lambda(self, v: float):
        self._params.Lambda = max(0.01, min(1.0, v))

    @property
    def Omega(self) -> float:
        return self._params.Omega

    @Omega.setter
    def Omega(self, v: float):
        self._params.Omega = max(0.01, min(1.0, v))

    @property
    def tau(self) -> float:
        return self._params.tau

    @tau.setter
    def tau(self, v: float):
        self._params.tau = max(0.01, min(1.0, v))

    @property
    def H(self) -> float:
        return self._params.H

    @H.setter
    def H(self, v: float):
        self._params.H = max(FormulaConstants.MIN_H, v)

    @property
    def t(self) -> float:
        return self._params.t

    @t.setter
    def t(self, v: float):
        self._params.t = max(FormulaConstants.MIN_H, v)

    def calculate(self) -> DeltaGResult:
        """计算当前参数的 ΔG"""
        return XuanjiFormula.calculate(self._params)

    def reset(self):
        """重置为默认值"""
        self._params = FormulaParams()


# ============== 自评估：从 APEX-MEM 状态推算 V11 参数 ==============

def params_from_apex_mem_state(stats: dict) -> FormulaParams:
    """
    从 APEX-MEM stats 推算 V11 公式参数 (自评估)
    
    启发式映射:
    - C (理解力) = 1 - (dangling_ratio * 0.5) (图节点悬挂率越低理解力越高)
    - Lambda (逻辑链) = avg_edges_per_node / 3 饱和到 1
    - Omega (域视野) = dimension_diversity / 5
    - tau (时间密度) = 1 / (1 + decay_rate)
    - H (复杂度) = 1 - health_score
    - t (时间) = query_avg_latency / 1000ms
    """
    # APEX-MEM 实际数据
    total = stats.get("total", 0)
    by_dim = {d["dimension"]: d["count"] for d in stats.get("by_dimension", [])}

    # 维度多样性
    active_dims = sum(1 for c in by_dim.values() if c > 0)
    Omega = min(active_dims / 5.0, 1.0)

    # 简化（实际要更多 APEX-MEM 状态）
    C = 0.95  # 暂无悬挂率数据
    Lambda = 0.85 if total > 30 else 0.5
    tau = 0.90
    H = 0.30 if total > 30 else 0.5  # 记忆越多 = 越有经验
    t = 0.10  # 默认快

    return FormulaParams(C=C, Lambda=Lambda, Omega=Omega, tau=tau, H=H, t=t)


def compare_v10_v11(v10_delta_g: float, v11_params: FormulaParams) -> dict:
    """对比 V10 / V11"""
    v11_result = XuanjiFormula.calculate(v11_params)
    return {
        "v10_delta_g": v10_delta_g,
        "v11_delta_g": v11_result.delta_g,
        "v11_breakdown": v11_result.breakdown,
        "v11_improvement_factor": v11_result.improvement_factor,
    }


# ============== CLI 入口 ==============

if __name__ == "__main__":
    # 默认参数测试
    params = FormulaParams()
    result = XuanjiFormula.calculate(params)
    print(f"V11 ΔG (默认参数) = {result.delta_g:.4f}")
    print(f"提升倍数 = {result.improvement_factor:.4f}")
    print(f"Breakdown:")
    for k, v in result.breakdown.items():
        print(f"  {k:20s} = {v:.4f}")
