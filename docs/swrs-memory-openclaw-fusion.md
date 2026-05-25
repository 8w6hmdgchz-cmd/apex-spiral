# SWRs Memory × OpenClaw/Hermes-Agent Fusion

## Goal

Transform conversation/tool experiences into durable, selected, replayed memory instead of unfiltered logs.

## Mechanism

Biological mapping:

1. Experience input → hippocampal encoding.
2. SWRs select important experiences.
3. Replay consolidates to neocortex.
4. Stable memory supports future behavior.

AI mapping:

1. Dialogue/tool event → temporary trace.
2. Importance scoring chooses candidate traces.
3. RingBuffer stores replay candidates.
4. Periodic consolidation writes `MEMORY.md`, `memory/YYYY-MM-DD.md`, or skill trajectories.
5. Retrieval reuses memory for future tasks.

## Formula

Memory update:

`h_t = F(h_{t-1}, x_t, u_t)`

Prediction/retrieval:

`x_{t+1} = G(h_t)`

Update gate:

`u_t = score(importance, novelty, future utility, recurrence, explicit user intent, sensitivity risk)`

## OpenClaw integration

- `memory/YYYY-MM-DD.md`: daily raw log.
- `MEMORY.md`: curated stable memory.
- `memory/swrs-ring.jsonl`: bounded replay buffer.
- `skills/swrs-memory-skill/references/memory-policy.md`: scoring policy.
- `crates/swrs_memory`: Rust deterministic scorer and RingBuffer prototype.
- Cron job: periodic replay/consolidation.

## Relationship with Emv Entropy Skill

- Emv distills skills from long context.
- SWRs decides which experiences/skills deserve durable memory.
- Replay prevents skill drift and memory bloat.

## Honesty boundary

This does not change model weights. It adds explicit memory selection, replay, and consolidation around OpenClaw's existing file/session/cron mechanisms.
