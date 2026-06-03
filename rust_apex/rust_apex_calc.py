#!/usr/bin/env python3
"""
Rust APEX Calculator for Python
调用 nanoGPT-claw 的 Rust APEX 计算器
"""

import subprocess
import json
from pathlib import Path
from typing import Dict, Optional

# Rust APEX 二进制文件路径
RUST_APEX_BIN = Path("/Users/lihongxin/.openclaw/workspace/rust_apex/target/release/rust_apex")

class RustApexCalculator:
    """调用 Rust 编译的 APEX 计算器"""
    
    def __init__(self):
        self.binary = RUST_APEX_BIN
        self._available = None
    
    @property
    def available(self) -> bool:
        """检查 Rust APEX 是否可用"""
        if self._available is None:
            self._available = self.binary.exists()
        return self._available
    
    def calculate(self) -> Dict:
        """
        计算 APEX ΔG
        使用 nanoGPT-claw 的 APEX·阿卡西融合公式
        """
        if not self.available:
            return {"error": "Rust APEX not available", "fallback": "python"}
        
        try:
            result = subprocess.run(
                [str(self.binary)],
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode == 0:
                data = json.loads(result.stdout.strip())
                return {
                    "delta_g": data.get("final_score", 0),
                    "omega_a": data.get("omega_a", 0),
                    "dim_product_1": data.get("dimension_product_1", 0),
                    "dim_product_2": data.get("dimension_product_2", 0),
                    "total_penalty": data.get("total_penalty", 0),
                    "confidence": data.get("confidence", 0),
                    "source": "rust"
                }
            else:
                return {"error": result.stderr, "fallback": "python"}
        except Exception as e:
            return {"error": str(e), "fallback": "python"}
    
    def calculate_from_python(self) -> Dict:
        """Python 回退计算器（当 Rust 不可用时）"""
        # 默认参数（来自 nanoGPT-claw）
        omega_a = 0.85
        dimensions = {
            "E": 0.7, "V": 0.75, "M": 0.8, "A": 0.65, "B": 0.7,
            "T": 0.72, "D": 0.68, "H": 0.75, "L": 0.78, "G": 0.8, "W": 0.7, "B2": 0.72
        }
        penalties = [0.02, 0.01, 0.015, 0.0, 0.005, 0.001, 0.01, 0.008, 0.02, 0.012, 0.01, 0.005]
        
        dim_prod_1 = dimensions["E"] * dimensions["V"] * dimensions["M"] * dimensions["A"] * dimensions["B"]
        dim_prod_2 = dimensions["T"] * dimensions["D"] * dimensions["H"] * dimensions["L"] * dimensions["G"] * dimensions["W"] * dimensions["B2"]
        penalty_sum = sum(penalties)
        
        base_score = omega_a * 0.4
        dim_contribution = min((dim_prod_1 ** 0.5) * (dim_prod_2 ** 0.5), 0.5)
        raw_score = base_score + dim_contribution - penalty_sum
        final_score = max(0.0, min(1.0, raw_score))
        
        confidence = min(max(omega_a + penalty_sum, 0.5), 1.0)
        
        return {
            "delta_g": final_score,
            "omega_a": omega_a,
            "dim_product_1": dim_prod_1,
            "dim_product_2": dim_prod_2,
            "total_penalty": penalty_sum,
            "confidence": confidence,
            "source": "python"
        }

# 全局实例
_rust_apex = None

def get_rust_apex() -> RustApexCalculator:
    global _rust_apex
    if _rust_apex is None:
        _rust_apex = RustApexCalculator()
    return _rust_apex

def calculate_apex() -> Dict:
    """计算 APEX，优先使用 Rust，回退到 Python"""
    calc = get_rust_apex()
    if calc.available:
        return calc.calculate()
    else:
        return calc.calculate_from_python()

if __name__ == "__main__":
    print("=== Rust APEX Calculator ===")
    result = calculate_apex()
    print(f"Source: {result.get('source', 'error')}")
    print(f"ΔG = {result.get('delta_g', 0):.4f}")
    print(f"Ω_A = {result.get('omega_a', 0):.4f}")
    print(f"Confidence = {result.get('confidence', 0):.1%}")
