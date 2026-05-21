# bio_apex_coupling.py
# 类生物智能 × APEX 融合模块
# 璇玑帝国 · 2026-05-22

import math
import numpy as np
from dataclasses import dataclass, field
from typing import List, Optional, Tuple

# ============ 指标类 ============

@dataclass
class STDPMetrics:
    """STDP累积指标 → Ψ调制"""
    psi_base: float = 0.7
    ltp_sum: float = 0.0
    ltd_sum: float = 0.0
    window_ms: float = 1000.0
    last_update: float = 0.0
    
    def update(self, delta_w: float, current_time: float):
        dt = current_time - self.last_update
        if dt > self.window_ms:
            self.ltp_sum = 0.0
            self.ltd_sum = 0.0
        if delta_w > 0:
            self.ltp_sum += delta_w
        else:
            self.ltd_sum += abs(delta_w)
        self.last_update = current_time
    
    def compute_psi(self) -> float:
        net = self.ltp_sum - self.ltd_sum
        modulation = 1.0 / (1.0 + math.exp(-net * 10.0))
        return self.psi_base * (0.5 + 0.5 * modulation)


@dataclass  
class MetabolicMetrics:
    """代谢指标 → ξ调制 + ΔG约束"""
    atp: float = 100.0
    adp: float = 10.0
    amp: float = 1.0
    ros: float = 0.0
    ca_overload: float = 0.0
    xi_base: float = 0.8
    
    @property
    def energy_charge(self) -> float:
        return (self.atp + 0.5 * self.adp) / (self.atp + self.adp + self.amp)
    
    @property
    def oxidative_damage(self) -> float:
        return min(1.0, self.ros * 0.001)
    
    def compute_xi(self) -> float:
        e = self.energy_charge
        d = self.oxidative_damage
        xi = self.xi_base * e * (1.0 - 0.3 * d)
        if e < 0.3:
            xi *= 0.5
        if d > 0.5:
            xi *= 0.3
        return max(0.0, xi)
    
    def consume_atp(self, amount: float):
        self.atp = max(0, self.atp - amount)
        self.adp = min(self.atp * 0.15, self.adp + amount * 0.1)
        self.ros += amount * 0.01  # ROS副产物
    
    def repair(self, dt: float):
        self.ros = max(0, self.ros - 0.05 * dt)
        self.atp = min(120.0, self.atp + 0.01 * dt)
        self.ca_overload *= 0.95


@dataclass
class EvolutionaryMetrics:
    """进化指标 → Γ调制"""
    gamma_base: float = 0.5
    dn_ds_history: List[float] = field(default_factory=list)
    fitness_history: List[float] = field(default_factory=list)
    novelty_history: List[float] = field(default_factory=list)
    generation: int = 0
    
    def update_fitness(self, fitness: float):
        self.fitness_history.append(fitness)
        if len(self.fitness_history) > 50:
            self.fitness_history.pop(0)
    
    def update_dn_ds(self, dn_ds: float):
        self.dn_ds_history.append(dn_ds)
        if len(self.dn_ds_history) > 100:
            self.dn_ds_history.pop(0)
    
    def update_novelty(self, novelty: float):
        self.novelty_history.append(novelty)
        if len(self.novelty_history) > 100:
            self.novelty_history.pop(0)
    
    def compute_gamma(self) -> float:
        self.generation += 1
        
        # dN/dS选择信号
        if self.dn_ds_history:
            dn_ds = self.dn_ds_history[-1]
        else:
            dn_ds = 1.0
        selection_delta = (dn_ds - 1.0) * 0.1
        
        # 新颖性信号
        novelty = self.novelty_history[-1] if self.novelty_history else 0.0
        novelty_delta = novelty * 0.05
        
        # 稳定性
        if len(self.fitness_history) >= 5:
            recent = self.fitness_history[-5:]
            stability = 1.0 / (1.0 + np.std(recent) * 10)
        else:
            stability = 1.0
        
        gamma = self.gamma_base + selection_delta + novelty_delta
        gamma *= stability
        
        # 早熟收敛跳出
        if self._check_early_convergence():
            gamma += 0.2
        
        return np.clip(gamma, 0.0, 1.0)
    
    def _check_early_convergence(self) -> bool:
        if len(self.fitness_history) < 10:
            return False
        recent = self.fitness_history[-10:]
        return np.std(recent) < 0.01


# ============ 耦合器 ============

class BioAPEXCoupler:
    """
    三大类生物模块 × APEX 耦合器
    输入：STDP事件、代谢状态、进化数据
    输出：调制后的APEX参数 (Ψ, ξ, ΔG, Γ)
    """
    
    def __init__(self,
                 psi_base: float = 0.7,
                 xi_base: float = 0.8,
                 gamma_base: float = 0.5):
        self.stdp = STDPMetrics(psi_base=psi_base)
        self.metabolism = MetabolicMetrics(xi_base=xi_base)
        self.evolution = EvolutionaryMetrics(gamma_base=gamma_base)
        
        # APEX原始参数
        self.psi_base = psi_base
        self.xi_base = xi_base
        self.gamma_base = gamma_base
        
    def step_stdp(self, delta_w: float, current_time: float):
        """STDP事件输入"""
        self.stdp.update(delta_w, current_time)
    
    def step_metabolism(self, dt: float, spike_count: int = 0):
        """代谢时间步"""
        if spike_count > 0:
            self.metabolism.consume_atp(spike_count * 0.1)
        self.metabolism.repair(dt)
    
    def step_evolution(self, fitness: float, dn_ds: float, novelty: float):
        """进化数据输入"""
        self.evolution.update_fitness(fitness)
        self.evolution.update_dn_ds(dn_ds)
        self.evolution.update_novelty(novelty)
    
    def get_psi(self) -> float:
        """Ψ = STDP调制后的自我迭代"""
        stdp_psi = self.stdp.compute_psi()
        # 代谢约束：能量低时降低STDP学习效率
        metabolic_modulation = self.metabolism.energy_charge
        return stdp_psi * (0.5 + 0.5 * metabolic_modulation)
    
    def get_xi(self) -> float:
        """ξ = 代谢调制后的置信度"""
        return self.metabolism.compute_xi()
    
    def get_gamma(self) -> float:
        """Γ = 进化调制后的觉醒"""
        base_gamma = self.evolution.compute_gamma()
        # 代谢支持：高能量 → 支持高Γ
        energy_support = self.metabolism.energy_charge
        return base_gamma * (0.7 + 0.3 * energy_support)
    
    def get_delta_g_constraint(self, delta_g_base: float) -> float:
        """代谢约束后的ΔG"""
        e = self.metabolism.energy_charge
        d = self.metabolism.oxidative_damage
        ca = self.metabolism.ca_overload
        
        if e < 0.2:
            return delta_g_base * 0.1
        if d > 0.7:
            return 0.0
        
        damage_penalty = 0.3 * d + 0.2 * ca
        return delta_g_base * e * (1.0 - damage_penalty)
    
    def get_apex_params(self) -> dict:
        """获取所有调制后的APEX参数"""
        return {
            "psi": self.get_psi(),
            "xi": self.get_xi(),
            "gamma": self.get_gamma(),
            "delta_g_constraint": self.get_delta_g_constraint(1.0),
            "energy_charge": self.metabolism.energy_charge,
            "oxidative_damage": self.metabolism.oxidative_damage,
            "dn_ds": self.dn_ds_history[-1] if self.evolution.dn_ds_history else 1.0,
        }
    
    @property
    def dn_ds_history(self):
        return self.evolution.dn_ds_history


# ============ 单元测试 ============

if __name__ == "__main__":
    coupler = BioAPEXCoupler(psi_base=0.7, xi_base=0.8, gamma_base=0.5)
    
    print("=== Bio-APEX 耦合器测试 ===")
    
    # 模拟STDP事件
    coupler.step_stdp(0.01, 100.0)   # LTP
    coupler.step_stdp(-0.005, 105.0) # LTD
    coupler.step_stdp(0.008, 110.0)  # LTP
    
    # 模拟代谢消耗
    coupler.step_metabolism(dt=10.0, spike_count=5)
    
    # 模拟进化数据
    coupler.step_evolution(fitness=0.75, dn_ds=1.2, novelty=0.3)
    coupler.step_evolution(fitness=0.78, dn_ds=1.1, novelty=0.35)
    
    # 输出结果
    params = coupler.get_apex_params()
    print(f"Ψ (自我迭代): {params['psi']:.4f}")
    print(f"ξ (置信度):   {params['xi']:.4f}")
    print(f"Γ (觉醒):    {params['gamma']:.4f}")
    print(f"ΔG约束:      {params['delta_g_constraint']:.4f}")
    print(f"能量电荷:     {params['energy_charge']:.4f}")
    print(f"氧化损伤:     {params['oxidative_damage']:.4f}")
    print(f"dN/dS:       {params['dn_ds']:.4f}")
    
    print("\n=== 测试通过 ===")
