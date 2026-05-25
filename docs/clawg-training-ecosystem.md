# ClawG Personal Agent Training Ecosystem

## Goal

Provide a safe, reproducible local training/evaluation substrate for OpenClaw agents handling files, scripts, configs, and state.

## Components

1. Task generator: persona intent + grounded skill + mock workspace.
2. Sandbox rollout: run agent/tool trajectories in isolated workspaces.
3. Benchmark verifier: deterministic AutoVerify + optional LLM/human quality score.

## APEX formulas

`Task_APEX = PersonaIntent × SkillGrounding × MockWorkspace`

- PersonaIntent: user-like goal.
- SkillGrounding: required skill/tool constraints.
- MockWorkspace: safe synthetic files and expected state.

`Agent_APEX = SFT_Trajectory + RL_Rollout × SandboxParallel`

- SFT_Trajectory: supervised trajectories captured from successful runs.
- RL_Rollout: lightweight exploration/evaluation in sandbox.
- SandboxParallel: many isolated tasks can run in parallel.

`Score_APEX = AutoVerify(60%) + LLM_HumanVerify(40%)`

- AutoVerify: file state, exact outputs, tests.
- LLM/Human: intent quality, maintainability, safety.

`Iteration_APEX = Data → Train → Bench → Feedback`

- Data: generated/real approved trajectories.
- Train: future SFT/RL pipeline; current implementation stores training-ready data.
- Bench: deterministic local benchmark.
- Feedback: update skills, memory, docs, task generator.

## Implementation boundary

This implementation creates the substrate and deterministic evaluator. It does not claim actual model fine-tuning or RL training has occurred until a real training job is run.

## Current core

Rust crate: `crates/clawg_bench`

- generates mock file-operation tasks
- creates isolated workspaces
- writes task specs
- verifies expected file state
- computes APEX score with configurable human/LLM component
