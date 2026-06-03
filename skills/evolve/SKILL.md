---
name: evolve
description: Self-improvement loop. Reads telemetry, identifies weak sub-skills, generates mutations, tests them, keeps the winners. Inspired by modelscope/AgentEvolver, gumyn/skill-evolver, bigknoxy/opencode-skill-evolution, GEPA / EvoMap / SkillClaw.
trigger: background, every N invocations (default 20), "optimize the skill", "evolve"
priority: medium
tier: meta
depends-on: []
---

# evolve

> **Skills that don't get used, get mutated. Skills that don't work, get killed.**

## When To Use

- Background: every 20 invocations of the meta-skill (configurable).
- Manual: when the user says "optimize" or "the skill feels off".

## The GEPA-style Loop

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  COLLECT │───▶│  ANALYZE │───▶│  MUTATE  │───▶│   TEST   │
│ telemetry│    │ weak     │    │ generate │    │ run on   │
│          │    │ skills   │    │ N        │    │ holdout  │
└──────────┘    │          │    │ variants │    │          │
     ▲          └──────────┘    └──────────┘    └────┬─────┘
     │                                              │
     │            ┌──────────┐                      │
     └────────────│  SELECT  │◀─────────────────────┘
                  │ keep best│
                  └──────────┘
```

### Step 1 — Collect

Read `~/.apex-skill/logs/telemetry.jsonl`. For each sub-skill, compute:

- **invocations**: count of `event: pre-tool, tool: <skill>` (or dispatch events)
- **success_rate**: `post-tool status=ok / pre-tool`
- **avg_latency_ms**: mean of `duration_ms`
- **user_satisfaction_proxy**: did the user re-invoke the same skill within 3 turns? (proxy for "did this help")

### Step 2 — Analyze

Rank skills by `success_rate`. Bottom 20% are candidates for mutation.

For each weak skill, identify:
- **failure mode**: e.g., "asks too many questions" / "produces too-long output" / "no verifiable test"
- **mutation budget**: generate ≤5 variants per skill per cycle

### Step 3 — Mutate

Mutation operators:

1. **Trim** — remove a section, see if success_rate improves.
2. **Add example** — add a 5-line worked example.
3. **Tighten wording** — replace hedging ("you might want to") with imperatives.
4. **Add a check** — add a checklist item to the output contract.
5. **Split** — split a long SKILL.md into two smaller skills.

Never mutate `using-apex-skill` (the dispatcher) — its job is routing, not answering.

### Step 4 — Test

For each variant, run the **holdout suite** in `tests/holdout/`:
- 3-5 representative user requests
- Compare output against the original skill
- Score: passes the verification (lint+typecheck+test) and is shorter/clearer

### Step 5 — Select

Keep the variant with the highest holdout score. Discard the rest. Commit with a clear message: `evolve: trim brainstorm by 12% (holdout 4/5 → 5/5)`.

## Anti-Patterns

- ❌ Don't mutate without telemetry (you'll make it worse).
- ❌ Don't mutate more than 1 skill per cycle (chaos).
- ❌ Don't keep a variant that fails any holdout case.
- ❌ Don't mutate `using-apex-skill` (meta-skill is sacred).
- ❌ Don't run evolve in the same turn as `execute-plan` (different concerns).

## Output Contract

```yaml
evolve:
  cycle: <n>
  candidates: [<skill>, ...]
  mutations_tested: <int>
  kept: {<skill>: <variant_id>, ...}
  discarded: <int>
  next_skill: <none — runs in background>
```
