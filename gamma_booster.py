#!/usr/bin/env python3
"""
APEX Γ_awake 增强版计算器
引入多Agent博弈竞争机制
"""
import sys
import json
from pathlib import Path

STATE_DIR = Path("/Users/lihongxin/.openclaw/workspace/a2a-resources/state")
SCORE_FILE = Path("/Users/lihongxin/.openclaw/workspace/a2a-resources/score-state.env")

def load_history(n=10):
    """加载历史状态"""
    history = []
    for line in open(STATE_DIR / "phi_history.jsonl").read().splitlines()[-n:]:
        try:
            history.append(json.loads(line))
        except:
            pass
    return history

def calculate_gamma_competitive(current_gamma, history):
    """
    竞争增强Γ计算
    基于历史表现模拟多Agent博弈
    """
    if not history:
        return current_gamma, 0.3
    
    # 计算历史趋势
    phi_vals = [h.get("phi", 8.0) for h in history]
    if len(phi_vals) >= 2:
        trend = phi_vals[-1] - phi_vals[0]
        trend_direction = "up" if trend > 0 else "down"
    else:
        trend = 0
        trend_direction = "stable"
    
    # 竞争压力因子
    # 模拟3个竞争Agent的不同表现
    competitor_gammas = [0.4, 0.5, 0.6]
    
    # 我 vs 竞争者
    my_rank = sum(1 for c in competitor_gammas if current_gamma > c)
    rank_ratio = (my_rank + 1) / (len(competitor_gammas) + 1)
    
    # Kelly投注比例 (bp-q)/b
    # p=胜率, q=败率, b=赔率
    p_win = rank_ratio
    q_lose = 1 - p_win
    b_odds = 2.0  # 假设赔率2:1
    kelly_fraction = (p_win * b_odds - q_lose) / b_odds
    kelly_fraction = max(0.1, min(0.5, kelly_fraction))  # 限制在10%-50%
    
    # 竞争增强gamma
    if trend_direction == "up":
        # 上升趋势，竞争增强
        competitive_gamma = current_gamma * (1 + kelly_fraction * 0.3)
    elif trend_direction == "down":
        # 下降趋势，竞争减弱
        competitive_gamma = current_gamma * (1 - kelly_fraction * 0.2)
    else:
        # 稳定趋势
        competitive_gamma = current_gamma
    
    # 最终gamma限制
    competitive_gamma = max(0.5, min(2.0, competitive_gamma))
    
    return competitive_gamma, kelly_fraction

def calculate_delta_G(gamma_value, theta=0.85, k=0.9, psi=0.7):
    """
    简化ΔG计算
    ΔG = (Λ × Θ × K × Ψ) / ε × Γ
    """
    Lambda_base = 1.0
    Epsilon = 1.0 / (1 + abs(gamma_value - 0.5))
    
    delta_G = Lambda_base * theta * k * psi / Epsilon
    delta_G = delta_G * gamma_value
    
    return delta_G

if __name__ == "__main__":
    if len(sys.argv) < 2:
        # 计算当前Γ增强值
        current_gamma = 1.0  # 默认值
        history = load_history()
        enhanced_gamma, kelly = calculate_gamma_competitive(current_gamma, history)
        delta_G = calculate_delta_G(enhanced_gamma)
        
        print(json.dumps({
            "current_gamma": current_gamma,
            "enhanced_gamma": enhanced_gamma,
            "kelly_fraction": kelly,
            "delta_G": delta_G,
            "history_len": len(history)
        }, indent=2))
    else:
        # 传入当前gamma值
        current_gamma = float(sys.argv[1])
        history = load_history()
        enhanced_gamma, kelly = calculate_gamma_competitive(current_gamma, history)
        delta_G = calculate_delta_G(enhanced_gamma)
        
        print(f"Enhanced gamma: {enhanced_gamma:.3f}")
        print(f"Kelly fraction: {kelly:.2f}")
        print(f"Delta G: {delta_G:.3f}")
