---
name: apex-unified-research-engine
description: "Unified APEX research engine integrating UI control, local agent training, and autonomous multi-agent scientific discovery workflows."
---

# APEX Unified Research Engine

Use when a task needs an end-to-end research pipeline: precise UI/tool control, local agent training/evaluation, literature/resource synthesis, hypothesis generation, experiment planning, analysis, and publishable structured outputs.

## Contract

- Evidence first. Do not fabricate research results.
- Separate hypotheses, plans, simulations, and verified findings.
- Use local mock/sandbox workspaces before touching real data.
- Use multi-agent critique for high-stakes scientific claims.
- Store project artifacts under `research/apex`.
- Python is glue only; deterministic core should be Rust/Go/C.

## Unified formula

`Engine_APEX = (Coord_Fix × Token_Control) × (Task_Syn + Train_SFT/RL + Bench_Verify) × (ERA + CoScientist + Robin)`

## Modules

### 1. UI control precision

- Coordinate correction:
  `X_real = X_out * W_screen / W_img`
  `Y_real = Y_out * H_screen / H_img`
- Token control:
  `Token_keep = TextToken + Latest_3_ImgToken`

### 2. Local agent training

- `Task = Persona × Skill × MockWorkspace`
- `Agent = SFT + RL × SandboxParallel`
- `Score = AutoVerify + LLM-HumanVerify`

### 3. Autonomous research discovery

- `ERA = LLM × TreeSearch × CodeSandbox`
- `CoScientist = Gen + Rank + Reflect + Evolve × Memory`
- `Robin = Hypo + Plan + Exp + Analyze × Mechanism`

## Workflow

1. Define research question and boundaries.
2. Build evidence ledger: local docs, papers, datasets, code, web sources.
3. Generate hypotheses and rank by novelty, feasibility, risk, and evidence.
4. Plan experiments/simulations in sandbox.
5. Run analysis or code only on approved/local data.
6. Critique with multi-agent roles.
7. Produce structured output: protocol, report, paper outline, target/drug/algorithm candidate, or negative finding.
8. Store outputs and update memory/skills only for durable lessons.
