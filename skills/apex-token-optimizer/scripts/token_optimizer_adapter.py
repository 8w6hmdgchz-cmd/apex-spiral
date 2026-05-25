#!/usr/bin/env python3
"""
TokenOptimizerAdapter - Python粘合层调用Rust apex_token_rs /optimize API

功能:
1. 坐标校正 - X_real = X_out × (W_screen / W_img)
2. 滑动窗口 - RingBuffer O(1) 追加，自动覆盖最旧
3. 25步净化 - 去重/低置信度过滤

用法:
    adapter = TokenOptimizerAdapter(emv_client, cfg)
    result = adapter.optimize(trace_id, tokens, coords)
"""
import json
import time
from typing import Dict, List, Optional, Any

# 默认配置
DEFAULT_CFG = {
    "max_frames": 3,          # 滑动窗口最大帧数
    "soft_cap": 200,          # Token软上限
    "hard_cap": 300,          # Token硬上限
    "confidence_threshold": 0.5,  # 置信度阈值 (0-1 映射到 0-100)
    "roi": {"x": 0, "y": 0, "w": 1920, "h": 1080},  # 感兴趣区域
    "image_size": {"w": 1920, "h": 1080},  # 原始图像尺寸
}


class TokenOptimizerAdapter:
    """
    Token优化适配器 - 封装Rust apex_token_rs /optimize API调用
    
    APEX Token优化三件套:
    1. 坐标校正 - 截图坐标 → 实际屏幕坐标
    2. 滑动窗口 - 保留最近N帧上下文
    3. 25步净化 - 去除噪声/重复/低置信度目标
    """
    
    def __init__(self, emv_client, cfg: Optional[Dict] = None):
        """
        初始化Token优化适配器
        
        Args:
            emv_client: EMVClient实例 (已配置base_url和timeout)
            cfg: 配置字典 (可选，使用DEFAULT_CFG)
        """
        self.emv = emv_client
        self.cfg = {**DEFAULT_CFG, **(cfg or {})}
        
    def optimize(
        self, 
        trace_id: str, 
        tokens: List[str] = None,
        coords: Optional[List[float]] = None,
        frames: Optional[List[Dict]] = None,
        text_base: Optional[str] = None,
        sanitize_frame: Optional[Dict] = None,
    ) -> Dict[str, Any]:
        """
        调用Rust /optimize端点执行Token优化
        
        Args:
            trace_id: 追踪ID (用于日志)
            tokens: Token列表 (保留，未使用)
            coords: 坐标 [x, y] (用于坐标校正)
            frames: 帧列表 (用于滑动窗口)
            text_base: 基础文本 (用于Token预算)
            sanitize_frame: 待净化帧 (用于25步净化)
            
        Returns:
            Dict: {
                "status": "ok",
                "optimize": {
                    "coordinate_correction": {...},
                    "sliding_window": {...},
                    "token_budget": {...},
                    "sanitization": {...}
                }
            }
        """
        payload = {
            "request_id": trace_id,
            "timestamp_ms": int(time.time() * 1000),
            "coords": coords,
            "image_size": self.cfg.get("image_size"),
            "roi": self.cfg.get("roi"),
            "frames": frames,
            "max_frames": self.cfg.get("max_frames"),
            "text_base": text_base,
            "soft_cap": self.cfg.get("soft_cap"),
            "hard_cap": self.cfg.get("hard_cap"),
            "sanitize_frame": sanitize_frame,
        }
        
        # 移除None值
        payload = {k: v for k, v in payload.items() if v is not None}
        
        try:
            response = self.emv._post("/optimize", payload)
            return response
        except Exception as e:
            return {
                "status": "error",
                "request_id": trace_id,
                "error": str(e)
            }
    
    def optimize_coords(self, trace_id: str, x: float, y: float) -> Dict:
        """坐标校正 - 截图坐标 → 实际屏幕坐标"""
        result = self.optimize(trace_id, coords=[x, y])
        return result.get("optimize", {}).get("coordinate_correction", {})
    
    def optimize_frames(self, trace_id: str, frames: List[Dict]) -> Dict:
        """滑动窗口优化 - 保留最近max_frames帧"""
        result = self.optimize(trace_id, frames=frames)
        return result.get("optimize", {}).get("sliding_window", {})
    
    def optimize_text(self, trace_id: str, text_base: str) -> Dict:
        """Token预算优化 - 计算prompt和token数"""
        result = self.optimize(trace_id, text_base=text_base)
        return result.get("optimize", {}).get("token_budget", {})
    
    def sanitize(self, trace_id: str, frame: Dict) -> Dict:
        """25步净化 - 去除低置信度/重复目标"""
        result = self.optimize(trace_id, sanitize_frame=frame)
        return result.get("optimize", {}).get("sanitization", {})


# ============================================================
# EMVClient扩展 - 支持/optimize端点
# ============================================================

class OptimizerEMVClient:
    """
    扩展版EMVClient - 支持/optimize端点
    继承EMVClient所有功能，添加_optimize方法
    """
    
    def __init__(self, base_url: str = "http://127.0.0.1:8080", timeout: float = 5.0):
        import urllib.request
        self.base_url = base_url
        self.timeout = timeout
        self._urllib = urllib.request
    
    def _post(self, path: str, payload: dict) -> dict:
        """POST请求辅助方法"""
        data = json.dumps(payload).encode("utf-8")
        req = self._urllib.Request(
            f"{self.base_url}{path}",
            data=data,
            headers={"Content-Type": "application/json"},
            method="POST"
        )
        with self._urllib.urlopen(req, timeout=self.timeout) as resp:
            return json.loads(resp.read().decode("utf-8"))
    
    def optimize(self, payload: dict) -> dict:
        """调用Rust /optimize端点"""
        return self._post("/optimize", payload)
    
    def health(self) -> dict:
        """健康检查"""
        req = self._urllib.Request(
            f"{self.base_url}/api/v1/health",
            method="GET"
        )
        with self._urllib.urlopen(req, timeout=self.timeout) as resp:
            return json.loads(resp.read().decode("utf-8"))


# ============================================================
# 便捷函数
# ============================================================

def create_adapter(
    base_url: str = "http://127.0.0.1:8080",
    cfg: Optional[Dict] = None
) -> TokenOptimizerAdapter:
    """创建TokenOptimizerAdapter实例的便捷函数"""
    client = OptimizerEMVClient(base_url=base_url)
    return TokenOptimizerAdapter(client, cfg)


# ============================================================
# CLI测试
# ============================================================

if __name__ == "__main__":
    import sys
    
    print("=== TokenOptimizerAdapter 测试 ===\n")
    
    # 创建适配器
    client = OptimizerEMVClient()
    adapter = TokenOptimizerAdapter(client)
    
    # 1. 健康检查
    print("[1] 健康检查")
    try:
        health = client.health()
        print(f"  状态: {health}")
    except Exception as e:
        print(f"  ⚠️ 健康检查失败: {e}")
        sys.exit(1)
    
    # 2. 坐标校正测试
    print("\n[2] 坐标校正测试")
    coords_result = adapter.optimize_coords("test-001", 960.0, 540.0)
    print(f"  输入: (960, 540)")
    print(f"  输出: {coords_result}")
    
    # 3. 滑动窗口测试
    print("\n[3] 滑动窗口测试")
    test_frames = [
        {"frame_id": 1, "timestamp_ms": 100, "player_state": "idle", "event_flags": ["move"]},
        {"frame_id": 2, "timestamp_ms": 200, "player_state": "running", "event_flags": ["shoot"]},
        {"frame_id": 3, "timestamp_ms": 300, "player_state": "fighting", "event_flags": ["damage"]},
        {"frame_id": 4, "timestamp_ms": 400, "player_state": "dead", "event_flags": ["death"]},
    ]
    frames_result = adapter.optimize_frames("test-002", test_frames)
    print(f"  输入帧数: 4, 最大保留: 3")
    print(f"  输出: {frames_result}")
    
    # 4. Token预算测试
    print("\n[4] Token预算测试")
    text_result = adapter.optimize_text("test-003", "你是APEX助手，正在分析游戏画面")
    print(f"  文本: '你是APEX助手，正在分析游戏画面'")
    print(f"  输出: {text_result}")
    
    # 5. 25步净化测试
    print("\n[5] 25步净化测试")
    sanitize_input = {
        "frame_id": 1,
        "timestamp_ms": 100,
        "targets": [
            {"id": 1, "cls": "enemy", "x": 100, "y": 200, "vx": 1, "vy": -1, "conf": 30},  # 低置信度
            {"id": 2, "cls": "enemy", "x": 105, "y": 205, "vx": 1, "vy": -1, "conf": 85},  # 重复目标
            {"id": 3, "cls": "loot", "x": 300, "y": 400, "vx": 0, "vy": 0, "conf": 72},
        ],
        "player_state": "fighting",
        "event_flags": ["shot", "shot"],  # 重复事件
    }
    sanitize_result = adapter.sanitize("test-004", sanitize_input)
    print(f"  输入: 2个enemy目标(1个低置信度+1个重复), 1个loot目标, 2个重复事件")
    print(f"  输出: {sanitize_result}")
    
    print("\n=== 测试完成 ===")
