# Devour: OpenHands Execution/Sandbox Pattern

## Objective

吞噬 OpenHands 的 agentic code execution / sandbox lifecycle / event stream pattern，用于补强 APEX Evol_code 的 Working 执行记忆与自修复闭环。

## Source Evidence

- Repo: `git@github.com:All-Hands-AI/OpenHands.git`
- Commit: `5e311f7f995008ffe4c74f8cf6f3085d4030c670`
- Local snapshot: `vendor/github/All-Hands-AI/OpenHands`

Inspected files:

- `openhands/app_server/sandbox/README.md`
- `openhands/app_server/event/README.md`
- `openhands/app_server/config.py`
- `openhands/app_server/sandbox/sandbox_service.py`
- `openhands/app_server/sandbox/process_sandbox_service.py`
- `openhands/app_server/event/event_service.py`

## Distilled Pattern

OpenHands separates code execution into four enforceable contracts:

1. **SandboxService** — lifecycle boundary for execution spaces.
   - create/start/resume/search sandboxes
   - wait for RUNNING status
   - health-check agent server before using it

2. **Process/Docker/Remote backends** — runtime strategy is injected, not hardcoded.
   - `RUNTIME=remote` → RemoteSandboxService
   - `RUNTIME=local|process` → ProcessSandboxService
   - default → DockerSandboxService

3. **EventService** — all actions/results become persisted events.
   - `save_event(conversation_id, event)`
   - `search_events(...)`
   - `count_events(...)`

4. **Deadlock avoidance** — process logs go to files, not pipes.
   - `subprocess.Popen(... stdout=log_handle, stderr=log_handle)`
   - avoids pipe-buffer deadlocks during long-running agent-server execution

## APEX Adaptation

For APEX Evol_code, the useful local reimplementation is not a full OpenHands copy. The distilled minimum is:

```text
WorkingExecutionRecord = {
  id,
  task,
  sandbox_strategy,
  command_or_patch,
  started_at,
  result,
  verification,
  log_path,
  rollback_hint
}
```

Execution loop:

```text
plan → start bounded sandbox/process → execute → persist event → verify → archive into Σ_memory as Working/Episodic
```

Safety constraints:

- workspace-bounded paths only
- no destructive shell by default
- command whitelist for automated execution
- long-running output redirected to log file
- every execution must have verification and event record

## Σ_memory Injection

This devour contributes **Working** memories because it is about execution mechanics, not factual notes:

- sandbox lifecycle contract
- runtime injection strategy
- event persistence contract
- pipe-deadlock avoidance
- verification-before-archive loop

## Verification

- Repo cloned via SSH and commit recorded.
- Source files inspected locally.
- No secrets copied.
- No blind copy of OpenHands code; only distilled pattern archived.
