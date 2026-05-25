# SearchSkill × OpenClaw/Hermes-Agent Fusion Plan

## Goal

Turn retrieval from implicit random search into a controllable, learnable, auditable skill loop:

`Select skill card → Read rule → Act with tools → Verify evidence → Synthesize → Evolve SkillBank`

This document records the fusion design requested in `超级进化3-深度自进化` and maps it to current OpenClaw primitives.

## Current integration points

- OpenClaw skills: `skills/search-skill/SKILL.md` exposes trigger and workflow.
- SkillBank: `skills/search-skill/references/skillbank.md` stores reusable atomic search skills.
- Trajectory memory: `skills/search-skill/references/search-trajectories.md` stores successful/failed retrieval patterns.
- A2A auto evolution: `/Users/lihongxin/.openclaw/workspace/a2a-auto-trigger.sh` runs fetch → absorb → hunt daily.

## Runtime trigger policy

Use SearchSkill when any condition is true:

- Fresh/current/external fact is needed.
- Multi-hop search or source verification is needed.
- GitHub/repo/package/entity discovery is needed.
- User asks about prior work or local decisions: run context_backtrack/memory before answering.
- A search/API fails: invoke failure_recover instead of stopping after one failed path.

## APEX mapping

- Λ: source authority and coverage.
- Θ: task relevance of selected search card.
- K: cross-source knowledge fold; number and diversity of verified evidence paths.
- ξ: query precision; low ξ triggers keyword_expand.
- Ψ: verification strength; low Ψ triggers multi_source_verify.
- Φ: synthesis fidelity; low Φ triggers context_backtrack and quote extraction.
- H/T/ε: hallucination, time, and cost penalties.

ΔG improves when the system increases source coverage and verification while reducing failed searches and hallucinated claims.

## Hermes-Agent / OpenClaw alignment

OpenClaw already provides the primitives needed for Hermes-Agent style evolution:

- Skills = modular policy cards loaded on demand.
- Tools = Act layer (`web_search`, `web_fetch`, `read`, `memory_search`, `exec`, etc.).
- Cron = durable autonomous trigger for scheduled evolution.
- Sessions/subagents = parallel decomposition for larger search tasks.
- Memory files = long-term trajectory consolidation.

Do not claim hidden gateway/model weights changed. Fusion is implemented as skill policy, trajectory storage, and deterministic scripts around existing OpenClaw mechanisms.

## Core language rule

Python is acceptable for glue and prototypes. Durable high-throughput components should be implemented in C/Go/Rust:

- Go: concurrent search orchestration, HTTP fetching, cron-friendly CLIs.
- Rust: fast parsers, ranking/scoring, local vector/index engine, safe file normalization.
- C: only for narrow performance-critical native kernels where Rust/Go are unsuitable.

## Next durable components

1. `searchskill-runner` in Go: Select-Read-Act CLI over SkillBank cards.
2. `skillbank-index` in Rust: local card/trajectory index with scoring.
3. `a2a-normalizer` in Rust or Go: canonicalize typo files (`pending. list`, `pending.`) into `pending.list` safely.
4. `evidence-ledger` in Go: JSONL evidence store with source, claim, confidence, timestamp.

## Validation gates

- Skill frontmatter parses as YAML.
- SearchSkill can be selected from metadata.
- A2A auto trigger runs fetch → absorb → hunt without leaving pending items.
- Every reported external claim has source evidence or an uncertainty marker.
