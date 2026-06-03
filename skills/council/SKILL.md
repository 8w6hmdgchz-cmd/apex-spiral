---
name: council
description: Multi-model debate. Run a decision through N (default 3) personas/models, synthesize, surface the dissent. Inspired by karpathy's "council" idea, ag0.xyz Agent0 (multi-agent game theory), GitHub Agentic Workflows (parallel reviewers).
trigger: high-stakes decision, "should I choose A or B?", architectural call
priority: medium
tier: think
depends-on: []
---

# council

> **One model's opinion is a guess. Three models arguing is a decision.**

## When To Use

- Architectural decision (library, language, pattern).
- High-stakes choice (refactor, migration, deprecation).
- User explicitly asks for a multi-perspective view.

## Procedure

### 1. Frame the decision

One paragraph, neutral:

> "We must choose between A and B. Context: <constraints>. Stakeholders: <who>. Reversibility: <easy|hard>."

### 2. Convene the council

Default 3 personas, configurable:

| Persona | Bias | Cares about |
|---|---|---|
| **The Pragmatist** | "ship it" | time-to-value, simplicity, maintenance burden |
| **The Purist** | "do it right" | correctness, future-proofing, type safety |
| **The Skeptic** | "what could go wrong?" | edge cases, failure modes, security |

If you have access to multiple model APIs, dispatch the same prompt to all three and compare. If not, simulate the three personas in your own thinking and label them clearly.

### 3. Run the debate

Each persona gives a 3-paragraph opinion **on the record** (so the user can see the dissent). Then:

- Where do they agree? (Strong signal.)
- Where do they disagree? (Real tradeoff — surface to the user.)

### 4. Synthesize

Write a **minority opinion** section. The user gets:

```markdown
## Council Verdict

**Majority view:** ... (2 of 3 personas prefer B because ...)

**Minority view:** ... (1 of 3 — The Purist — prefers A because ...)

**My recommendation:** ... (one sentence, with which side and why)

**Reversibility cost of being wrong:** ... (low / medium / high)
```

### 5. Defer to the user

For high-stakes decisions, do **not** auto-pick. Present the verdict and ask:

> "Council leans B, but the Purist's minority view is non-trivial. Do you want to go with B, A, or dig deeper?"

## Anti-Patterns

- ❌ Don't convene the council for trivial choices (overhead > benefit).
- ❌ Don't hide the dissent — the user deserves the minority view.
- ❌ Don't always pick the majority (sometimes the minority is right).
- ❌ Don't skip reversibility — a reversible decision doesn't need a council.
- ❌ Don't impersonate personas dishonestly (label them clearly).

## Output Contract

```yaml
council:
  decision: <one-line>
  personas:
    - {name, model, opinion: <paragraph>}
  majority: ...
  minority: ...
  recommendation: ...
  reversibility_cost: low | medium | high
  next_skill: brainstorm (if user wants to re-frame) | write-plan (if user picks)
```
