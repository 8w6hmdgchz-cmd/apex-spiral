---
name: emv-entropy-skill
description: "Multi-agent self-play skill distillation from long context using Challenger/Reasoner/Judge, replay, entropy, and Gini-gain selection."
---

# Emv Entropy Skill

Use when a long/complex document should become reusable natural-language skills instead of being answered from raw context once.

## Contract

- Convert long context into reusable skill cards.
- Use Challenger → Reasoner → Judge self-play.
- Use cross-time replay to prevent adversarial collapse and overfitting.
- Select skill mutations by entropy/information gain/Gini gain.
- Store reusable lessons in `references/emv-skillbank.md`.
- Python only as glue/prototype. Durable scoring/selection core should be Rust/Go/C.

## Loop

1. **Extract** candidate claims, procedures, formulas, constraints, failure cases.
2. **Challenger** creates tasks that expose missing or brittle skills.
3. **Reasoner** solves using existing skill cards, not raw full context when avoidable.
4. **Judge** scores correctness, grounding, transferability, cost, and failure mode.
5. **Replay** mixes new tasks with older trajectories to avoid collapse.
6. **Mutate** skill cards: add, merge, split, prune, compress.
7. **Select** mutation path with Gini/Entropy gain.
8. **Commit** high-fitness cards and record trajectory.

## Fitness

Prefer skills with:

- high transfer to new tasks
- readable natural-language procedure
- lower context cost
- fewer hallucinated claims
- verified examples
- stable replay performance

## Output

For each distilled skill:

- `name`
- `trigger`
- `procedure`
- `verification`
- `failure_recovery`
- `fitness_notes`

If the request is about OpenClaw/Hermes integration, also update `docs/emv-entropy-openclaw-fusion.md` when design changes.
