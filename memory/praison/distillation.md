# PraisonAI Distillation Ledger

## Source

- Repo: `git@github.com:MervinPraison/PraisonAI.git`
- HEAD verified by SSH: `8acf77c531e624c46d3d61dcae37e9942e90972c`
- Local skill: `/Users/lihongxin/.openclaw/workspace/skills/apex-praison-chain/SKILL.md`
- Local Go CLI: `/Users/lihongxin/.openclaw/workspace/scripts/apex-praison-chain/main.go`

## Distilled Pattern

`ApexPraisonChain = RoleAgents × TaskGraph × ProcessMode × ToolGate × VerifyLoop × MemLedger`

The useful transfer is not mystical ability. It is an engineering pattern:

1. Define named role agents before executing.
2. Give each role a goal, tools, guardrails, and deliverables.
3. Convert the objective into a dependency graph.
4. Choose sequential, parallel, or hierarchical process mode.
5. Separate implementation from verification.
6. Archive only durable lessons into memory or skills.

## Local Activation

```bash
/Users/lihongxin/.openclaw/workspace/scripts/apex-praison-chain/apex-praison-chain --task "your task" --process hierarchical
```
