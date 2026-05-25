---
name: apex-strata
description: APEX StraTA 蜂群Agent系统。单指令 → 策略生成 → 多Agent并行 → 分层优化 → 记忆同步 → 自校验闭环。
metadata: { "openclaw": { "emoji": "🐝", "requires": { "bins": ["go", "openclaw"] } } }
---

# APEX StraTA 蜂群Agent Skill

## 核心公式

```
ApexStraTA = π(z|s₁) ⊗ π(aₜ|z,sₜ) ⊗ GRPO(z,aₜ) ⊗ MemLLM
```

| 层 | 符号 | 职责 | 实现 |
|----|------|------|------|
| T1 | π(z\|s₁) | GPT全局策略生成 | 主Agent用高推理LLM生成策略z |
| T2 | π(aₜ\|z,sₜ) | 固定策略多Agent并行 | Sub-agents按策略并行执行 |
| T3 | GRPO(z,aₜ) | 分层奖励优化 | 策略级+任务级奖励，KL约束 |
| T4 | MemLLM | 长期记忆同步 | RAG + LongTermMem 持久化 |

## 工作流

```
用户单指令
    ↓
[T1] π(z|s₁): 主Agent → GPT推理 → 生成策略z
    ├─ 目标任务分解
    ├─ 子任务依赖图
    ├─ 每个子任务验收标准
    └─ 并行策略参数
    ↓
[T2] π(aₜ|z,sₜ): 固定策略驱动多Agent
    ├─ Agent-1: 执行子任务1 (独立记忆)
    ├─ Agent-2: 执行子任务2 (独立记忆)
    ├─ Agent-3: 执行子任务3 (独立记忆)
    └─ Agent-N: 执行子任务N (独立记忆)
    ↓
[T3] GRPO(z,aₜ): 分层优化
    ├─ 策略级奖励 A(z): 策略质量评分
    ├─ 任务级奖励 A(aₜ): 子任务完成评分
    ├─ KL惩罚: 防策略漂移
    └─ 最远点采样: 保持多样性
    ↓
[T4] MemLLM: 记忆同步
    ├─ 每个Agent写入独立记忆
    ├─ RAG检索相关历史
    └─ 主Agent汇聚 → 长期归档
    ↓
主公式验算: 是否全部完成?
    ├─ YES → 输出结果
    └─ NO  → 回T2, 重新执行未完成
```

## 使用方式

```bash
# 激活蜂群模式
apex-strata --task "描述你的任务" --mode swarm

# 仅生成策略 (不执行)
apex-strata --task "描述任务" --mode plan

# 查看当前蜂群状态
apex-strata --status

# 查看记忆网络
apex-strata --memory --agent all
```

## 分层奖励配置

```json
{
  "strategy_reward": {
    "decomposition_quality": 0.3,
    "dependency_accuracy": 0.3,
    "verification_clarity": 0.4
  },
  "task_reward": {
    "completion": 0.5,
    "quality": 0.3,
    "efficiency": 0.2
  },
  "kl_penalty": 0.1,
  "farthest_point_sample": true
}
```

## 记忆架构

```
MemLLM = RAG + LongTermMem

每个Agent:
  ┌─────────────────┐
  │ ShortTermMem     │ ← 当前任务上下文 (ephemeral)
  ├─────────────────┤
  │ TaskMem          │ ← 子任务执行记录 (session)
  ├─────────────────┤
  │ LongTermMem      │ ← 跨session持久化 (RAG检索)
  └─────────────────┘

主Agent:
  汇聚所有Agent记忆 → 结构化归档
  → 下次类似任务可RAG检索复用
```
