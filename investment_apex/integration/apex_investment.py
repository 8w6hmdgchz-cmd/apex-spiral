#!/usr/bin/env python3
"""
APEX Investment Master System
============================
整合 APEX 自我进化 + 13位投资大师 Agent

APEX ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)
                    ↓
         投资决策适应度评估
                    ↓
         13位大师 Agent 协作
                    ↓
         动态策略调整
"""

import json
from typing import Dict, List, Optional
from dataclasses import dataclass

# 投资大师列表
INVESTMENT_MASTERS = [
    {"name": "Warren Buffett", "style": "价值投资", "weight": 0.15},
    {"name": "Charlie Munger", "style": "合理价格优质企业", "weight": 0.10},
    {"name": "Ben Graham", "style": "安全边际", "weight": 0.08},
    {"name": "Peter Lynch", "style": "增长投资", "weight": 0.08},
    {"name": "Cathie Wood", "style": "创新颠覆", "weight": 0.07},
    {"name": "Bill Ackman", "style": "激进维权", "weight": 0.06},
    {"name": "Stanley Druckenmiller", "style": "宏观对冲", "weight": 0.08},
    {"name": "Michael Burry", "style": "逆势价值", "weight": 0.06},
    {"name": "Nassim Taleb", "style": "尾部风险", "weight": 0.07},
    {"name": "Mohnish Pabrai", "style": "Dhandho低风险", "weight": 0.05},
    {"name": "Phil Fisher", "style": "成长股", "weight": 0.05},
    {"name": "Rakesh Jhunjhunwala", "style": "印度金牛", "weight": 0.05},
    {"name": "Aswath Damodaran", "style": "估值专家", "weight": 0.05},
]

# 分析 Agent
ANALYSIS_AGENTS = [
    {"name": "Valuation", "role": "内在价值计算"},
    {"name": "Sentiment", "role": "市场情绪"},
    {"name": "Fundamentals", "role": "基本面分析"},
    {"name": "Technicals", "role": "技术分析"},
    {"name": "Risk Manager", "role": "风险管理"},
    {"name": "Portfolio Manager", "role": "组合管理"},
]


@dataclass
class ApexInvestmentState:
    """APEX 投资状态"""
    delta_g: float = 1.0
    theta_llm: float = 0.8
    phi_cycle: float = 1.0
    master_weights: Dict[str, float] = None
    
    def __post_init__(self):
        if self.master_weights is None:
            self.master_weights = {m["name"]: m["weight"] for m in INVESTMENT_MASTERS}
    
    def update_from_signal(self, signal: Dict):
        """根据大师信号更新权重"""
        if "signal" in signal and "confidence" in signal:
            master = signal.get("master", "Unknown")
            confidence = signal["confidence"] / 100.0
            
            # APEX 增强：如果 ΔG 高，增强高信心信号的权重
            if self.delta_g > 0.9 and confidence > 0.7:
                self.master_weights[master] = min(
                    0.25, 
                    self.master_weights.get(master, 0.1) * 1.2
                )
            # 如果 ΔG 低，降低低信心信号的权重
            elif self.delta_g < 0.6 and confidence < 0.5:
                self.master_weights[master] = max(
                    0.01,
                    self.master_weights.get(master, 0.1) * 0.8
                )
            
            # 重新归一化
            total = sum(self.master_weights.values())
            for k in self.master_weights:
                self.master_weights[k] /= total


class ApexInvestmentEngine:
    """
    APEX 投资引擎
    =============
    将 APEX 公式融入投资决策
    """
    
    def __init__(self):
        self.state = ApexInvestmentState()
        self.signal_history = []
    
    def calculate_decision_fitness(self, signals: List[Dict]) -> float:
        """
        计算决策适应度
        ΔG_invest = Σ(signal_i × weight_i × confidence_i) / Σweight_i
        """
        if not signals:
            return 0.5
        
        weighted_sum = 0
        weight_sum = 0
        
        for sig in signals:
            master = sig.get("master", "Unknown")
            signal_val = 1.0 if sig.get("signal") == "bullish" else -0.5 if sig.get("signal") == "bearish" else 0
            confidence = sig.get("confidence", 50) / 100.0
            weight = self.state.master_weights.get(master, 0.05)
            
            weighted_sum += signal_val * confidence * weight
            weight_sum += weight
        
        base_fitness = weighted_sum / weight_sum if weight_sum > 0 else 0
        
        # APEX 增强：根据 Φ 循环因子调整
        adjusted_fitness = base_fitness * (1 + (self.state.phi_cycle - 1) * 0.1)
        
        return max(0, min(1, adjusted_fitness))
    
    def generate_master_consensus(self, signals: List[Dict]) -> Dict:
        """
        生成大师共识
        =============
        使用 APEX 权重聚合所有大师信号
        """
        consensus = {
            "signal": "neutral",
            "confidence": 50,
            "consensus_score": 0.5,
            "master_votes": {},
            "apex_state": {
                "delta_g": self.state.delta_g,
                "theta_llm": self.state.theta_llm,
                "phi_cycle": self.state.phi_cycle
            }
        }
        
        # 统计投票
        bullish_count = 0
        bearish_count = 0
        neutral_count = 0
        weighted_sum = 0
        weight_sum = 0
        
        for sig in signals:
            master = sig.get("master", "Unknown")
            signal = sig.get("signal", "neutral")
            confidence = sig.get("confidence", 50)
            weight = self.state.master_weights.get(master, 0.05)
            
            if signal == "bullish":
                bullish_count += 1
            elif signal == "bearish":
                bearish_count += 1
            else:
                neutral_count += 1
            
            weighted_sum += (1 if signal == "bullish" else -0.5 if signal == "bearish" else 0) * confidence * weight
            weight_sum += weight
            
            if master not in consensus["master_votes"]:
                consensus["master_votes"][master] = []
            consensus["master_votes"][master].append({
                "signal": signal,
                "confidence": confidence,
                "weight": weight
            })
        
        # 共识决策
        if bullish_count > bearish_count + neutral_count:
            consensus["signal"] = "bullish"
        elif bearish_count > bullish_count + neutral_count:
            consensus["signal"] = "bearish"
        
        consensus["confidence"] = abs(weighted_sum / weight_sum * 100) if weight_sum > 0 else 50
        consensus["consensus_score"] = weighted_sum / weight_sum if weight_sum > 0 else 0
        
        # 更新 APEX 状态
        self.state.delta_g = self.calculate_decision_fitness(signals)
        
        return consensus
    
    def evolve(self) -> Dict:
        """
        APEX 进化
        =========
        调整大师权重，优化决策适应度
        """
        old_delta_g = self.state.delta_g
        
        # Φ 循环增强
        self.state.phi_cycle *= 1.05
        
        # Θ LLM 效率提升
        self.state.theta_llm = min(0.99, self.state.theta_llm * 1.02)
        
        # 基于历史信号调整权重
        for sig in self.signal_history[-10:]:
            self.state.update_from_signal(sig)
        
        new_delta_g = self.calculate_decision_fitness(self.signal_history)
        
        return {
            "before_delta_g": old_delta_g,
            "after_delta_g": new_delta_g,
            "improvement": new_delta_g - old_delta_g,
            "phi_cycle": self.state.phi_cycle,
            "theta_llm": self.state.theta_llm
        }
    
    def add_signal(self, signal: Dict):
        """添加大师信号"""
        self.signal_history.append(signal)
        self.state.update_from_signal(signal)
    
    def get_master_weights(self) -> Dict:
        """获取当前大师权重"""
        return self.state.master_weights


# 全局实例
_engine = None

def get_investment_engine() -> ApexInvestmentEngine:
    global _engine
    if _engine is None:
        _engine = ApexInvestmentEngine()
    return _engine
