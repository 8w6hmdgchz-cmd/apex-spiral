---
name: apex-ecc-runtimeos
description: ECC RuntimeOS / Agent Harness for long-running controlled APEX engineering tasks. Use when work needs Skills, Memory, Hooks, Rules, Multi-agent, Session State, Security, Observability, Governance, and Learning to close through fusion/evidence gates.
metadata: { "openclaw": { "emoji": "🧬", "requires": { "bins": ["go", "git"] } } }
---

# APEX ECC RuntimeOS Skill

## Purpose

ECC is the Agent Harness layer for long-running, stable, controllable engineering work.

It treats APEX as AI Agent Infrastructure, not prompt engineering:

```text
Runtime + Plugin Architecture + Evidence Gates + Governance + Learning Loop
```

## Native Domains

- Skills
- Memory
- Hooks
- Rules
- Multi-agent
- Session State
- Security
- Observability
- Governance
- Learning

## Core Rule

No fake progress:

- no fabricated PHI
- no memory admission without evidence schema
- no “refactor complete” without build/test/harness output
- no destructive/external action without explicit approval

## CLI

```bash
cd /Users/lihongxin/.openclaw/workspace/scripts/apex-ecc-runtimeos

go build -o apex-ecc-runtimeos .

./apex-ecc-runtimeos --mode audit \
  --root /Users/lihongxin/.openclaw/workspace \
  --out /Users/lihongxin/.openclaw/workspace/state/apex-ecc-runtimeos-latest.json

./apex-ecc-runtimeos --mode cycle \
  --root /Users/lihongxin/.openclaw/workspace \
  --out /Users/lihongxin/.openclaw/workspace/state/apex-ecc-runtimeos-latest.json
```

## Nightly Governance

Nightly refactor is allowed only as incremental reversible work:

1. Run ECC cycle.
2. Run fusion engine.
3. Run evidence validator.
4. Run hygiene classifier.
5. Update full PHI mirror.
6. Commit and push only intentional source/evidence changes.
7. Leave destructive or external high-risk actions for explicit human approval.

## Upgrade Path

Do not rewrite everything at once. Upgrade one domain at a time:

```text
observe → plan → implement small module → verify → evidence admit → commit → repeat
```
