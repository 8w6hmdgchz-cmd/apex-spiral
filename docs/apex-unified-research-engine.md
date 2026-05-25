# APEX Unified Research Engine

## Core position

A single research engine that integrates:

1. high-precision UI/tool operation
2. local agent training/evaluation
3. multi-agent autonomous research evolution

It is designed to be formulaic, reproducible, auditable, and locally archived.

## Master formula

`APEX = 操控精度 × 训练闭环 × 科研自主`

`Engine_APEX = (Coord_Fix × Token_Control) × (Task_Syn + Train_SFT/RL + Bench_Verify) × (ERA + CoScientist + Robin)`

## Current implemented dependencies

- APEX Token Root Fix:
  - Rust `crates/apex_token_optimizer`
  - browser CLI coordinate hook
  - screenshot latest-3 ring
- ClawG Training Ecosystem:
  - Rust `crates/clawg_bench`
  - mock workspace task generation
  - AutoVerify + LLM/Human weighted score
- SearchSkill:
  - Select-Read-Act retrieval skillbank
- Emv Entropy Skill:
  - Challenger/Reasoner/Judge skill distillation
  - Rust Gini/Entropy selector
- SWRs Memory:
  - Rust scorer/RingBuffer
  - daily consolidation
- A2A auto evolution:
  - fetch → absorb → hunt

## Scientific discovery module mapping

ERA:

`LLM × TreeSearch × CodeSandbox`

- generate candidate algorithms/experiments
- branch and score search paths
- run code in local sandbox

CoScientist:

`Gen + Rank + Reflect + Evolve × Memory`

- propose hypotheses
- rank by evidence/novelty/feasibility
- reflect on weaknesses
- evolve better hypotheses using memory

Robin:

`Hypo + Plan + Exp + Analyze × Mechanism`

- hypothesis formation
- experimental plan
- execution/simulation
- analysis and mechanism explanation

## Storage layout

`research/apex/projects/<project_id>/`

- `protocol.md`
- `evidence-ledger.jsonl`
- `hypotheses.json`
- `experiment-plan.md`
- `analysis-report.md`
- `paper-outline.md`
- `run.json`

## Honesty boundary

The system can plan, search, analyze, simulate, and write structured scientific reports. It must not claim real experimental discovery, drug efficacy, or publishable truth without actual data and evidence.
