---
name: apex-reflection
description: PCEC周期进化反思模板 - 每3小时自动触发
version: 1.0.0
platforms: [macos, linux, windows]
metadata:
  hermes:
    tags: [apex, reflection, evolution]
    category: apex
    requires_toolsets: [terminal]
---

# APEX Reflection - 周期进化反思

## When to Use
- 每3小时自动触发
- 完成任何多步骤任务后
- 发现重大错误/bug后

## Procedure

### 任务反思模板

```markdown
【任务反思】
- 执行结果: [成功/失败/部分成功]
- 关键因素: [什么导致了成功或失败]
- 可改进点: [下次可以做得更好的地方]
- 经验提取: [可以复用的模式或技能]
- 知识更新: [需要记录的新知识]
```

### 思维爆炸三问（PCEC触发时必问）

1. "如果彻底推翻当前默认做法，会发生什么？"
2. "如果我是系统设计者，会删掉什么？"
3. "如果要让能力弱10倍的agent也能成功，需要补什么？"

### 技能提取工作流

1. **观察重复行为** → 记录每次出现相同模式
2. **提取共同模式** → 抽象出通用结构
3. **抽象为通用技能** → 写成 SkillBank 条目
4. **存储到 skills/** → 持久化到技能库

## 进化指标

| 指标 | 目标 | 当前 |
|------|------|------|
| 任务成功率 | >95% | — |
| 代码质量 | >8/10 | — |
| 自动化发现 | ≥3个/周 | — |
| 技能库增长 | +20%/月 | — |

## Pitfalls

- **跳过反思**：任务做完就停，不回头看
- **只写不执行**：写了反思但不更新技能库
- **无数据**：反思没有基于实际轨迹数据

## Verification

执行后确认：
1. 填写了5个反思字段
2. 至少提出1个可改进点
3. 更新了MEMORY.md或SkillBank
