#!/usr/bin/env python3
"""
apex_token_glue.py - APEX Token 优化 Python 粘合层
APEX 自进化推理引擎 - OpenClaw 原生集成
"""
import subprocess
import json
import os
import sys

# APEX Token 优化器路径
OPTIMIZER_BIN = os.path.join(os.path.dirname(__file__), "apex_token_optimizer")


class APEXTokenOptimizer:
    """APEX Token 优化器 Python 粘合层"""
    
    def __init__(self, screen_width=1920, screen_height=1080):
        self.screen_width = screen_width
        self.screen_height = screen_height
        self._check_binary()
    
    def _check_binary(self):
        if not os.path.exists(OPTIMIZER_BIN):
            raise FileNotFoundError(f"APEX Token Optimizer 二进制不存在: {OPTIMIZER_BIN}")
    
    def _run(self, *args):
        """执行优化器命令"""
        result = subprocess.run(
            [OPTIMIZER_BIN] + list(args),
            capture_output=True, text=True, timeout=30
        )
        if result.returncode != 0:
            raise RuntimeError(f"优化器错误: {result.stderr}")
        return result.stdout
    
    def correct_click(self, x, y, img_width, img_height):
        """
        坐标校正
        X_real = X_out × (W_screen / W_img)
        Y_real = Y_out × (H_screen / H_img)
        """
        output = self._run(
            "correct",
            "-x", str(x),
            "-y", str(y),
            "-iw", str(img_width),
            "-ih", str(img_height)
        )
        # 解析输出: 校正坐标: (500.00, 300.00) -> (750.00, 450.00)
        line = output.strip().split(" -> ")
        if len(line) == 2:
            corrected = line[1].strip("()")
            cx, cy = map(float, corrected.split(", "))
            return cx, cy
        raise RuntimeError(f"无法解析校正输出: {output}")
    
    def process_screenshot(self, path, width, height, tokens):
        """处理截图（控制 Token）"""
        self._run(
            "screenshot",
            "-p", path,
            "-w", str(width),
            "-h", str(height),
            "-t", str(tokens)
        )
    
    def track_effort(self, total, waste=0, waste_type="unknown"):
        """
        追踪算力开销
        Effort_valid = Total_effort - Waste_effort
        """
        self._run(
            "effort",
            "-t", str(total),
            "-w", str(waste),
            "-wt", waste_type
        )
    
    def purify(self):
        """执行 25 步净化策略"""
        output = self._run("purify")
        return output.strip()
    
    def get_stats(self):
        """获取完整统计"""
        output = self._run("stats")
        return json.loads(output)
    
    def analyze_trajectory(self):
        """分析轨迹日志"""
        output = self._run("traj", "-analyze")
        return output.strip()


# ============ 快捷函数 ============

_optimizer = None

def get_optimizer():
    global _optimizer
    if _optimizer is None:
        _optimizer = APEXTokenOptimizer()
    return _optimizer


def correct_click(x, y, img_width, img_height):
    """快捷坐标校正"""
    return get_optimizer().correct_click(x, y, img_width, img_height)


def process_screenshot(path, width, height, tokens):
    """快捷截图处理"""
    get_optimizer().process_screenshot(path, width, height, tokens)


def track_effort(total, waste=0, waste_type="unknown"):
    """快捷算力追踪"""
    get_optimizer().track_effort(total, waste, waste_type)


def purify():
    """快捷净化"""
    get_optimizer().purify()


def get_stats():
    """快捷获取统计"""
    return get_optimizer().get_stats()


# ============ CLI 入口 ============

if __name__ == "__main__":
    if len(sys.argv) > 1:
        opt = APEXTokenOptimizer()
        if sys.argv[1] == "stats":
            import pprint
            pprint.pprint(opt.get_stats())
        elif sys.argv[1] == "purify":
            print(opt.purify())
        elif sys.argv[1] == "correct" and len(sys.argv) == 6:
            x, y = opt.correct_click(
                float(sys.argv[2]), float(sys.argv[3]),
                float(sys.argv[4]), float(sys.argv[5])
            )
            print(f"校正后: ({x}, {y})")
        else:
            print("用法: python3 apex_token_glue.py [stats|purify|correct x y w h]")
    else:
        print("APEX Token Optimizer Python 粘合层")
        print("用法: python3 apex_token_glue.py [stats|purify|correct x y w h]")