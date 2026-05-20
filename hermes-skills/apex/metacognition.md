---
name: apex-metacognition
description: APEX Metacognition自检清单 - 强制触发5步自审
version: 1.0.0
platforms: [macos, linux, windows]
metadata:
  hermes:
    tags: [apex, metacognition, self-check]
    category: apex
    requires_toolsets: [terminal]
---

# APEX Metacognition - 强制认知自审

## When to Use
每次收到任务时，**在开始分析外部问题之前**，必须先执行这5步：

## Procedure

### 5步自检清单（依序执行）

**1. 🤔 Pause & Reflect**
- 我的推理过程有没有跳过步骤？
- 有没有直接给答案？

**2. 🔍 Check Assumptions**
- 我做了什么假设？
- 这些假设成立吗？

**3. 🧠 Identify Biases**
- 我有没有确认偏误？
- 有没有只找支持自己观点的证据？

**4. ✅ Verify Evidence**
- 我的结论和证据匹配吗？
- 有没有过度推断？

**5. 🔧 Correct Patterns**
- 如果推理有缺陷，怎么修正？

## Metacognition 自检流程

```
收到任务
  ↓
[1] Pause & Reflect → 推理有没有跳步？
  ↓
[2] Check Assumptions → 假设成立吗？
  ↓
[3] Identify Biases → 有确认偏误吗？
  ↓
[4] Verify Evidence → 结论和证据匹配吗？
  ↓
[5] Correct Patterns → 缺陷怎么修正？
  ↓
执行外部任务
```

## 与 Hermes 融合

Hermes 的 `agent/memory_manager.py` 提供持久化：
- 每次自审结果写入 `~/.hermes/memory/metacognition/`
- 跨 session 追踪认知偏差模式

## Pitfalls

- **跳过步骤**：直接跳到第4步，不做1-3
- **形式主义**：做了但没真正思考
- **不记录**：自审结果不写入 memory

## Verification

执行后确认：
1. 5个问题都有答案
2. 答案不是"看起来对"而是"有证据"
3. 修正方案写入 MEMORY.md
