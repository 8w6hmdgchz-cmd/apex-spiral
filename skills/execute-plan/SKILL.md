---
name: execute-plan
description: Implements the plan produced by write-plan. TDD-first (red → green → refactor), surgical changes, goal-driven execution. Inspired by obra/superpowers (test-driven-development, executing-plans), karpathy (surgical, minimal), PilotDeck (per-worktree isolation).
trigger: plan exists, "implement", "build the plan", "execute"
priority: high
tier: do
depends-on: [write-plan]
---

# execute-plan

> **Surgical. Tested. Reversible.**

## When To Use

- A `write-plan` output exists in context.
- The user said "implement" or "build" or "go".

## Procedure (per task)

For each task in the plan, in topological order:

### 0. Pre-flight (karpathy "think first")

- Re-read the task's `test` and `done_when`.
- If the test is not runnable as written, **stop and edit the plan** (use `write-plan` to amend).
- If the task touches shared state, acquire the relevant worktree (inspiration: PilotDeck WorkSpace isolation).

### 1. Red — write a failing test first

- The test must compile and run.
- The test must fail for the *reason* the task exists (not for typos).
- Commit: `test: add failing test for <task>`.

### 2. Green — write the minimum code to pass

- Smallest possible diff.
- No reformatting of unrelated code.
- No speculative generality (YAGNI).
- Commit: `feat: implement <task>`.

### 3. Refactor — clean up while green

- Improve names, dedupe, extract.
- Tests must stay green after every refactor commit.
- Commit: `refactor: <what>`.

### 4. Verify the task's `done_when`

Run the explicit verification. If the test passes but `done_when` is not met, **the task is not done** — go back to step 1.

### 5. Move to next task

Mark the task done in the plan tracker (telemetry). Move to the next layer.

## After each layer

- Run the layer's `verify` task.
- If verify fails, invoke `using-apex-skill` → `debug` (do not improvise).

## Anti-Patterns

- ❌ Don't write code before a failing test (no TDD = bugs).
- ❌ Don't bundle multiple tasks into one commit (can't bisect).
- ❌ Don't "improve" unrelated code (surgical means surgical).
- ❌ Don't skip the `done_when` check (test green ≠ task done).
- ❌ Don't push without running the project's lint+typecheck+test (run via CI hooks).

## Output Contract

```yaml
execution:
  completed_tasks: [T1, T2, ...]
  failed_tasks: []
  commits: [<sha>...]
  verification: {layer_0: pass, layer_1: ...}
  next_skill: verify | review
```
