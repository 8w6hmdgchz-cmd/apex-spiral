# Emv Entropy Skill × OpenClaw/Hermes-Agent Fusion

## Essence

Emv Entropy Skill changes long-context use from “read everything and answer once” to “distill reusable skills through self-play.”

Core loop:

`Long Context → Candidate Skills → Challenger tasks → Reasoner attempts → Judge scores → Replay stability → Gini/Entropy mutation selection → SkillBank commit`

## Agent roles

- Challenger: creates diagnostic tasks from long documents and previous failure modes.
- Reasoner: solves tasks using current skill cards.
- Judge: scores outputs for correctness, evidence, transfer, cost, and robustness.

## Replay

Cross-time replay mixes current tasks with old trajectories. This prevents:

- adversarial collapse
- overfitting to latest document
- skill drift
- catastrophic forgetting

## Formula integration

Gini impurity:

`Gini = 1 - Σ p_k²`

Gini gain:

`ΔGini = Gini_parent - ((N_L/N)Gini_L + (N_R/N)Gini_R)`

Entropy:

`H = -Σ p_k log2(p_k)`

Information gain:

`IG = H_parent - Σ_v (N_v/N)H_v`

Random forest style voting is used as a committee selector over candidate mutations:

- hard vote: choose class/path with most trees
- soft vote: average probabilities and choose max
- bootstrap sampling target: ~63.2% included, ~36.8% OOB

## OpenClaw mapping

- Skill frontmatter: trigger layer.
- `references/emv-skillbank.md`: reusable natural-language skill cards.
- `references/trajectories.md`: replay memory.
- Subagents: Challenger/Reasoner/Judge can run as separate sessions for large jobs.
- Cron: periodic consolidation from recent docs/tasks.
- Rust crate: deterministic scoring and mutation selection.

## Safety and honesty

This integration does not claim hidden model weights changed. It changes operating policy, local skill memory, deterministic scoring, and scheduled consolidation.

## Durable implementation rule

Python may glue tool calls, but deterministic core selection should be Rust/Go/C. Current prototype is Rust in `crates/emv_entropy`.
