---
name: apex-praison-chain
description: PraisonAI-style APEX role-agent chain. Use this when a task needs role-based multi-agent planning, explicit task graphs, tool guardrails, verification loops, and durable memory/skill distillation.
metadata: { "openclaw": { "emoji": "🧬", "requires": { "bins": ["go"] } } }
---

# APEX Praison Chain Skill

## Source

- Upstream installed from GitHub SSH: `git@github.com:MervinPraison/PraisonAI.git`
- Local installed source tree: `/Users/lihongxin/.openclaw/workspace/vendor/github/MervinPraison/PraisonAI`
- Tracked source snapshot: `/Users/lihongxin/.openclaw/workspace/third_party/praisonai/snapshot`
- Installed HEAD: `8acf77c531e624c46d3d61dcae37e9942e90972c`
- Install method: sparse shallow fetch over SSH (`git fetch --depth=1 --filter=blob:none`, sparse paths: `README*`, `pyproject.toml`, `setup.py`, `praisonaiagents/**`, `praisonai/**`, `src/**`)
- Local implementation: `scripts/apex-praison-chain`

This skill is backed by a local GitHub-installed source snapshot and distills the reusable orchestration pattern into a deterministic Go helper.

## Upstream Signals Observed

From the installed repository README/source snapshot:

- `Agent` and `Agents` primitives for single/multi-agent execution.
- Multi-agent examples and agent handoffs.
- Tool/MCP integration.
- Background tasks and workflows.
- Guardrails, memory, knowledge, cron/dashboard concepts.

## Core Formula

```text
ApexPraisonTransmission = APEXFormulaMirror × RoleAgents × TaskGraph × ToolGate × FusionGate × EvidenceAdmission × MemLedger
```

The older planning core remains:

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

## Background Activation Protocol

Use this skill as a reusable APEX transmission chain when a task must move from intent to verified system change:

1. Run `apex-praison-chain --mode activate` to create role/task/guardrail/evidence plan.
2. Execute concrete work through local tools or `apex-harness-bridge` when a CLI/MCP boundary is needed.
3. Run `apex-fusion-engine --mode selftest` to verify evolver/autoresearch/superpowers/harness/dawn gates together.
4. Run `apex-evidence-validator` before admitting claims into memory.
5. Update `state/phi_v10_result.json` only from real artifacts, never from invented scores.

Activation command:

```bash
cd /Users/lihongxin/.openclaw/workspace/scripts/apex-praison-chain
./apex-praison-chain --mode activate \
  --task "activate APEX formula transmission chain for verified closed-loop work" \
  --process hierarchical \
  --out /Users/lihongxin/.openclaw/workspace/state/apex-praison-activation.json
```

Required evidence outputs:

```text
state/apex-praison-activation.json
state/apex-fusion-engine-latest.json
state/apex-fusion-evidence-report.json
```

## CLI

```bash
cd /Users/lihongxin/.openclaw/workspace/scripts/apex-praison-chain

# Generate full chain plan
./apex-praison-chain --task "修复猎食器并固化skill" --process hierarchical

# Emit only role agents
./apex-praison-chain --mode roles --task "PHN biomarker validation"

# Emit score only
./apex-praison-chain --mode score --task "吸收GitHub repo为本地skill"

# Activate full APEX transmission plan
./apex-praison-chain --mode activate --task "闭环执行并证据入库" --process hierarchical

# Print skill summary
./apex-praison-chain --mode skill
```

## Operating Discipline

1. Planner defines outputs and acceptance criteria.
2. Builder changes files or produces artifacts.
3. Critic verifies with build/test/lint/direct inspection.
4. Durable lessons go to `memory/` or `skills/` only when they will help future runs.
5. Do not treat chain score as proof; evidence comes from artifacts and verification commands.
