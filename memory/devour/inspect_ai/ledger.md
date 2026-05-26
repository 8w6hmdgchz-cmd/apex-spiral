# Devour: Inspect AI → APEX Eval Harness

## Objective

继续突破 PHI：把上一轮 `apex-mini-executor` 的单点 selftest 升级为 Inspect AI 风格的 eval/task/sample/scorer/log harness。目标是补强 verify gate，而不是再堆架构叙事。

## Source Evidence

- Repo: `https://github.com/UKGovernmentBEIS/inspect_ai.git`
- Commit: `84980c88259ba310ffba142747df73704ff55762`
- Local snapshot: `vendor/github/UKGovernmentBEIS/inspect_ai`

Inspected files:

- `README.md`
- `tests/test_sample_limits.py`
- `tests/test_helpers/tasks.py`
- `tests/test_helpers/utils.py`
- `docs/_errors_and_retries.md`
- `docs/_sandboxenv-interface.md`

## Distilled Pattern

Inspect AI contributes a clear eval contract:

1. **Task/Sample**
   - `Task(dataset=[Sample(input, target)], solver, scorer, limits...)`
   - task definitions are reconstructible when decorated; this enables retry.

2. **Solver/Scorer split**
   - solver mutates/solves state
   - scorer independently grades state against target
   - metrics aggregate sample scores.

3. **Limits as first-class gates**
   - message limit
   - token limit
   - time limit
   - working limit
   - cost limit

4. **Retry and sample preservation**
   - interrupted/error eval writes a log
   - retry can preserve completed samples and resume with adjusted config

5. **Sandbox interface**
   - exec/read/write operations have expected error surfaces
   - output/read limits are explicit
   - timeout retry is advisory and should be disabled for non-idempotent commands

## Local Reimplementation

Created:

- `scripts/apex-eval-harness/main.go`
- `scripts/apex-eval-harness/go.mod`
- `scripts/apex-eval-harness/apex-eval-harness`
- latest eval log: `state/apex-eval-harness-latest.json`
- sample trajectories: `state/apex-eval-harness/*.traj.json`

Capabilities:

- Task schema: `id | command | expect_contains | timeout`
- sample execution through `apex-mini-executor`
- scorer: substring match → score 0/1
- metric: accuracy
- eval log with per-sample status/output/error/duration/trajectory
- selftest mode exits non-zero on failure

## Verification

```bash
cd scripts/apex-eval-harness
go build -o apex-eval-harness .
./apex-eval-harness --mode selftest --workspace /Users/lihongxin/.openclaw/workspace --out /Users/lihongxin/.openclaw/workspace/state/apex-eval-harness-latest.json
```

Result:

```text
status: success
accuracy: 1
samples: 2/2 passed
```

## PHI Impact

This devour is PHI-relevant because it adds a real verification layer:

- **Evol_code**: APEX now has a runnable eval harness, not just executor.
- **Working memory**: task/sample/scorer/log semantics are operational.
- **Ω_dawn**: failures can be represented as sample status rather than uncontrolled dirty state.
- **Σ_memory**: future memories can cite task run logs with accuracy.

## Next Step

Wire `apex-eval-harness` into:

- `bench/openclaw_agent_tasks/tasks.yaml`
- `memory/metrics/task_runs.jsonl`
- `phi_tracker.sh` full-environment path
- PaperQA-style evidence validator
