---
name: apex-token-optimizer
description: APEX Token 优化 - 坐标校正、上下文控耗、算力有效率
metadata:
  openclaw.os: ["darwin", "linux"]
---

# APEX Token 优化超级功能

解决 OpenClaw Agent 三类原生工程缺陷：
1. 截图缩放导致物理点击坐标偏移
2. 单帧截图Token过高（1000-1800）引发上下文溢出
3. 文字等无效思维开销造成算力空耗

## 核心公式

### 1. 坐标校正
```
X_real = X_out × (W_screen / W_img)
Y_real = Y_out × (H_screen / H_img)
```

当截图尺寸与实际屏幕不符时，点击坐标需要校正。

### 2. 上下文控耗（仅保留最新3帧）
```
Token_reserve = Token_text + ΣToken_img(n) for n=N-2 to N
```

### 3. 算力有效率
```
Effort_valid = Total_effort - Waste_effort
```

## 使用方法

### 坐标校正

当需要点击截图中的某个位置时：

```bash
cd /Users/lihongxin/.openclaw/workspace/skills/apex-core
./apex_token_optimizer correct -x <x> -y <y> -iw <img_width> -ih <img_height>
```

输出：`校正坐标: (500.00, 300.00) -> (750.00, 450.00)`

### 截图处理

处理截图后调用（自动保留最新3帧）：

```bash
./apex_token_optimizer screenshot -p <path> -w <width> -h <height> -t <tokens>
```

### 算力追踪

追踪推理开销：

```bash
./apex_token_optimizer effort -t <total> -w <waste> -wt <waste_type>
```

### 净化策略

定期清理过期截图和缓存（约每15分钟执行一次）：

```bash
./apex_token_optimizer purify
```

### 查看统计

```bash
./apex_token_optimizer stats
./apex_token_optimizer traj -analyze
```

## Python 粘合层

也可以用 Python 调用：

```python
import sys
sys.path.insert(0, '/Users/lihongxin/.openclaw/workspace/skills/apex-token-optimizer/scripts')
from token_optimizer_adapter import TokenOptimizerAdapter, create_adapter, OptimizerEMVClient

# 方式1: 使用便捷函数 (推荐)
adapter = create_adapter(base_url="http://127.0.0.1:8080")

# 方式2: 手动创建
client = OptimizerEMVClient(base_url="http://127.0.0.1:8080")
adapter = TokenOptimizerAdapter(client, cfg={"max_frames": 3, "soft_cap": 200})

# 坐标校正
coords = adapter.optimize_coords("trace-001", 960.0, 540.0)

# 滑动窗口优化
frames_result = adapter.optimize_frames("trace-002", frames)

# Token预算优化
text_result = adapter.optimize_text("trace-003", "你是APEX助手")

# 25步净化
sanitized = adapter.sanitize("trace-004", frame)

# 完整优化 (一次调用包含所有功能)
result = adapter.optimize(
    trace_id="trace-005",
    coords=[100, 200],
    frames=frames,
    text_base="基础文本",
    sanitize_frame=frame
)
```

## REST API 端点

Rust服务器在 `http://127.0.0.1:8080/optimize` 提供REST API：

```json
POST /optimize
{
    "request_id": "trace-001",
    "coords": [960, 540],
    "image_size": {"w": 1920, "h": 1080},
    "roi": {"x": 0, "y": 0, "w": 1920, "h": 1080},
    "frames": [...],
    "max_frames": 3,
    "text_base": "你是APEX助手",
    "soft_cap": 200,
    "hard_cap": 300,
    "sanitize_frame": {...}
}
```

响应格式：
```json
{
    "status": "ok",
    "optimize": {
        "coordinate_correction": {"x": 960, "y": 540, "correction_applied": true},
        "sliding_window": {"frame_count": 3, "frames": [...]},
        "token_budget": {"prompt": "你是APEX助手", "token_count": 8},
        "sanitization": {"cleaned_targets": 2, "cleaned_events": 1}
    }
}
```

## 25步净化周期

| 步骤 | 周期(每5步) | 清理内容 |
|------|------------|---------|
| 0,5,10,15,20 | 75min | 过期截图(>24h) |
| 1,6,11,16,21 | 75min | 临时缓存文件 |
| 2,7,12,17,22 | 75min | 对话缓存(>7天) |
| 3,8,13,18,23 | 75min | 重复截图 |
| 4,9,14,19,24 | 75min | 压缩早间截图 |

## 触发场景

在以下场景中**必须**使用优化器：

1. **截图点击前** - 校正坐标偏移
2. **处理多帧截图** - 限制在3帧以内
3. **长任务推理** - 追踪并最小化无效开销
4. **周期性净化** - 保持上下文余量

## 优化效果目标

- 坐标精度：>95%（消除缩放偏移）
- Token节省：>60%（上下文压缩）
- 算力效率：>80%（消除无效推理）

## Rust root-fix core

Durable Rust CLI added at `crates/apex_token_optimizer`.

```bash
cargo test --manifest-path crates/apex_token_optimizer/Cargo.toml
cargo run --manifest-path crates/apex_token_optimizer/Cargo.toml -- correct --x 100 --y 50 --sw 1920 --sh 1080 --iw 1000 --ih 500
cargo run --manifest-path crates/apex_token_optimizer/Cargo.toml -- reserve --text 100 --imgs 1000,1200,900,800 --keep 3
cargo run --manifest-path crates/apex_token_optimizer/Cargo.toml -- effort --total 100 --waste 25
```

Use this core for deterministic coordinate correction, latest-3-frame token reserve, effort efficiency, and local purification helpers. Python remains glue only.
