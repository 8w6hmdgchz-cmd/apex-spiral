---
name: apex-praison-chain
description: PraisonAI-style APEX role-agent chain. Use this when a task needs role-based multi-agent planning, explicit task graphs, tool guardrails, verification loops, and durable memory/skill distillation.
metadata: { "openclaw": { "emoji": "🧬", "requires": { "bins": ["go"] } } }
---

# APEX Praison Chain Skill

## Source

- Upstream inspiration: `MervinPraison/PraisonAI`
- Verified SSH reachability: `git@github.com:MervinPraison/PraisonAI.git` HEAD `8acf77c531e624c46d3d61dcae37e9942e90972c`
- Local implementation: `scripts/apex-praison-chain`

This skill distills the reusable orchestration pattern, not a claim of importing the whole repository.

## Core Formula

```text
ApexPraisonChain = RoleAgents × TaskGraph × ProcessMode × ToolGate × VerifyLoop × MemLedger
```

## When To Use

Use for complex tasks where the work benefits from explicit agent roles:

- building or repairing code/tools/skills
- GitHub repo distillation
- research workflows needing separate planner/builder/critic roles
- tasks where verification must be separated from implementation
- converting vague goals into executable task graphs

## Modules

| Module | Function |
|---|---|
| RoleAgents | Creates planner, builder, critic, plus domain roles such as researcher/scavenger |
| TaskGraph | Turns the objective into dependency-aware tasks |
| ProcessMode | Selects sequential, parallel, or hierarchical flow |
| ToolGate | Assigns allowed tools and guardrails per role |
| VerifyLoop | Forces concrete checks before final claims |
| MemLedger | Records durable lessons into memory or skills |

## CLI

```bash
cd /Users/lihongxin/.openclaw/workspace/scripts/apex-praison-chain

# Generate full chain plan
./apex-praison-chain --task "修复猎食器并固化skill" --process hierarchical

# Emit only role agents
./apex-praison-chain --mode roles --task "PHN biomarker validation"

# Emit score only
./apex-praison-chain --mode score --task "吸收GitHub repo为本地skill"

# Print skill summary
./apex-praison-chain --mode skill
```

## Operating Discipline

1. Planner defines outputs and acceptance criteria.
2. Builder changes files or produces artifacts.
3. Critic verifies with build/test/lint/direct inspection.
4. Durable lessons go to `memory/` or `skills/` only when they will help future runs.
5. Do not treat chain score as proof; evidence comes from artifacts and verification commands.
