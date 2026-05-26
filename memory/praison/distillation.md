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

## 2026-05-26 Activation Upgrade

Praison chain is now a background-callable APEX transmission skill, not only a planner.

New activation mode:

```bash
scripts/apex-praison-chain/apex-praison-chain --mode activate \
  --task "activate APEX formula transmission chain for verified closed-loop work across any scenario" \
  --process hierarchical \
  --out state/apex-praison-activation.json
```

Activation formula:

```text
ApexPraisonTransmission = APEXFormulaMirror × RoleAgents × TaskGraph × ToolGate × FusionGate × EvidenceAdmission × MemLedger
```

Verification evidence:

- `state/apex-praison-activation.json`: activation status `active`, composite score `0.854`
- `state/apex-fusion-engine-latest.json`: fusion status `success`, gates `5/5`
- `state/apex-fusion-evidence-report.json`: evidence validator `success`, checked `1`, passed `1`

No-virtual-data rule remains binding: activation is only accepted when backed by runnable artifacts and evidence-validator admission.
