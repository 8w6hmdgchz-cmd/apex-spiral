#!/usr/bin/env python3
"""
Rust APEX Calculator - Python binding for nanoGPT-claw's APEX formula
调用 Rust 编译的 APEX 计算器
"""

import subprocess
import json
from pathlib import Path

RUST_APEX_BIN = Path(__file__).parent / "target" / "release" / "rust_apex"

def calculate() -> dict:
    """调用 Rust APEX 计算器"""
    try:
        result = subprocess.run(
            [str(RUST_APEX_BIN)],
            capture_output=True,
            text=True,
            timeout=5
        )
        if result.returncode == 0:
            return json.loads(result.stdout.strip())
        else:
            return {"error": result.stderr}
    except Exception as e:
        return {"error": str(e)}

def calculate_json() -> str:
    """返回 JSON 格式结果"""
    result = calculate()
    return json.dumps(result, indent=2)

if __name__ == "__main__":
    print("=== Rust APEX Calculator (nanoGPT-claw) ===")
    result = calculate()
    print(f"ΔG = {result.get('final_score', 'error'):.4f}")
    print(f"Ω_A = {result.get('omega_a', 0):.4f}")
    print(f"Total Penalty = {result.get('total_penalty', 0):.4f}")
    print(f"Confidence = {result.get('confidence', 0):.1%}")
