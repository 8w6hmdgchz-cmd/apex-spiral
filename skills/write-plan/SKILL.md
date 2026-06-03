---
name: write-plan
description: Decompose a chosen design into a task DAG. Each task is testable, atomic, and assigned an agent (main / subagent). Inspired by obra/superpowers (write-plan / subagent-driven-development / dispatching-parallel-agents).
trigger: design chosen, multi-step work, "plan it"
priority: high
tier: plan
depends-on: [brainstorm]
---

# write-plan

> **A plan is a DAG, not a list.** Every task has an owner and a test.

## When To Use

- The user approved a design (output of `brainstorm`).
- The work spans >1 file, >1 component, or >1 service.
- Multiple steps can run in parallel.

## Procedure

### 1. Inventory the work

Read the design card from `brainstorm`. Break each component into 1-5 **tasks**. A task is:

```yaml
- id: T1
  title: <verb + object>
  owner: main | subagent:<name>
  depends-on: []
  files: [list]
  test: <one-line, executable>
  done_when: <observable, verifiable>
  estimated_minutes: <5..60>
```

### 2. Topological sort

Build a DAG from `depends-on`. Print the layers:

```
Layer 0 (parallel):  T1, T2, T3
Layer 1 (parallel):  T4 (after T1, T2), T5 (after T2, T3)
Layer 2 (sequential): T6 (after T4, T5)
```

### 3. Mark parallelism

Tasks in the same layer with no shared files may be dispatched as parallel subagents (inspiration: `superpowers` `dispatching-parallel-agents`).

### 4. Add verification gates

After each layer, add a `verify` task:

```yaml
- id: V1
  title: verify layer 0
  owner: main
  depends-on: [T1, T2, T3]
  test: <how we know layer 0 is correct>
  done_when: <observable>
```

### 5. Add risk tasks

For any task with `reversibility: hard` (from `brainstorm`), add a rollback task.

## Output Contract

```yaml
plan:
  goal: <from brainstorm>
  layers:
    - [task_ids...]
  tasks:
    - id: T1
      ...
  verification_gates: [V1, V2, ...]
  rollback_plan: <one-line per hard-to-reverse task>
  next_skill: execute-plan
```

## Anti-Patterns

- ❌ Don't create tasks with no `test` (un-testable = un-finishable).
- ❌ Don't create tasks with no `done_when` (vague = infinite).
- ❌ Don't nest tasks (if a task needs sub-tasks, it's a layer).
- ❌ Don't forget the verification gate (it's how `verify` knows what to check).
