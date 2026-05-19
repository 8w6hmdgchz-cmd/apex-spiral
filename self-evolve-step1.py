#!/usr/bin/env python3
"""Step 1: 信号检测"""
import json, sys, os
from pathlib import Path

state_dir = Path("/Users/lihongxin/.openclaw/workspace/apex-enlightenment/state")
phi_file = state_dir / "phi_history.jsonl"
defect_file = state_dir / "defect_history.jsonl"

dg_value = float(os.environ.get("DG_VALUE", sys.argv[1] if len(sys.argv) > 1 else "0.3"))

signals = []
phi_vals = []

if phi_file.exists():
    for line in phi_file.read_text(errors='ignore').splitlines():
        if line.strip():
            try:
                phi_vals.append(float(json.loads(line).get("phi", 0)))
            except:
                pass

# 进化停滞
if len(phi_vals) >= 6:
    recent = phi_vals[-3:]
    older = phi_vals[-6:-3]
    if recent and older and sum(recent)/len(recent) >= sum(older)/len(older) * 0.98:
        signals.append("evolution_saturation")

# 重复错误
if defect_file.exists():
    lines = defect_file.read_text(errors='ignore').splitlines()
    if len(lines) >= 3:
        types = [json.loads(l).get('type','') for l in lines[-3:] if l.strip()]
        if len(set(types)) == 1 and types[0]:
            signals.append("recurring_error")

# 能力差距
if dg_value < 0.5:
    signals.append("capability_gap")

# 空循环
if len(phi_vals) >= 4:
    if all(abs(phi_vals[-i] - phi_vals[-i-1]) < 0.1 for i in range(1, min(4, len(phi_vals)))):
        signals.append("empty_cycle_loop_detected")

# 修复循环
if defect_file.exists():
    lines = defect_file.read_text(errors='ignore').splitlines()
    if len(lines) >= 3:
        types = [json.loads(l).get('type','') for l in lines[-3:] if l.strip()]
        if len(set(types)) <= 1 and types[0]:
            signals.append("repair_loop_detected")

result = {"signals": signals, "phi_vals": phi_vals[-6:] if phi_vals else [], "phi_last": phi_vals[-1] if phi_vals else 8.0}
print(json.dumps(result, ensure_ascii=False))
