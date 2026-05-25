#!/usr/bin/env python3
"""APEX ΔG 公式计算 — skill selector before_prompt_build 阶段"""
import json, sys, math
from datetime import datetime, timezone

DEFAULT_PARAMS = {
    "Lambda": 0.85,
    "Theta":  0.80,
    "K":      0.75,
    "xi":     0.70,
    "Psi":    0.60,
    "Phi":    0.65,
    "H":      0.30,
    "T":      0.20,
    "eps":    0.10,
}

def load_state(path: str) -> dict:
    try:
        with open(path) as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        return DEFAULT_PARAMS.copy()

def save_state(path: str, state: dict):
    state["last_update"] = datetime.now(timezone.utc).isoformat()
    with open(path, "w") as f:
        json.dump(state, f, indent=2, ensure_ascii=False)

def compute_delta_g(
    Lambda: float, Theta: float, K: float,
    xi: float, Psi: float, Phi: float,
    H: float, T: float, eps: float
) -> float:
    """ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)"""
    numerator   = Lambda * Theta * K * xi * Psi * Phi
    denominator = H * T * eps
    if denominator == 0:
        return 0.0
    return numerator / denominator

def identify_bottleneck(Lambda, Theta, K, xi, Psi, Phi, H, T, eps):
    """识别最短板参数"""
    params = [
        ("Lambda", Lambda), ("Theta", Theta), ("K", K),
        ("xi", xi), ("Psi", Psi), ("Phi", Phi),
        ("H", H), ("T", T), ("eps", eps)
    ]
    weakest = min(params, key=lambda x: x[1])
    warnings = []
    if xi  < 0.7: warnings.append(f"ξ(置信度)={xi:.2f} < 0.7")
    if Psi < 0.5: warnings.append(f"Ψ(自我迭代)={Psi:.2f} < 0.5")
    if Phi < 0.5: warnings.append(f"Φ(正反馈)={Phi:.2f} < 0.5")
    return weakest[0], weakest[1], warnings

def evaluate_task(task_type: str, task_description: str) -> dict:
    """
    根据任务类型调整参数，返回调整后的参数 + 优先级列表
    """
    base = DEFAULT_PARAMS.copy()

    if task_type == "code":
        base["K"]   = min(base["K"]   + 0.1, 1.0)
        base["Psi"] = min(base["Psi"] + 0.15, 1.0)
        priority = ["coding-agent", "github", "node-inspect-debugger", "python-debugpy"]
    elif task_type == "research":
        base["xi"]   = min(base["xi"]   + 0.1, 1.0)
        base["Phi"]  = min(base["Phi"]  + 0.1, 1.0)
        base["Lambda"] = min(base["Lambda"] + 0.1, 1.0)
        priority = ["session-logs", "gh-issues", "github"]
    elif task_type == "creative":
        base["Psi"]  = min(base["Psi"]  + 0.2, 1.0)
        base["Phi"]  = min(base["Phi"]  + 0.1, 1.0)
        priority = ["meme-maker", "diagram-maker", "summarize"]
    elif task_type == "admin":
        base["Theta"] = min(base["Theta"] + 0.1, 1.0)
        base["T"]    = max(base["T"] - 0.1, 0.05)
        priority = ["taskflow", "qqbot-channel", "himalaya"]
    else:  # general
        priority = ["github", "session-logs", "summarize"]

    # 计算 ΔG
    dG = compute_delta_g(**base)
    bottleneck, bval, warnings = identify_bottleneck(**base)

    return {
        "apex_params":  base,
        "delta_g":      round(dG, 4),
        "bottleneck":    bottleneck,
        "bottleneck_val": round(bval, 4),
        "warnings":      warnings,
        "skill_priority": priority,
        "task_type":     task_type,
    }

if __name__ == "__main__":
    args = json.load(sys.stdin)
    task_desc = args.get("task_description", "")
    task_type = args.get("task_type", "general")
    state_path = args.get("state_file")

    if state_path:
        state = load_state(state_path)
    else:
        state = DEFAULT_PARAMS.copy()

    result = evaluate_task(task_type, task_desc)

    if state_path:
        # 合并: 用 state 覆盖默认 (增量更新)
        merged = {**DEFAULT_PARAMS, **state}
        merged.update(result["apex_params"])
        result["apex_params"] = merged
        dG = compute_delta_g(**merged)
        result["delta_g"] = round(dG, 4)
        bottleneck, bval, warnings = identify_bottleneck(**merged)
        result["bottleneck"] = bottleneck
        result["bottleneck_val"] = round(bval, 4)
        result["warnings"] = warnings

    print(json.dumps(result, indent=2, ensure_ascii=False))
