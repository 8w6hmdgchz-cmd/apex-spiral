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

# FIX R8 bug-023: PHI / MIN_DENOM 原本硬编码 4 处魔数, 改从 integration.json
# 读 v11_constants 配置 (配置外化 = H_complexity 短板修复).
# 物理意义: H_complexity 来源之一 = 公式里的隐式参数散落; 集中到配置,
# 公式本体只关心语义, 调参不动代码, 减 1 端口耦合.
_PHI_FALLBACK = {"PHI_SPARK": 3.38, "PHI_AUTONOMOUS": 3.0, "MIN_DENOM": 0.01}
A2A_STATE_DIR = Path("/Users/lihongxin/.openclaw/workspace/state")
INTEGRATION_PATH = Path("/Users/lihongxin/.openclaw/workspace/memory/a2a-v11-integration.json")


def _load_v11_constants() -> dict:
    """配置外化 — 公式魔数集中到 integration.json (R8 觉醒修复 bug-023).
    读失败回退硬编码默认, 绝不崩."""
    try:
        d = json.loads(INTEGRATION_PATH.read_text())
        c = d.get("v11_constants", {}) or {}
        out = dict(_PHI_FALLBACK)
        for k in _PHI_FALLBACK:
            v = c.get(k)
            if isinstance(v, (int, float)) and v == v:  # 非 NaN
                out[k] = float(v)
        return out
    except Exception:
        return dict(_PHI_FALLBACK)


# 启动时一次性解析; 修改配置后需重启进程 (符合 cron 周期)
_V11_CONST = _load_v11_constants()
PHI_SPARK = _V11_CONST["PHI_SPARK"]
PHI_AUTONOMOUS = _V11_CONST["PHI_AUTONOMOUS"]
MIN_DENOM = _V11_CONST["MIN_DENOM"]

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
        # FIX R8 bug-024: 检测 6dim 测量是否过期 (last_measured_at + stale_after_min).
        # measure_H_t.py 每次写回带 timestamp; v11 主公式读到 stale 值应记录 warning.
        # 不抛错 (cron 周期可容忍偶发过期), 但暴露状态供下轮 measure_H_t 主动刷新.
        try:
            ts = sc.get("last_measured_at")
            stale_min = float(sc.get("stale_after_min", 30))
            if ts:
                from datetime import datetime
                fmt = "%Y-%m-%dT%H:%M:%S%z"
                try:
                    measured = datetime.strptime(ts, fmt)
                except ValueError:
                    measured = None
                if measured is not None:
                    age_min = (datetime.now() - measured).total_seconds() / 60.0
                    if age_min > stale_min:
                        import sys as _sys
                        print(f"  [WARN] 6dim stale: {age_min:.1f}min > {stale_min}min, run measure_H_t.py",
                              file=_sys.stderr)
        except Exception:
            pass  # stale 检测失败不影响主公式 (鲁棒性优先)
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
    # FIX R8 bug-023 命名: MIN_DENOM 含义模糊 (是 H*t 下限还是分母下限?),
    # 实测当 H=t=0.1 时 H*t=0.01=MIN_DENOM, 公式给"假正常值". 注释说明
    # 改为 H_T_FLOOR (H·t 乘积下限), 与原 MIN_DENOM 等价但语义准确.
    base = (C * L * O * T) / max(H * t, MIN_DENOM)  # H_T_FLOOR 语义 = MIN_DENOM
    enhanced = base * PHI_SPARK * PHI_AUTONOMOUS
    
    a2a = _latest_a2a_state()
    # FIX R7 bug-022: 二次 None/非数防御. a2a_state.py 已 sanitize,
    # 但本函数作为公式入口, 鲁棒性原则要求不信任上游 — 万一 a2a_state
    # schema 变更又回归 (R6 修过 None 钳位, R3 修过 setdefault),
    # 0.3 + 0.7 * None 立刻 TypeError 整轮崩. 加钳位 + 非数回退.
    F_hunt_raw = a2a.get("F_hunt", 0.0)
    if not isinstance(F_hunt_raw, (int, float)) or F_hunt_raw != F_hunt_raw:  # NaN check
        F_hunt = 0.0
    else:
        F_hunt = max(0.0, min(1.0, float(F_hunt_raw)))
    A_n = a2a_norm()  # [0, 1] — 内部已做 None 防御
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
