---
name: brainstorm
description: Design-before-code. Captures user intent, asks 1-3 sharp Socratic questions, then proposes 2-3 concrete designs with tradeoffs. Inspired by obra/superpowers, bevibing/socrates-skill, karpathy surgical engineering.
trigger: open-ended request, ambiguous goal, "help me design", "I want to build"
priority: high
tier: think
depends-on: []
---

# brainstorm

> **No code until 2-3 designs are on the table.**

## When To Use

- User said "I want to build / design / create" without a clear spec.
- The task has more than one reasonable architecture.
- The user is exploring (not yet committing).

## Procedure (Karpathy + Socratic + superpowers)

### Phase 1 — Understand the goal (Socratic, ≤3 questions)

Ask only what you cannot infer. Prefer multi-choice:

```
To design well, I need to know:
  1. <one question>?  (a) ..., (b) ..., (c) ...
  2. <one question>?  (a) ..., (b) ...
  3. <one question>?  (a) ..., (b) ...
```

**Rules** (from `bevibing/socratic-skill`):
- Never answer for the user.
- One question at a time if it's deep; all three at once if it's shallow.
- If you can already answer the question from context, skip it.

### Phase 2 — Propose 2-3 designs (superpowers + karpathy)

For each design, fill the **Design Card** below. Be terse. Prefer ASCII boxes for structure.

```markdown
## Design <letter>: <name>

**One-line summary:** ...

**Architecture (ASCII):**
┌────────┐    ┌────────┐
│ ...    │───▶│ ...    │
└────────┘    └────────┘

**Pros:** ...
**Cons:** ...
**Complexity:**  S / M / L
**Reversibility:** easy / medium / hard
**Lines of code (rough):** ...
```

### Phase 3 — Recommend (karpathy "think first, decide once")

Pick one. Justify in 2 sentences. State the **next action** in one sentence.

> "I recommend **Design B** because ... After you confirm, the next skill is `write-plan`."

## Anti-Patterns

- ❌ Don't ask more than 3 questions.
- ❌ Don't propose 4+ designs (decision fatigue).
- ❌ Don't start coding in this skill — that's `execute-plan`'s job.
- ❌ Don't propose designs you can't justify with a tradeoff.

## Output Contract

The output of this skill is a `Decision` object:

```yaml
decision:
  goal: <one-line>
  design: <letter>
  rationale: <≤2 sentences>
  next_skill: write-plan | execute-plan
```
