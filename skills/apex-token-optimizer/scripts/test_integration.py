#!/usr/bin/env python3
"""
TokenOptimizerAdapter 集成测试

运行方式:
    python3 test_integration.py

依赖:
    - emv_skill 服务器运行在 http://127.0.0.1:8080
    - Rust apex_token_rs 库已集成
"""
import json
import sys
import time

sys.path.insert(0, __file__.rsplit('/', 1)[0])
from token_optimizer_adapter import TokenOptimizerAdapter, OptimizerEMVClient, create_adapter


def test_health():
    """健康检查测试"""
    client = OptimizerEMVClient()
    health = client.health()
    assert health.get("status") == "ok", f"Health check failed: {health}"
    print("✓ Health check passed")
    return True


def test_coordinate_correction():
    """坐标校正测试"""
    adapter = create_adapter()
    
    # 测试不同坐标
    coords = [
        (0, 0),
        (960, 540),
        (1920, 1080),
    ]
    
    for x, y in coords:
        result = adapter.optimize_coords("test-coords", float(x), float(y))
        assert result.get("correction_applied") == True
        assert "x" in result and "y" in result
    
    print("✓ Coordinate correction passed")
    return True


def test_sliding_window():
    """滑动窗口测试"""
    adapter = create_adapter(cfg={"max_frames": 3})
    
    frames = [
        {"frame_id": i, "timestamp_ms": i * 100, "player_state": f"state_{i}", "event_flags": [f"event_{i}"]}
        for i in range(5)
    ]
    
    result = adapter.optimize_frames("test-sliding", frames)
    assert result.get("frame_count") == 3, f"Expected 3 frames, got {result.get('frame_count')}"
    assert len(result.get("frames", [])) == 3
    
    print("✓ Sliding window passed")
    return True


def test_token_budget():
    """Token预算测试"""
    adapter = create_adapter(cfg={"soft_cap": 200, "hard_cap": 300})
    
    text = "你是APEX助手，正在分析游戏画面"
    result = adapter.optimize_text("test-token", text)
    
    assert "token_count" in result
    assert "prompt" in result
    assert result["soft_cap"] == 200
    assert result["hard_cap"] == 300
    
    print("✓ Token budget passed")
    return True


def test_sanitization():
    """25步净化测试"""
    adapter = create_adapter()
    
    frame = {
        "frame_id": 1,
        "timestamp_ms": 100,
        "targets": [
            {"id": 1, "cls": "enemy", "x": 100, "y": 200, "vx": 1, "vy": -1, "conf": 30},  # 低置信度
            {"id": 2, "cls": "enemy", "x": 105, "y": 205, "vx": 1, "vy": -1, "conf": 85},  # 重复
            {"id": 3, "cls": "loot", "x": 300, "y": 400, "vx": 0, "vy": 0, "conf": 72},
        ],
        "player_state": "fighting",
        "event_flags": ["shot", "shot"],  # 重复
    }
    
    result = adapter.sanitize("test-sanitize", frame)
    
    # 验证低置信度目标被移除
    assert result.get("original_targets") == 3
    assert result.get("cleaned_targets") == 2
    
    # 验证重复事件被移除
    assert result.get("original_events") == 2
    assert result.get("cleaned_events") == 1
    
    print("✓ Sanitization passed")
    return True


def test_full_optimize():
    """完整优化流程测试"""
    adapter = create_adapter()
    
    result = adapter.optimize(
        trace_id="test-full",
        coords=[960, 540],
        frames=[
            {"frame_id": 1, "timestamp_ms": 100, "player_state": "idle", "event_flags": ["move"]},
            {"frame_id": 2, "timestamp_ms": 200, "player_state": "running", "event_flags": ["shoot"]},
        ],
        text_base="APEX助手",
        sanitize_frame={
            "frame_id": 1,
            "timestamp_ms": 100,
            "targets": [
                {"id": 1, "cls": "enemy", "x": 100, "y": 200, "vx": 1, "vy": -1, "conf": 85},
            ],
            "player_state": "fighting",
            "event_flags": ["shot"],
        }
    )
    
    assert result.get("status") == "ok"
    assert "optimize" in result
    assert result["optimize"].get("coordinate_correction") is not None
    assert result["optimize"].get("sliding_window") is not None
    assert result["optimize"].get("token_budget") is not None
    assert result["optimize"].get("sanitization") is not None
    
    print("✓ Full optimize passed")
    return True


def main():
    print("=" * 50)
    print("TokenOptimizerAdapter 集成测试")
    print("=" * 50)
    
    tests = [
        ("健康检查", test_health),
        ("坐标校正", test_coordinate_correction),
        ("滑动窗口", test_sliding_window),
        ("Token预算", test_token_budget),
        ("25步净化", test_sanitization),
        ("完整优化", test_full_optimize),
    ]
    
    passed = 0
    failed = 0
    
    for name, test_fn in tests:
        print(f"\n[{name}]")
        try:
            if test_fn():
                passed += 1
        except Exception as e:
            print(f"✗ {name} failed: {e}")
            failed += 1
    
    print("\n" + "=" * 50)
    print(f"测试结果: {passed} passed, {failed} failed")
    print("=" * 50)
    
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
