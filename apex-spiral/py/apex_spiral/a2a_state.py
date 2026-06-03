"""
A2A 共享状态访问层 — R3 H_complexity 短板修复:
  之前 _latest_a2a_state() + a2a_norm() + a2a_calibration() 在 v11_with_a2a.py 实现,
  a2a-hunt-realstate.py / batch_diag.py / v11_with_a2a.py 各自
  sys.path.insert + import + 重复 schema 假设 (absorbed_count / A_net_breakdown / None).
  统一到本模块, 一处真相, 减 4 端口耦合.
"""
import json
import math
import os
import sys as _sys_a2a
from pathlib import Path

# R6 自补短: a2a_state.py 自身带 sys.path 注入. 之前 v11_with_a2a.py /
# a2a-hunt-realstate.py / batch_diag.py 三处各自做 `sys.path.insert(BASE/apex_spiral)`,
# 改集中到本模块顶部, 调用方无感 (import a2a_state 即生效), H_complexity 耦合 -1.
_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in _sys_a2a.path:
    _sys_a2a.path.insert(0, str(_THIS_DIR))

A2A_STATE_DIR = Path("/Users/lihongxin/.openclaw/workspace/state")
INTEGRATION_PATH = Path("/Users/lihongxin/.openclaw/workspace/memory/a2a-v11-integration.json")

DEFAULT_A2A_CENTER = 21.64
DEFAULT_A2A_SCALE = 8.0
_CALIB_CACHE: dict | None = None


def a2a_calibration() -> tuple[float, float]:
    """从 integration.json 读 A2A 归一化中心/缩放（配置外化 - 修 R1 bug）"""
    global _CALIB_CACHE
    if _CALIB_CACHE is not None:
        return _CALIB_CACHE["center"], _CALIB_CACHE["scale"]
    try:
        d = json.loads(INTEGRATION_PATH.read_text())
        cal = d.get("a2a_calibration", {}) or {}
        center = float(cal.get("A_net_center", DEFAULT_A2A_CENTER))
        scale = float(cal.get("A_net_scale", DEFAULT_A2A_SCALE))
    except Exception:
        center, scale = DEFAULT_A2A_CENTER, DEFAULT_A2A_SCALE
    _CALIB_CACHE = {"center": center, "scale": scale}
    return center, scale


def _coerce_int(v) -> int:
    """R3 bug-012 防御: schema 数字字段可能是 None / str / float, 统一安全转 int.
    R7 bug-016 fix: 浮点字符串 "118.0" 之前被 isdigit() False 静默返 0 — 错归零.
    改用 try/except 链: float() 后 int() 截断, 兼容 "118", "118.0", "118.5", 118.0.
    """
    if v is None:
        return 0
    if isinstance(v, bool):
        return int(v)
    if isinstance(v, int):
        return v
    if isinstance(v, float):
        return int(v)  # 截断 118.7 -> 118, 物理意义: 吸收数取整
    if isinstance(v, str):
        s = v.strip()
        if not s:
            return 0
        try:
            return int(float(s))  # 兼容 "118", "118.0", "118.7"
        except (ValueError, OverflowError):
            return 0
    return 0


def latest_a2a_state() -> dict:
    """
    拉最新一份 a2a-hunt 状态. R3 修复:
      1. absorbed 缺失时优先用 details.resource_state.absorbed_count (R0 修过)
      2. 顶层 absorbed=None 不再让 > 比较 TypeError (R3 修)
      3. 兼容 details.A_net_breakdown 长度 fallback
    返回字典恒含: F_hunt(float), A_net(float), absorbed(int), Delta_G_unlimited(float).
    """
    files = sorted(A2A_STATE_DIR.glob("a2a-hunt-*.json"), key=os.path.getmtime, reverse=True)
    if not files:
        return {"F_hunt": 0.0, "A_net": 0.0, "absorbed": 0, "Delta_G_unlimited": 0.0}
    try:
        s = json.loads(open(files[0]).read())
    except Exception:
        return {"F_hunt": 0.0, "A_net": 0.0, "absorbed": 0, "Delta_G_unlimited": 0.0}

    # FIX R6 bug-013: setdefault 不覆盖 None 值. F_hunt=None 时
    # 0.3 + 0.7 * None * A_n 会 TypeError. 强制把 None/非数 拉回 0.0
    for k in ("F_hunt", "A_net", "Delta_G_unlimited"):
        v = s.get(k)
        if v is None or not isinstance(v, (int, float)):
            s[k] = 0.0
    s["absorbed"] = _coerce_int(s.get("absorbed")) if s.get("absorbed") is not None else 0

    # 顶层 absorbed 缺失或为 0/None 时, 推算
    if s.get("absorbed") in (None, 0, "0") and isinstance(s.get("details"), dict):
        rs = s["details"].get("resource_state", {}) or {}
        real_absorbed = _coerce_int(rs.get("absorbed_count"))
        if real_absorbed == 0:
            breakdown = s["details"].get("A_net_breakdown", {}) or {}
            real_absorbed = len(breakdown) if isinstance(breakdown, dict) else 0
        s["absorbed"] = real_absorbed
    else:
        s["absorbed"] = _coerce_int(s.get("absorbed"))

    return s


def a2a_norm() -> float:
    """A2A 吸收量归一化到 [0, 1], sigmoid 形式"""
    s = latest_a2a_state()
    raw = s.get("A_net") or 0.0
    try:
        raw = float(raw)
    except (TypeError, ValueError):
        raw = 0.0
    if raw <= 0:
        return 0.0
    center, scale = a2a_calibration()
    return 1.0 / (1.0 + math.exp(-(raw - center) / scale))


# R7 补 O_horizon 短板: 观察维度靠时序变化感知. 维护一个固定长度
# RingBuffer 记录最近 N 次 A2A (F_hunt, A_net) 对, 暴露 horizon
# (最大 - 最小, 归一到 [0, 1]) 让上游可读. O_horizon 之前是单点 0.86
# (盲设), 改动态: 多源差异越大 horizon 越高 (信息越广).
_HORIZON_LOG_PATH = Path("/Users/lihongxin/.openclaw/workspace/state/a2a_resource_log.jsonl")
_HORIZON_WINDOW = 16


def a2a_horizon() -> dict:
    """R7 O_horizon 补短: 时序观察 — 读 a2a_resource_log.jsonl 最近 N 条
    计算 (F_hunt, A_net) 对的最大 - 最小, 归一化到 [0, 1].
    调用副作用: 追加当前快照 (一行 JSON) 到 log 文件. 没有文件 → 0.0.
    """
    log_path = _HORIZON_LOG_PATH
    snap = latest_a2a_state()
    # FIX R9 bug-025: `or 0.0` 不能防 NaN (NaN truthy → 透传). 加 isnan 检查.
    def _safe_float(x, default=0.0):
        try:
            v = float(x)
        except (TypeError, ValueError):
            return default
        return default if (v != v) else v  # NaN: v != v 为真
    F_h = _safe_float(snap.get("F_hunt"), 0.0)
    A_n = _safe_float(snap.get("A_net"), 0.0)
    rec = {"ts": int(_now_a2a()), "F_hunt": round(F_h, 4), "A_net": round(A_n, 4)}
    try:
        log_path.parent.mkdir(parents=True, exist_ok=True)
        with open(log_path, "a") as f:
            f.write(json.dumps(rec) + "\n")
    except Exception:
        return {"O_horizon": 0.0, "samples": 0}
    try:
        lines = log_path.read_text().splitlines()[-_HORIZON_WINDOW:]
        snaps = [json.loads(l) for l in lines if l.strip()]
    except Exception:
        return {"O_horizon": 0.0, "samples": 0}
    if not snaps:
        return {"O_horizon": 0.0, "samples": 0}
    Fs = [s["F_hunt"] for s in snaps]
    Ans = [s["A_net"] for s in snaps]
    F_span = (max(Fs) - min(Fs)) if Fs else 0.0
    A_span = (max(Ans) - min(Ans)) if Ans else 0.0
    # A_net 跨度归一: A_net 经验范围 [0, 50], F_hunt 已在 [0,1]
    horizon = max(0.0, min(1.0, 0.5 * F_span + 0.5 * (A_span / 50.0)))
    return {"O_horizon": round(horizon, 4), "samples": len(snaps)}


# R9 补 O_horizon 短板: 把 a2a_horizon() 动态观察值写回 integration.json
# 6_dim_self_check.O_horizon, 让 v11_with_a2a.load_6dim() 读到的 O 是真动态
# 值而非盲设 0.86. samples<2 时不写 (单点归零会污染 v11 主公式).
_INTEGRATION_PATH = Path("/Users/lihongxin/.openclaw/workspace/memory/a2a-v11-integration.json")


def sync_o_horizon() -> dict:
    """R9 O_horizon 补短: 时序观察值写回 integration.json 真源.

    返回: 写回的 O_horizon 数值 + samples. samples<2 跳过写 (单点 horizon=0
    会让 v11 主公式 O=0 拉低 system_delta_g, 反脆弱).
    """
    h = a2a_horizon()
    if h.get("samples", 0) < 2:
        return {"written": False, "O_horizon": h.get("O_horizon", 0.0),
                "samples": h.get("samples", 0), "reason": "samples<2"}
    try:
        d = json.loads(_INTEGRATION_PATH.read_text())
        sc = d.setdefault("6_dim_self_check", {})
        sc["O_horizon"] = float(h["O_horizon"])
        # 更新 timestamp, 让 v11 load_6dim 的 stale 检测用同一个时钟
        from datetime import datetime, timezone, timedelta
        tz = timezone(timedelta(hours=8))
        sc["last_measured_at"] = datetime.now(tz).strftime("%Y-%m-%dT%H:%M:%S%z")
        _INTEGRATION_PATH.write_text(json.dumps(d, ensure_ascii=False, indent=2))
        return {"written": True, "O_horizon": h["O_horizon"], "samples": h["samples"]}
    except Exception as e:
        return {"written": False, "O_horizon": h.get("O_horizon", 0.0),
                "samples": h.get("samples", 0), "error": str(e)[:80]}


def _now_a2a() -> float:
    """R7: 模块内时间戳辅助 (避免 v11_with_a2a 重复 import time)"""
    import time
    return time.time()
