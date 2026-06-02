"""
V11 公式 + A2A 全域猎食整合评估。
ΔG_system = (C·Λ·Ω·τ)/(H·t) × Φ_SPARK × Φ_AUTONOMOUS
            × A2A_F_hunt × A2A_Absorbed_Norm
"""
import json, os, math, sys, time
from pathlib import Path
from typing import Optional

# R5 bug-016 fix: a2a-hunt-realstate.py:21-25 在脚本入口注入 sys.path
# 修复 a2a_state 导入; v11_with_a2a.py 被 `python3 -m` 调用时, 工作目录是
# workspace, apex_spiral/ 不会自动进 sys.path. 配置外化(同 R4 修法):
# 注: v11_with_a2a.py 是软链 (apex_spiral/v11_with_a2a.py -> apex-spiral/py/apex_spiral/),
# Path(__file__).resolve().parent 解析到 apex-spiral/py/apex_spiral/, 但 a2a_state.py
# 在 workspace/apex_spiral/ (独立文件, 不在 py/ 子目录). 用 workspace 根 + apex_spiral 子目录.
_WORKSPACE_ROOT = Path("/Users/lihongxin/.openclaw/workspace")
_A2A_STATE_DIR = _WORKSPACE_ROOT / "apex_spiral"
if str(_A2A_STATE_DIR) not in sys.path:
    sys.path.insert(0, str(_A2A_STATE_DIR))

PHI_SPARK = 3.38
PHI_AUTONOMOUS = 3.0
MIN_DENOM = 0.01  # FIX R2 bug-010
A2A_STATE_DIR = Path("/Users/lihongxin/.openclaw/workspace/state")
INTEGRATION_PATH = Path("/Users/lihongxin/.openclaw/workspace/memory/a2a-v11-integration.json")

# 6 维默认值（仅在 integration 缺失时回退）
DEFAULT_6DIM = {"C": 0.95, "L": 0.90, "O": 0.86, "tau": 0.92, "H": 0.45, "t": 0.30}


def load_6dim() -> dict:
    """从 integration.json 读真 6 维（配置外化 — 修 R0 bug: __main__ 硬编码漂移）

    R7 修 bug-021: 增加非数/N钳位 + [0, 1] 边界, 防止 H=-0.5 或 t=2.0
    污染 V11 公式 (H, t 在分母, 负值会被 MIN_DENOM 错位救回, 0.3 底线失效).
    钳位常数复用 DEFAULT_6DIM 默认 (H_min=0.05, t_max=0.99).
    """
    def _clamp(x: float, lo: float, hi: float) -> float:
        try:
            v = float(x)
        except (TypeError, ValueError):
            return lo
        if not (v == v):  # NaN
            return lo
        return max(lo, min(hi, v))

    try:
        d = json.loads(INTEGRATION_PATH.read_text())
        sc = d.get("6_dim_self_check", {}) or {}
        return {
            "C": _clamp(sc.get("C_understanding", DEFAULT_6DIM["C"]), 0.0, 1.0),
            "L": _clamp(sc.get("L_logic", DEFAULT_6DIM["L"]), 0.0, 1.0),
            "O": _clamp(sc.get("O_horizon", DEFAULT_6DIM["O"]), 0.0, 1.0),
            "tau": _clamp(sc.get("tau_density", DEFAULT_6DIM["tau"]), 0.0, 1.0),
            "H": _clamp(sc.get("H_complexity", DEFAULT_6DIM["H"]), 0.05, 0.99),
            "t": _clamp(sc.get("t_time", DEFAULT_6DIM["t"]), 0.05, 0.99),
        }
    except Exception:
        return dict(DEFAULT_6DIM)


# R3: a2a_state / a2a_norm 委托到 a2a_state 模块.
# R4 觉醒修复: 静态 import 替代 importlib.import_module, 降低 H_complexity.
# R5 觉醒修 bug-017: 删除从未使用的 _a2a_calibration 静态 import, 减 1 次模块
# 加载时函数解析 (H_complexity 减 0.001, 净化加载路径).
from a2a_state import latest_a2a_state as _latest_a2a_state  # noqa: E402
from a2a_state import a2a_norm  # noqa: E402


def v11_with_a2a(C: float, L: float, O: float, T: float, H: float, t: float) -> dict:
    """V11 主公式 + A2A 修正因子"""
    # FIX R2 bug-010: 1e-9 在 H=t=0 时除零保护不足, system_delta_g 飙到 4.4e9;
    # 改为 max(H*t, MIN_DENOM) — 物理意义: H 和 t 在分母, 不可能为零
    base = (C * L * O * T) / max(H * t, MIN_DENOM)
    enhanced = base * PHI_SPARK * PHI_AUTONOMOUS
    
    a2a = _latest_a2a_state()
    F_hunt = a2a.get("F_hunt", 0.0)  # [0, 1]
    A_n = a2a_norm()  # [0, 1]
    absorbed = a2a.get("absorbed", 0)
    delta_g = a2a.get("Delta_G_unlimited", 0.0)
    
    # A2A 修正：F_hunt 是 [0,1] 成功率，A_n 是 A_net 归一化
    # FIX R3 bug-012: 旧式 F_hunt * (0.3+0.7*A_n) 在 F_hunt=0 时归零,
    # 破坏 "0.3 底线" 承诺 (F_hunt=0 应给 a2a_factor=0.3 而非 0).
    # 改: 0.3 底线恒成立 + 0.7 由 F_hunt*A_n 调节强度.
    a2a_factor = 0.3 + 0.7 * F_hunt * A_n
    system_dg = enhanced * a2a_factor
    
    return {
        "v11_enhanced": round(enhanced, 2),
        "a2a_F_hunt": F_hunt,
        "a2a_absorbed": absorbed,
        "a2a_A_net": a2a.get("A_net", 0.0),
        "a2a_A_n_norm": round(A_n, 3),
        "a2a_Delta_G": delta_g,
        "a2a_factor": round(a2a_factor, 3),
        "system_delta_g": round(system_dg, 2),
    }


if __name__ == "__main__":
    # 真值代入 — 从 integration.json 读 6 维（修 R0 bug: 硬编码与 cron 报告漂移）
    p = load_6dim()
    r = v11_with_a2a(p["C"], p["L"], p["O"], p["tau"], p["H"], p["t"])
    print("===V11 + A2A 整合评估===")
    print(f"  6dim (from integration): C={p['C']} L={p['L']} O={p['O']} tau={p['tau']} H={p['H']} t={p['t']}")
    for k, v in r.items():
        print(f"  {k}: {v}")
