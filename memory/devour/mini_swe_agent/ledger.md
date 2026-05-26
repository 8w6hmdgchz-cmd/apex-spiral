# Devour: mini-swe-agent → APEX Mini Executor

## Objective

突破 PHI 的方式不是继续吞大架构，而是吞可验证执行 harness。`SWE-agent/mini-swe-agent` 命中当前瓶颈：线性轨迹、独立 subprocess、环境可替换、trajectory 保存、SWE-bench/ProgramBench harness。

## Source Evidence

- Repo: `https://github.com/SWE-agent/mini-swe-agent.git`
- Commit: `adfe20233d456104c38c3129161b54f0fd39f2c7`
- Local snapshot: `vendor/github/SWE-agent/mini-swe-agent`

Inspected files:

- `README.md`
- `src/minisweagent/agents/default.py`
- `src/minisweagent/environments/local.py`
- `src/minisweagent/run/benchmarks/swebench_single.py`
- `src/minisweagent/utils/serialize.py`
- `src/minisweagent/models/utils/actions_text.py`

## Distilled Pattern

mini-swe-agent is useful because it removes scaffold noise:

1. **Linear trajectory**
   - `messages` are the trajectory.
   - Every step appends model message + observation.
   - Debugging and training data are the same object.

2. **Independent shell actions**
   - no persistent shell session
   - each action is `subprocess.run(...)`
   - easier sandbox replacement: local → docker exec → singularity/bubblewrap/etc.

3. **Hard limits**
   - step limit
   - cost limit
   - wall-time limit

4. **Trajectory serialization**
   - saved as JSON with config, model stats, exit status, submission, messages.

5. **Benchmark harness**
   - `swebench_single.py` loads a benchmark instance, builds config, constructs env/model/agent, then `agent.run(problem_statement)`.

## Local Reimplementation

Created:

- `scripts/apex-mini-executor/main.go`
- `scripts/apex-mini-executor/go.mod`
- `scripts/apex-mini-executor/apex-mini-executor`
- selftest output: `state/apex-mini-executor-selftest.traj.json`

Capabilities:

- workspace-bounded command execution
- allowlist + dangerous token block
- timeout
- dry-run validation mode
- independent `/bin/zsh -lc` subprocess per action
- trajectory JSON with input/output hashes
- selftest gate

## Verification

```bash
cd scripts/apex-mini-executor
go build -o apex-mini-executor .
./apex-mini-executor --mode selftest --workspace /Users/lihongxin/.openclaw/workspace
```

Result:

```text
selftest_ok
```

Selftest verified commands:

- `pwd`
- `printf 'ok\n'`
- `python3 -c 'print(2+2)'` → `4`

## PHI Impact

This is a better PHI breakthrough target than another agent architecture because it changes reality:

- **Evol_code**: new runnable executor exists.
- **Working memory**: execution harness and trajectory format are operational.
- **Ω_dawn**: execution is bounded/allowlisted/time-limited.
- **Σ_memory**: evidence-backed Working/Procedural memories can now cite selftest output.

## Next Reimplementation

Connect `apex-mini-executor` to:

- `memory/metrics/task_runs.jsonl`
- `bench/openclaw_agent_tasks/tasks.yaml`
- PaperQA-style evidence validator
- AutoGPT-style named-output node graph
