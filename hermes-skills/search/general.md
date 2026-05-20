---
name: search-general
description: SearchSkill通用检索 - Select-Read-Act三段式执行
version: 1.0.0
platforms: [macos, linux, windows]
metadata:
  hermes:
    tags: [search, retrieval, skillbank]
    category: search
    requires_toolsets: [terminal, web]
---

# SearchSkill General - 通用检索

## When to Use
- 需要检索外部信息时
- 复杂问题需要多跳推理时
- SkillBank 已有技能匹配时

## Procedure

### Select-Read-Act 三段式

**Select（选技能）**
```go
skillID := ss.Select(query)
// 从 SkillBank 选最优技能
```

**Read（读规则）**
```go
actQuery := ss.Bank.Read(card, query)
// 按技能规则生成检索query
```

**Act（执行）**
```go
results := ss.Act(actQuery, card)
// 调用检索，执行查询
```

### 多跳推理（带停机）

```go
chain := ss.ExecuteWithStop(query, skillChain)

// 停机条件：
// - 达到最大跳数 (4跳)
// - 不确定性 < 阈值 (0.3)
// - 边际增益 < ε (0.05)
```

### 检索压缩

```go
// 只保留 Top-K 高可执行信息
compressed := CompressResults(results, TopK=3)

score(doc) = relevance × novelty × actionability
```

## SkillBank 内置技能

| 技能 | 触发关键词 | 适用场景 |
|------|-----------|---------|
| apex_formula | 公式、代入、自检 | APEX相关问题 |
| apex_doubt | 幻觉、置信度、不确定 | 强认知任务 |
| apex_reflection | 反思、改进、进化 | 复盘任务 |
| apex_metacognition | 自审、认知、推理 | 元认知任务 |
| apex_evolution | evolver、闭环、GitHub | 演进系统 |
| apex_skill_fetch | 吸收、资源、EvoMap | 技能获取 |
| apex_github_sync | GitHub、同步、推送 | 代码同步 |

## Pitfalls

- **跳过 Select**：直接 Act，不选技能
- **不压缩**：把所有结果都带入上下文
- **无限多跳**：不设停机条件

## Verification

执行后确认：
1. Select 选出了技能
2. Act 返回了压缩后的结果
3. 多跳在2-4跳内停止
