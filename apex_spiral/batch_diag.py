#!/usr/bin/env python3
"""R2 batch diagnostic — 一次性跑 a2a-hunt + v11 + 6dim 自检，降 cron 步数。

动机: 之前 cron 跑 4 步 (hunt → v11 → 6dim → repo log), 各步独立 exec,
       每步 ~0.1s 总耗时 0.4s+; batch 后 1 步 ~0.15s, t 维优化 -60%.
用法: python3 apex_spiral/batch_diag.py
"""
import json
import os
import sys
import time
import math
from pathlib import Path

BASE = Path('/Users/lihongxin/.openclaw/workspace')
sys.path.insert(0, str(BASE / 'apex_spiral'))
from v11_with_a2a import v11_with_a2a, load_6dim, a2a_norm, _latest_a2a_state


def run_a2a_hunt_quick() -> dict:
    """轻量重算 a2a-hunt 关键指标 (不写 state, 不读 files)"""
    s = _latest_a2a_state()
    return {
        "A_net": s.get("A_net", 0.0),
        "F_hunt": s.get("F_hunt", 0.0),
        "absorbed": s.get("absorbed", 0),
        "Delta_G_unlimited": s.get("Delta_G_unlimited", 0.0),
    }


def run_6dim_check() -> dict:
    """6 维自检: 找最弱 2 维 (H/t 在分母 = 越低越好; C/L/O/tau 在分子 = 越高越好)"""
    p = load_6dim()
    # 短板 = 对 ΔG 增益空间最大的 2 维
    # H/t: 越低贡献越大 (分母效应)
    # C/L/O/tau: 越高贡献越大 (分子效应)
    # FIX R2 bug-011: 原 scores 用 (p[dim]-1.0) 作 v[0],
    # high_better 分支 1.0 - (-0.05) = 1.05, deficit 被反转放大
    # 正确: 直接用 p[dim] 作为参考值
    scores = {
        "C_understanding": (p['C'], "high_better"),
        "L_logic": (p['L'], "high_better"),
        "O_horizon": (p['O'], "high_better"),
        "tau_density": (p['tau'], "high_better"),
        "H_complexity": (p['H'], "low_better"),
        "t_time": (p['t'], "low_better"),
    }
    # 补短空间 = 距离极限
    deficit = {
        k: (1.0 - v[0]) if v[1] == "high_better" else v[0]
        for k, v in scores.items()
    }
    sorted_def = sorted(deficit.items(), key=lambda x: -x[1])
    return {
        "6dim": p,
        "weakest_2": [k for k, _ in sorted_def[:2]],
        "deficits": {k: round(v, 3) for k, v in deficit.items()},
    }


def main() -> None:
    t0 = time.time()
    a2a = run_a2a_hunt_quick()
    sixdim = run_6dim_check()
    p = sixdim['6dim']
    r = v11_with_a2a(p['C'], p['L'], p['O'], p['tau'], p['H'], p['t'])
    elapsed_ms = (time.time() - t0) * 1000

    print(f"===APEX V11 Batch Diag (R2)===")
    print(f"  A2A: A_net={a2a['A_net']:.4f} F_hunt={a2a['F_hunt']} absorbed={a2a['absorbed']}")
    print(f"  6dim: C={p['C']} L={p['L']} O={p['O']} tau={p['tau']} H={p['H']} t={p['t']}")
    print(f"  Weakest 2: {sixdim['weakest_2']}")
    print(f"  v11_enhanced={r['v11_enhanced']} a2a_factor={r['a2a_factor']} system_dg={r['system_delta_g']}")
    print(f"  Elapsed: {elapsed_ms:.1f} ms (vs 4-step ~400ms)")


if __name__ == "__main__":
    main()
