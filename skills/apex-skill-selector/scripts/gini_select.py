#!/usr/bin/env python3
"""Gini 选择算法 — APEX Skill Selector 核心"""
import json, sys, math

def gini(probabilities: list[float]) -> float:
    """Gini 不纯度: Gini = 1 - Σp_k²"""
    return 1.0 - sum(p**2 for p in probabilities if p > 0)

def delta_gini(parent_probs: list[float], left_probs: list[float], right_probs: list[float]) -> float:
    """ΔGini = Gini_parent - (N_L/N × Gini_L + N_R/N × Gini_R)"""
    n = sum(parent_probs)
    if n == 0:
        return 0.0
    g_parent = gini(parent_probs)
    g_left  = gini(left_probs)
    g_right = gini(right_probs)
    n_l = sum(left_probs)
    n_r = sum(right_probs)
    return g_parent - (n_l / n * g_left + n_r / n * g_right)

def score_skill(skill: dict, base_gini: float = 0.5) -> float:
    """基于 fitness + delta_gini 计算综合得分"""
    fitness = skill.get("fitness", 0.5)
    score   = skill.get("score",   0.5)
    tags    = skill.get("tags",    [])
    # 归一化 tag 权重
    tag_bonus = min(len(tags) * 0.05, 0.25)
    return (fitness * 0.6 + score * 0.3 + tag_bonus)

def gini_select(candidates: list[dict]) -> dict:
    """
    输入: [{"name": str, "score": float, "fitness": float, "tags": [str]}]
    输出: {"selected": str, "confidence": float, "delta_gini": float, "all_scores": []}
    """
    if not candidates:
        return {"selected": None, "confidence": 0.0, "delta_gini": 0.0, "all_scores": []}

    base_gini = 0.5  # 初始不纯度
    scored = []
    for s in candidates:
        f    = s.get("fitness", 0.5)
        sc   = s.get("score",   0.5)
        p    = [f, sc, 1.0 - f - sc]  # 构造概率分布
        p    = [max(x, 0.0) for x in p]
        g    = gini(p)
        dg   = max(base_gini - g, 0.0)
        conf = (f * 0.6 + sc * 0.4) * (1.0 + dg)
        scored.append({
            "name":       s["name"],
            "score":      sc,
            "fitness":    f,
            "gini":       g,
            "delta_gini": dg,
            "confidence": min(conf, 1.0),
        })

    scored.sort(key=lambda x: x["delta_gini"], reverse=True)
    best = scored[0]
    return {
        "selected":   best["name"],
        "confidence": round(best["confidence"], 4),
        "delta_gini": round(best["delta_gini"], 4),
        "all_scores": scored,
    }

if __name__ == "__main__":
    data = json.load(sys.stdin)
    result = gini_select(data.get("candidates", []))
    print(json.dumps(result, indent=2, ensure_ascii=False))
