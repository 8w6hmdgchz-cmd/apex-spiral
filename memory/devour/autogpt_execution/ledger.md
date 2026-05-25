# Devour: AutoGPT Execution Graph / Agent Loop Pattern

## Objective

吞噬 Significant-Gravitas `AutoGPT` 的 autonomous loop、workspace permission、graph/node executor、block schema/yield-output 结构，用于补强 APEX Evol_code 的执行调度与自修复闭环。

## Source Evidence

- Repo: `git@github.com:significant-gravitas/AutoGPT.git`
- Commit: `127a0fa96a31076dbb297d585de2faacc0c7a890`
- Local snapshot: `vendor/github/significant-gravitas/AutoGPT`

Inspected files:

- `classic/original_autogpt/CLAUDE.md`
- `classic/original_autogpt/README.md`
- `docs/platform/agent-blocks.md`
- `docs/platform/block-sdk-guide.md`
- `docs/integrations/block-integrations/iteration.md`
- `autogpt_platform/backend/backend/executor/manager.py`
- `autogpt_platform/backend/backend/blocks/iteration.py`
- `autogpt_platform/backend/backend/data/execution.py`

## Distilled Pattern

AutoGPT has two relevant execution layers.

### 1. Classic agent loop

```text
while cycles_remaining:
  proposal = agent.propose_action()
  show thoughts + proposed command
  approve / reject / auto-execute
  agent.execute(proposal) or agent.do_not_execute(feedback)
  persist episode in state/history
```

Important constraints:

- Continuous mode is risky; one-step approval is default.
- Agent operates inside a workspace directory.
- Permissions are layered: agent deny → workspace deny → agent allow → workspace allow → prompt user.
- State persists under `.autogpt/agents/{id}/state.json` with profile, directives, and history.

### 2. Platform graph/node executor

Execution is graph-based:

```text
GraphExecutionEntry
  → queue starting nodes
  → execute_node(node, input)
  → block.execute(...) yields named outputs
  → persist output
  → enqueue downstream nodes when inputs become complete
  → update graph/node stats and final status
```

Key mechanics:

- `ExecutionContext` carries identity, dry-run, safety flags, hierarchy, workspace/session ids.
- Each node gets its own copied context with `node_id` and `node_exec_id`, preventing concurrent mutation races.
- `execute_node` validates inputs before running block logic.
- Credentials are acquired through metadata fields and locks, then released in `finally`.
- Blocks stream outputs as `(output_name, output_data)` yields.
- Node execution has per-block wall-clock timeout unless coordination blocks opt out.
- Failures persist an `error` output and update status.
- Downstream enqueueing is atomic around input upsert so multiple upstream nodes do not duplicate execution.
- Dry-run can simulate blocks without real side effects.
- Costs/stats are reconciled after completion/failure/termination.

### 3. Block SDK shape

```text
Block
  Input schema with validated fields/credentials
  Output schema with named pins
  async run(...) -> yields named outputs
```

The iteration block adds practical guardrails:

- max 10,000 items
- max 1 MB string input before parsing
- emit stable `item` and `key` output pins

## APEX Adaptation

APEX should add a local Evol_code execution graph contract:

```text
EvolNode = {
  node_id,
  block_name,
  input_schema,
  output_schema,
  timeout_seconds,
  side_effect_policy,
  verify_command
}

EvolExecutionContext = {
  run_id,
  node_id,
  workspace,
  dry_run,
  human_gate,
  safety_flags,
  parent_run_id,
  evidence_context_ids
}

EvolExecutionEvent = {
  run_id,
  node_id,
  status,
  input_hash,
  output_name,
  output_data_ref,
  error,
  verification,
  cost_or_time,
  source_commit
}
```

Execution rules:

- Validate node input before execution.
- Copy execution context per node; never mutate shared graph context concurrently.
- Acquire external credentials/locks only in the smallest scope and release in `finally`.
- Yield named outputs; persist each output before downstream scheduling.
- Timeout every leaf node.
- Persist error output on failure.
- Support `dry_run` for simulation before real side effects.
- Use explicit human gate for destructive/external actions.

## Σ_memory Injection

This devour contributes:

- **Working** memories for graph queue, per-node context copy, named-output persistence, dry-run simulation, failure output persistence.
- **Procedural** memory for permission order and approve/execute loop.
- **Semantic** memory for AutoGPT block schema and execution context object roles.

## Verification

- Repo cloned via SSH and commit recorded.
- Source docs and executor implementation inspected locally.
- No secrets copied.
- Pattern distilled; no blind source copy.
