---
name: apex-devour-engine
description: APEX 吞噬自进化引擎。Use when the user asks to learn from high-star open-source projects, install/audit/distill/reimplement capabilities, or activate Agentic RL-style self-evolution with no virtual data.
metadata: { "openclaw": { "emoji": "🧬", "requires": { "bins": ["go", "git"] } } }
---

# APEX Devour Engine Skill

## Contract

Devour means: discover, rank, install, audit, distill, reimplement, verify, archive.

It does not mean blind copying, fake benchmark claims, license-unsafe reuse, or invented GitHub stars/commits. Unknown fields stay empty until verified.

## Core Formula

```text
RL_base = π(a|s) -> R -> ∇π
APEX_ARL = RL ∪ {MetaG, Reflect, LongPlan}
I_total = M_base × C_think
C_think = G_set + P_decompose + S_review
ApexAgent ⊃ AgenticRL ⊃ StandardRL
```

Five-layer kernel:

```text
G_self != G_env
P_n = Split(G_total)
π_t = f(π_{t-1}, ΔE)
R_meta = Eval(Logic)
S_fix = Error -> Policy
```

## Pipeline

1. `discover`: find candidate repos for the user need.
2. `rank`: select top3 by explicit evidence.
3. `install`: SSH/sparse/shallow install selected repos.
4. `audit`: inspect architecture, license, safety, build surface.
5. `distill`: extract formulas, interfaces, and skill contracts.
6. `reimplement`: rebuild deterministic local core in Rust, C, or Go.
7. `integrate`: add CLI/skill/harness hooks inside OpenClaw.
8. `archive`: update EvoMap, metrics, memory, and failure records.

## CLI

```bash
cd /Users/lihongxin/.openclaw/workspace/scripts/apex-devour-engine

# Print formulas
./apex-devour-engine --mode formula

# Plan a devour cycle
./apex-devour-engine --need "CLI MCP bridge for APEX harness"

# Rank verified candidates from JSON
./apex-devour-engine --mode rank --need "agent framework" --candidates candidates.json

# Emit pipeline only
./apex-devour-engine --mode pipeline --need "OpenHands-style sandbox execution"

# Emit APEX Agentic RL readiness score
./apex-devour-engine --mode score --need "self modification evolver"
```

## Candidate JSON Schema

```json
[
  {
    "repo": "owner/name",
    "url": "git@github.com:owner/name.git",
    "stars": 0,
    "commit": "verified commit hash",
    "local_path": "vendor/github/owner/name",
    "license": "license if verified",
    "evidence": ["source path or command output"],
    "core_signals": ["Agent", "TaskGraph", "MCP", "Sandbox"]
  }
]
```

## Safety Gates

- No virtual stars, commits, benchmarks, or install claims.
- External writes require explicit approval unless committing this workspace's own code.
- Secrets or private data hits block export/push.
- Do not claim superiority until local benchmark proves a measurable delta.
- Every promoted capability needs build/check/smoke-test evidence.
