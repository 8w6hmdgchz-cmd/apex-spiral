---
name: clawg-training-ecosystem
description: "Local agent training ecosystem for file-operation tasks: mock workspaces, APEX task generation, trajectory capture, sandbox rollout, and mixed verification."
---

# ClawG Training Ecosystem

Use when building or evaluating local agent file-operation ability with safe mock workspaces, reproducible tasks, trajectories, and benchmark scoring.

## Contract

- Never train/evaluate against private real files unless explicitly requested.
- Use mock workspaces for tasks.
- Prefer deterministic AutoVerify before LLM/human judgment.
- Do not claim model weights were trained unless an actual training run happened.
- Python is glue only. Durable task/verify core should be Rust/Go/C.

## APEX formulas

Task generation:

`Task_APEX = PersonaIntent × SkillGrounding × MockWorkspace`

Training convergence:

`Agent_APEX = SFT_Trajectory + RL_Rollout × SandboxParallel`

Evaluation:

`Score_APEX = AutoVerify(60%) + LLM_HumanVerify(40%)`

Iteration:

`Iteration_APEX = Data → Train → Bench → Feedback`

## Workflow

1. Generate safe mock workspace and task spec.
2. Run agent/tool trajectory in sandbox.
3. AutoVerify file state and command outputs.
4. Add optional LLM/human score for quality/intent.
5. Store trajectory/result under `bench/clawg`.
6. Promote repeated failures to skills or docs.
7. Iterate with new tasks and regression set.

## Task types

- file create/edit/rename/move
- config patch and validation
- script creation plus smoke test
- data transform with expected output
- repo cleanup with no destructive deletion
- docs update with evidence markers
