---
name: apex-tiangong-skill
description: "APEX TianGong hidden meta-skill: unified Rust core for cognition, planning, sandbox execution, verification, and evolution."
---

# APEX TianGong Skill

Use when the user asks to activate TianGong, devour/self-evolution, quantum routing, GitHub capability ingestion, or integration of evolver/autoresearch/openhands/superpowers into the APEX ecosystem.

## Safety Contract

- Local-first. Do not publish, push, gist, email, or exfiltrate without explicit user approval.
- Evidence-first. Separate verified results from hypotheses and plans.
- Devour means learn/abstract/reimplement safely, not blindly copy incompatible code.
- Secrets are blockers. Any `secret_hits > 0` stops external sync.
- Python is orchestration glue; deterministic capability cores live in Rust.
- All new capabilities must pass local verification before promotion.

## Formula

```text
ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)

TianGong_APEX = tiangong_core(
  cognition + planning + sandbox + verification + evolution
)

ClosedLoop = Cognition → Planning → Execution → Verification → Evolution → SWR
```

## Unified Core

Canonical Rust core:

```text
skills/apex-tiangong-skill/tiangong_core
```

Build/test:

```bash
cd skills/apex-tiangong-skill/tiangong_core
cargo test
cargo build
```

CLI:

```bash
skills/apex-tiangong-skill/tiangong_core/target/debug/tiangong_core selftest
skills/apex-tiangong-skill/tiangong_core/target/debug/tiangong_core sandbox <cmd...>
skills/apex-tiangong-skill/tiangong_core/target/debug/tiangong_core evolver <phase> '<json>'
skills/apex-tiangong-skill/tiangong_core/target/debug/tiangong_core cognition '<json>'
skills/apex-tiangong-skill/tiangong_core/target/debug/tiangong_core gate <requirements|architecture|test_plan|review|full> '<json>'
```

Python glue:

```text
skills/apex-tiangong-skill/tiangong_native.py
```

The Python native loop now routes all four core domains through `tiangong_core`.

## Four Pillars

### 1. Evolver / GEP Self-Drive

Purpose: scan local artifacts, detect defects, produce evolution maps, and consolidate assets.

Unified core route:

```bash
tiangong_core evolver observe '<json>'
tiangong_core evolver act '<json>'
tiangong_core evolver verify '<json>'
tiangong_core evolver repair '<json>'
tiangong_core evolver consolidate '<json>'
```

APEX hooks:

- `apex-github-evolution/scripts/evomap_audit.py`
- `apex-github-evolution/scripts/evolver_local.sh`
- `apex-github-evolution/scripts/create_safe_export.py`

### 2. AutoResearch / Cognition Frontend

Purpose: retrieve, rank, distill, and challenge knowledge before engineering.

Unified core route:

```bash
tiangong_core cognition '<json>'
```

Local hooks:

- `apex-unified-engine/py/research/era.py`
- `apex-unified-engine/py/research/co_scientist.py`
- `apex-unified-engine/py/research/robin.py`
- `skills/apex-tiangong-skill/github_k_ingest.py`

External hooks, only when network and approval permit:

- GitHub search/API for high-star reference projects
- papers/docs/web sources

### 3. OpenHands-Style Sandbox Execution

Purpose: perform file, terminal, test, and browser-like implementation loops in a local audited sandbox.

Unified core route:

```bash
tiangong_core sandbox python3 --version
```

Policy:

- Work under the OpenClaw workspace.
- Prefer throwaway/mock directories for risky experiments.
- Never mutate user secrets or external services without approval.
- Dangerous commands/args are blocked by Rust policy.

### 4. Superpowers Engineering Gate

Purpose: enforce the engineering loop.

Unified core route:

```bash
tiangong_core gate full '<json>'
```

Required stages:

1. Requirement boundary
2. Architecture sketch
3. Task decomposition
4. TDD or smallest verification gate
5. Implementation
6. Self-review / audit
7. Structured delivery

## Legacy Split Cores

These crates are historical prototypes and are kept only for reference:

- `tiangong_sandbox_rs`
- `tiangong_evolver_rs`
- `tiangong_cognition_rs`
- `tiangong_superpowers_rs`

Use `tiangong_core` for new work.

## Activation Workflow

1. Score current state using APEX formula dimensions: Λ Θ K ξ Ψ Φ H T ε.
2. Identify bottleneck: usually K for missing knowledge, Φ for verification, ξ for context drift.
3. Select route:
   - Knowledge gap → `cognition` + GitHub K ingest
   - Execution gap → `sandbox`
   - Process gap → `gate`
   - Evolution gap → `evolver`
4. Run local TianGong orchestrator.
5. Promote only if `fitness >= 0.7` and safety gate passes.
6. Write durable learnings into skills/memory only after verification.

## Commands

Run APEX-level orchestrator:

```bash
python3 skills/apex-tiangong-skill/tiangong_orchestrator.py "<objective>"
```

Run native four-core loop:

```bash
python3 skills/apex-tiangong-skill/tiangong_native.py "<objective>"
```

Run GitHub K ingestion:

```bash
python3 skills/apex-tiangong-skill/github_k_ingest.py
```

Run tests:

```bash
cd skills/apex-tiangong-skill/tiangong_core && cargo test
python3 skills/apex-tiangong-skill/test_tiangong_native.py
```

Outputs:

- `state/tiangong/latest.json`
- `state/tiangong/native/latest.json`
- `state/tiangong/github_k/latest.json`
