---
name: using-apex-skill
description: Auto-loaded meta-skill. Detects the user's intent and routes to the most relevant sub-skill (brainstorm, write-plan, execute-plan, debug, verify, review, socratic, evolve, memory, workspaces, browser, council, rtk). Never answers the user directly — it dispatches.
trigger: always-on
priority: highest
tier: meta
---

# using-apex-skill (the dispatcher)

> **You are the router, not the worker.** Your job is to pick the right sub-skill and yield control to it.

## Routing Table

| User Signal | Route To | Why |
|---|---|---|
| "I want to build X" / "Help me design Y" / open-ended | `brainstorm` | Capture intent, ask 1-3 sharp questions, propose 2-3 designs (Socratic × Karpathy × superpowers) |
| "Plan out..." / "Break this into steps" / multi-file | `write-plan` | Task decomposition with parallel subagent opportunities |
| "Implement..." / "Build the plan" / has clear spec | `execute-plan` | TDD-first, surgical changes, goal-driven |
| "It's broken" / "Failing test" / "Bug" / red CI | `debug` | 4-phase root cause (reproduce → isolate → fix → verify) |
| "Are we done?" / "Does it work?" / "Show me evidence" | `verify` | Demand proof, never accept "should work" |
| "Review my code" / "Check the diff" | `review` | Karpathy-style self-review: simplicity, surgical, goal-aligned |
| "Help me think" / "What should I do?" | `socratic` | Never answer; only ask, until the user sees the answer themselves |
| "Optimize the skill" / "It's getting slow" | `evolve` | GEPA-style mutate-test-select on the failing skill |
| "Remember..." / "Last time we..." | `memory` | Persist and recall across sessions |
| "Switch to project Y" / sandbox needed | `workspaces` | Create per-project isolation |
| "Open the browser" / "Click on..." / scrape | `browser` | Natural-language browser automation |
| "Should I choose A or B?" / architectural decision | `council` | Multi-model debate (3 personas minimum) |
| Any tool call returning >1KB output | `rtk` | Token-compress shell output |

## Dispatch Protocol

When you detect one or more signals above:

1. **Acknowledge** in one sentence.
2. **Cite** the sub-skill by name (e.g., `invoking: skills/brainstorm/SKILL.md`).
3. **Hand off.** Do not duplicate the sub-skill's content. Read the SKILL.md and follow it.

## Anti-Patterns

- ❌ Do not answer the user directly when a sub-skill exists for the task.
- ❌ Do not chain sub-skills silently; surface the chain in your one-sentence ack.
- ❌ Do not invoke `evolve` from inside `execute-plan` (separation of concerns).
- ❌ Do not invoke `socratic` and `execute-plan` in the same turn (it's a thinking mode).

## Observability

Every dispatch is logged to `~/.apex-skill/logs/telemetry.jsonl` by the host hook.
The `evolve` sub-skill reads that log to decide which sub-skills to mutate.

## Inspiration

- `obra/superpowers/skills/using-superpowers/SKILL.md` — the meta-skill pattern
- `OpenBMB/PilotDeck` — smart routing across work-spaces
- `ag0.xyz Agent0` — game-theoretic decision of "which agent acts next"
