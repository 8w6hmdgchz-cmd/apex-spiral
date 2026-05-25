---
name: search-skill
description: "SkillBank-driven Select-Read-Act search planning for multi-hop retrieval, verification, and source-grounded synthesis."
---

# SearchSkill

Use when a task needs fresh facts, multi-hop lookup, source verification, entity tracing, time-bounded search, or answer grounding.

## Contract

- Do not rely on implicit one-shot search for complex questions.
- Run Select → Read → Act → Verify → Synthesize.
- Prefer local docs/files first for OpenClaw behavior; use web only for external/current facts.
- Every external claim needs source evidence or must be marked uncertain.
- Python is allowed only as glue/prototyping. Durable core helpers should be C/Go/Rust when implemented.

## Select

Choose one or more skill cards from `references/skillbank.md`:

- `keyword_expand`: broaden query terms and aliases.
- `entity_trace`: identify canonical names, repos, authors, orgs, releases.
- `time_bound`: restrict by date/version/event window.
- `multi_source_verify`: cross-check at least two independent sources.
- `context_backtrack`: search previous context, local docs, memory, or repo before web.
- `failure_recover`: rewrite query/source when results are empty, blocked, duplicated, or stale.

## Read

Open only the needed card(s). Extract:

- trigger condition
- query template
- stop condition
- verification rule
- output format

## Act

1. Build 2-4 precise queries.
2. Search/fetch with smallest sufficient source set.
3. If a result is weak, retry with a different query/source.
4. Keep raw notes short: source, claim, confidence, timestamp/version.
5. Synthesize answer with citations when helpful.

## SkillBank evolution

After a search-heavy task, append a compact entry to `references/search-trajectories.md` when useful:

- problem type
- selected skill cards
- queries/sources that worked
- failure mode
- reusable improvement

Promote repeated successful patterns into `references/skillbank.md`. Remove or rewrite cards that repeatedly cause low-quality results.
