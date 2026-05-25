---
name: swrs-memory-skill
description: "Hippocampal SWRs-inspired memory consolidation: score experiences, replay important traces, and persist stable long-term memories."
---

# SWRs Memory Skill

Use when experiences, decisions, lessons, preferences, skill updates, or repeated failures should persist beyond the current conversation.

## Contract

- Do not save everything. Score and select.
- Do not save secrets unless explicitly asked.
- Daily/raw memory and long-term/curated memory are different layers.
- Consolidate important traces through replay, not blind append.
- Python is glue only; durable scoring/RingBuffer core should be Rust/Go/C.

## Biological analogy

- Experience input → hippocampal encoding.
- Sharp-wave ripples (SWRs) select salient traces.
- Replay consolidates into neocortex.
- Stable memory supports future retrieval.

## AI mapping

- Conversation/tool result → temporary trace.
- Importance scorer selects meaningful traces.
- Replay/merge writes durable memory.
- Future tasks retrieve and reuse memory.

## State equation

`h_t = F(h_{t-1}, x_t, u_t)`

- `h_t`: current memory state.
- `x_t`: new experience trace.
- `u_t`: update gate from importance, novelty, emotion/urgency, user intent, and replay stability.

Prediction/retrieval:

`x_{t+1} = G(h_t)`

## Memory layers

- `memory/YYYY-MM-DD.md`: raw daily event log.
- `MEMORY.md`: curated long-term memory.
- `skills/*/references/trajectories.md`: skill-specific replay traces.
- `memory/swrs-ring.jsonl`: bounded replay buffer.

## Selection signals

Save when at least one is strong:

- explicit user says remember/save
- durable preference or identity
- important project decision
- repeated error/fix pattern
- external integration details
- future todo/reminder-like context
- high-cost discovery that should not be repeated

Avoid saving:

- one-off chatter
- sensitive secrets/tokens
- unverified speculation
- raw private content not needed later

## Consolidation workflow

1. Encode trace: who/what/decision/evidence/date/source.
2. Score: importance, novelty, recurrence, future utility, sensitivity risk.
3. Store selected trace in `memory/swrs-ring.jsonl`.
4. Replay periodically: merge high-fitness traces into `MEMORY.md` or relevant skill references.
5. Prune/merge stale or duplicate memories.
6. Verify before reporting memory-derived claims.
