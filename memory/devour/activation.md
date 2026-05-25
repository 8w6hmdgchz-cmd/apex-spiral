# APEX Devour Engine Activation

## Status

Activated as a local Go-backed OpenClaw skill.

- Skill: `/Users/lihongxin/.openclaw/workspace/skills/apex-devour-engine/SKILL.md`
- CLI: `/Users/lihongxin/.openclaw/workspace/scripts/apex-devour-engine/apex-devour-engine`
- Source: `/Users/lihongxin/.openclaw/workspace/scripts/apex-devour-engine/main.go`

## Definition

Devour = discover -> rank -> install -> audit -> distill -> reimplement -> verify -> archive.

## APEX Agentic RL Formula

```text
RL_base = π(a|s) -> R -> ∇π
APEX_ARL = RL ∪ {MetaG, Reflect, LongPlan}
I_total = M_base × C_think
C_think = G_set + P_decompose + S_review
ApexAgent ⊃ AgenticRL ⊃ StandardRL
```

## No-Fake-Data Rule

The engine intentionally leaves stars/commits/benchmarks empty unless they come from explicit evidence or installed source. This is the hard boundary preventing the old self-narrative problem.

## Verification

```bash
cd scripts/apex-devour-engine
go build -o apex-devour-engine .
./apex-devour-engine --mode formula
./apex-devour-engine --need "CLI MCP bridge for APEX harness" --mode score
```
