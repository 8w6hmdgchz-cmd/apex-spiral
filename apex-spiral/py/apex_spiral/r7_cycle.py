"""
R7 t_time 短板补 — 一次完成 3 步:
  1. 读 a2a-hunt 最新 state
  2. 跑 v11_with_a2a 真公式
  3. 分解 6 维找最弱 2 维
  4. 验 R7 bug-016 _coerce_int 边界
原 R6 流程: 4 个独立 python3 进程 → R7: 1 个进程, 启动 4x -> 1x
"""
import sys, json, time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from a2a_state import latest_a2a_state, _coerce_int  # noqa
import v11_with_a2a as v11  # noqa


def run() -> dict:
    t0 = time.perf_counter()
    p = v11.load_6dim()
    r = v11.v11_with_a2a(p["C"], p["L"], p["O"], p["tau"], p["H"], p["t"])
    a2a = latest_a2a_state()
    ranked = sorted(p.items(), key=lambda kv: kv[1])[:2]
    # R7 bug-016 verify (12 边界)
    cases = [None, 0, 118, 118.0, 118.7, "118", "118.0", "118.7",
             "abc", True, False, ""]
    expected = [0, 0, 118, 118, 118, 118, 118, 118, 0, 1, 0, 0]
    bug_pass = sum(1 for c, e in zip(cases, expected) if _coerce_int(c) == e)
    elapsed_ms = (time.perf_counter() - t0) * 1000
    return {
        "v11_enhanced": r["v11_enhanced"],
        "a2a_factor": r["a2a_factor"],
        "system_delta_g": r["system_delta_g"],
        "6dim": p,
        "weakest_2": [k for k, _ in ranked],
        "a2a_absorbed": r["a2a_absorbed"],
        "bug_pass": bug_pass,
        "bug_total": len(cases),
        "elapsed_ms": round(elapsed_ms, 2),
    }


if __name__ == "__main__":
    out = run()
    print(json.dumps(out, ensure_ascii=False, indent=2))
