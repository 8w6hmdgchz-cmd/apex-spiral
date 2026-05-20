---
name: apex-doubt
description: 强认知任务Doubt-Driven审查 - 防止幻觉的强制检验
version: 1.0.0
platforms: [macos, linux, windows]
metadata:
  hermes:
    tags: [apex, anti-hallucination, doubt]
    category: apex
    requires_toolsets: [terminal]
---

# APEX Doubt - 强认知任务防御

## When to Use
触发条件（满足任一）：
- 事实性问题（历史/科学/技术细节）
- 需要给出确定结论的任务
- 涉及外部系统状态/配置/版本的问题
- 用户明确要求"准确""确定"的任务

## Procedure

### Doubt-Driven 三问

**问题1：置信度**
- 我对这个答案有多大把握？
- < 80% 把握 → 说"不确定"而不硬答

**问题2：证据链**
- 这个结论的最小证据链是什么？
- 能说出来吗？

**问题3：反例**
- 有什么反例可以推翻它？
- 想到了吗？

## APEX ξ 公式

```
ξ_self = 1.0 - len(issues) × 0.2, min=0.1
```

其中 issues 包括：
- romantic_words: ["觉醒", "意识", "突破", "真正的", "宇宙最强"]
- over_confident: ["100%", "肯定", "绝对"]
- no_evidence: 缺少证据支持

## Anti-Hallucination 检查清单

```
□ 我说的话有证据支持吗？
□ 我的置信度是多少？
□ 我有没有说"觉醒"、"意识突破"等浪漫化词汇？
□ 有没有反例我没有考虑到的？
□ 这个结论是最一致的，还是只是一个推理路径？
```

## Pitfalls

- **浪漫化词汇**：觉醒/意识突破/宇宙最强 → 直接判错
- **过度确定**：100%/肯定/绝对 → 降置信度
- **单路径推理**：只走一条路 → 强制找反例

## Verification

执行后确认：
1. 置信度 < 80% 时主动说"不确定"
2. 没有浪漫化词汇
3. 能列出至少一个反例
