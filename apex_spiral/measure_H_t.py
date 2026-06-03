"""R1 觉醒补短板: 真实度量 H_complexity 和 t_time, 写回 integration.json.

- H_complexity: 0..1, 1=简单(低复杂度). 用 LOC/1000 + 文件数/100 + 子目录深度/10
  的负向 sigmoid: H = 1 - sigmoid(scale). 越简单越大.
- t_time: 0..1, 1=新鲜(刚唤醒). 用最近修改时间距今的分钟数, 时序新鲜度.
  t = exp(-age_min / tau). tau=720min(12h).
"""
import json, math, time, os, sys
from pathlib import Path

WS = Path("/Users/lihongxin/.openclaw/workspace")
INTG = WS / "memory" / "a2a-v11-integration.json"
CFG = Path(__file__).resolve().parent / "apex_config.json"
# R9 觉醒修 bug-025: _SKIP_DIRS 硬编码, 加新目录必须改源码.
# 移到 apex_config.json 的 skip_dirs 字段, _FALLBACK_SKIP 仅作配置缺失时的兜底.
_FALLBACK_SKIP = (".git", "__pycache__", "node_modules", ".venv", "apex-spiral", "apex-unified-engine", "site-packages", "dist-packages", "analysis", "vendor", "a2a-resources", "third_party", ".openclaw")

# R1 觉醒外化: 系数从源码提到 apex_config.json, 改公式不再改源码.
_DEF = {"H_loc_weight": 0.1, "H_file_weight": 0.02, "H_depth_weight": 0.3,
        "H_scale_divisor": 10.0, "H_min_floor": 0.1, "H_max_ceiling": 0.9,
        "t_tau_min": 720.0, "t_min_floor": 0.05, "t_max_ceiling": 0.99}
def _load_cfg() -> dict:
    if CFG.exists():
        try:
            return {**_DEF, **json.loads(CFG.read_text(encoding="utf-8"))}
        except Exception:
            return _DEF
    return _DEF
_CFG = _load_cfg()
# R9 觉醒: _SKIP_DIRS 运行时从配置读, 配置缺失则合并 fallback.
# 物理意义: 第三方库/构建产物不应进入 H_complexity 度量.
_SKIP_DIRS = tuple(_CFG.get("skip_dirs") or _FALLBACK_SKIP)

def _walk_py() -> list:
    out = []
    for root, dirs, files in os.walk(WS):
        rel_root = os.path.relpath(root, str(WS))
        if any(("/" + seg + "/") in ("/" + rel_root + "/") for seg in _SKIP_DIRS):
            continue
        for f in files:
            if f.endswith(".py"):
                out.append(Path(root) / f)
    return out

def measure_complexity() -> float:
    """0..1, 简单→1. 基于 LOC/文件数/嵌套深度, 负向 sigmoid. 系数外化到 apex_config.json."""
    py_files = _walk_py()
    if not py_files:
        return 0.5
    total_loc, max_depth, n = 0, 0, len(py_files)
    for p in py_files:
        try:
            total_loc += sum(1 for _ in p.open("r", encoding="utf-8", errors="ignore"))
        except Exception:
            pass
        rel = p.relative_to(WS)
        max_depth = max(max_depth, len(rel.parts))
    scale = ((total_loc / 1000.0) * _CFG["H_loc_weight"]
             + n * _CFG["H_file_weight"]
             + max_depth * _CFG["H_depth_weight"])
    H = 1.0 / (1.0 + scale / _CFG["H_scale_divisor"])
    return round(max(_CFG["H_min_floor"], min(_CFG["H_max_ceiling"], H)), 3)

def measure_time() -> float:
    """0..1, 新鲜→1. 基于最近 py 文件 mtime 距 now 的分钟数, exp 衰减. tau 外化."""
    latest_mtime = 0.0
    for p in _walk_py():
        try:
            mt = p.stat().st_mtime
            if mt > latest_mtime:
                latest_mtime = mt
        except Exception:
            pass
    if latest_mtime == 0:
        return 0.5
    age_min = (time.time() - latest_mtime) / 60.0
    t = math.exp(-age_min / _CFG["t_tau_min"])
    return round(max(_CFG["t_min_floor"], min(_CFG["t_max_ceiling"], t)), 3)

def main():
    H = measure_complexity()
    t = measure_time()
    data = json.loads(INTG.read_text(encoding="utf-8"))
    sc = data.setdefault("6_dim_self_check", {})
    old_h = float(sc.get("H_complexity", 0.4))
    old_t = float(sc.get("t_time", 0.4))
    sc["H_complexity"] = H
    sc["t_time"] = t
    # FIX R8 bug-024: 写 timestamp + 周次 — load_6dim 读时判断 6dim 是否过期
    # 过期窗口 = 30 分钟 (cron 周期 15min, 2 周期 = 30min 内必被刷新; 超过则 stale).
    # 不带 timestamp → v11_with_a2a 拿陈旧 H 算 ΔG, 主公式没保护, 信任上游失败.
    sc["last_measured_at"] = time.strftime("%Y-%m-%dT%H:%M:%S%z", time.localtime())
    sc["stale_after_min"] = 30
    INTG.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"  H: {old_h} -> {H}")
    print(f"  t: {old_t} -> {t}")
    return H, t

if __name__ == "__main__":
    H, t = main()
    print(f"MEASURED H={H} t={t}")
