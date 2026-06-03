#!/usr/bin/env python3
"""APEX V11 self_issue() — 自主出题闭环 (Φ_AUTONOMOUS 实化)
从 V11 公式 8 参数里找短板, 生成可独立验证的小问题.
"""
from dataclasses import dataclass
from typing import List, Dict

@dataclass
class Bottleneck:
    name: str
    current: float
    target: float
    delta_g_gain: float
    question: str
    verify: str

def find_bottlenecks(params: Dict[str, float]) -> List[Bottleneck]:
    """基于 V11 ΔG=(C·Λ·Ω·τ)/(H·t)×Φ_SPARK×Φ_AUTONOMOUS 找短板."""
    C, L, O, T, H, t = params['C'], params['Lambda'], params['Omega'], params['tau'], params['H'], params['t']
    base = (C*L*O*T)/(H*t)
    problems = []
    # 找参数里离 1.0 最远的 (除 H/t 越低越好)
    targets = {
        'C': (C, 0.99, "C(理解力) < 0.95: 输入有歧义"),
        'Lambda': (L, 0.95, "Λ(逻辑链) < 0.90: 推理缺中间步"),
        'Omega': (O, 0.90, "Ω(域视野) < 0.80: 跨域知识缺"),
        'tau': (T, 0.99, "τ(时间密度) < 0.95: 任务密度低"),
        'H': (H, 0.30, "H(噪声) > 0.35: 输出 35%+ 是空话"),
        't': (t, 0.15, "t(时间浪费) > 0.20: 1h 任务实际做 12min"),
    }
    for name, (cur, tgt, desc) in targets.items():
        if name in ('H', 't'):
            if cur > tgt:
                problems.append(Bottleneck(name, cur, tgt, 0, desc, f"降 {name} 到 {tgt}"))
        else:
            if cur < tgt:
                problems.append(Bottleneck(name, cur, tgt, 0, desc, f"提 {name} 到 {tgt}"))
    return sorted(problems, key=lambda b: b.current)

def self_issue(round_id: str = "R41") -> List[Dict]:
    """主入口: 给当前 round 出 5 道题."""
    params = {'C': 0.85, 'Lambda': 0.82, 'Omega': 0.60, 'tau': 0.92, 'H': 0.45, 't': 0.22}
    bns = find_bottlenecks(params)[:5]
    return [{"round": round_id, "param": b.name, "cur": b.current, "tgt": b.target, "q": b.question, "fix": b.verify} for b in bns]

if __name__ == "__main__":
    import json
    print(json.dumps(self_issue(), indent=2, ensure_ascii=False))
