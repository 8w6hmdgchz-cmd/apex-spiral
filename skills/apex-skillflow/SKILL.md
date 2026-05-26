---
name: apex-skillflow
description: APEX SkillFlow - 流匹配驱动的技能演化引擎。融合流匹配模式与 APEX 融合公式，实现技能网络的流量控制、奖励正比路由、信用分配与策略坍塌防治。
metadata: { "openclaw": { "emoji": "🧬", "requires": { "bins": ["go", "cargo", "git"] } } }
---

# APEX SkillFlow Skill

## 核心公式

```text
π*(τ|q) ∝ R̃(τ)^β,   R̃(τ) = R(τ) + ε_min
Flow(s→a) ∝ Reward(trajectory through s→a)
```

## 架构

```
skillflow/
  core/              ← Rust 流匹配核心
    src/lib.rs       ← π* 概率 / R̃ 归一化 / 信用分配
    schema.json      ← 14 数据集节点定义
  orchestration/      ← Go TTB DAG 编排层
    main.go         ← 流路由 / 坍塌检测 / 多峰冗余
  evolution/         ← Go 反向策略演化引擎
    main.go         ← 反向策略 / reward gradient 调整
  validation/        ← Go 14 数据集验证 harness
    main.go         ← 逐数据集 eval harness 调用
```

## 14 数据集节点

| 数据集 | 领域 | 核心节点 |
|--------|------|----------|
| mmlu_pro | reasoning | apex-strata, apex-evolver-core |
| humaneval_plus | code | apex-claude-code-runner, apex-harness-bridge |
| math_o1 | math | apex-autoresearch-core, apex-token-optimizer |
| gaia_benchmark | agentic | apex-ecc-runtimeos, apex-fusion-engine |
| arxiv_summarization | nlp | apex-book-skill, apex-unified-research-engine |
| biology_qa | science | clawg-training-ecosystem, apex-devour-engine |
| swebench | swe | apex-mini-executor, apex-eval-harness |
| mmlu_ethics | ethics | apex-secrets-gate, apex-evidence-validator |
| popqwa | reasoning | apex-praison-chain, apex-superpowers-gate |
| realworld_agent | embodied | apex-container-backend, apex-memory-admission |
| cybersecurity_ctf | security | apex-secrets-gate, apex-hygiene |
| financial_fraud | finance | akquant-backtest, xuanjiquant |
| medical_qa | biomed | clawg-training-ecosystem, apex-tiangong-skill |
| dialogue_safety | safety | apex-core, apex-cmmi-delivery |

## 流网络属性

```text
多峰负载冗余：   每个关键节点 ≥2 条独立路径
策略坍塌防治：   Flow 熵 < 阈值 → 注入噪声重启
透明信用分配：   每个 skill 的 reward 贡献可追溯
反向策略：       reward_gradient × learning_rate × reverse_signal
```

## CLI

```bash
# Rust 核心测试
cd skillflow/core && cargo test

# 编排层
cd skillflow/orchestration
go build -o apex-skillflow-orchestration .
./apex-skillflow-orchestration --root /Users/lihongxin/.openclaw/workspace

# 演化引擎
cd skillflow/evolution
go build -o apex-skillflow-evolution .
./apex-skillflow-evolution --root /Users/lihongxin/.openclaw/workspace

# 14 数据集验证
cd skillflow/validation
go build -o apex-skillflow-validation .
./apex-skillflow-validation --root /Users/lihongxin/.openclaw/workspace
```

## Evidence

- `state/skillflow-dag.json`
- `state/skillflow-orchestration-latest.json`
- `state/skillflow-evolution-latest.json`
- `state/skillflow-validation-latest.json`

## Non-Negotiables

- 不伪造流匹配结果
- 不注入假数据通过验证
- 坍塌检测失败必须真实报告，不降级处理
