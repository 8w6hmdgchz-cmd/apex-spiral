---
name: workspaces
description: Per-project sandbox. Worktree, env vars, port allocation, MCP servers — all isolated. Inspired by OpenBMB/PilotDeck (WorkSpace isolation), obra/superpowers (using-git-worktrees).
trigger: "switch to project Y", "isolate this", "use a worktree", "create a sandbox"
priority: high
tier: do
depends-on: []
---

# workspaces

> **One project. One worktree. One set of ports. Zero cross-contamination.**

## When To Use

- The user works on multiple repos / projects and they should not share state.
- A task is risky (CI breaking, data migration) and reversibility matters.
- The user says "use a worktree" / "isolate this" / "sandbox".

## WorkSpace Model (PilotDeck)

A workspace is a tuple:

```yaml
workspace:
  id: ws-2026-06-02-001
  cwd: /abs/path/to/project
  worktree: /abs/path/to/worktree            # optional
  branch: feat/<task-id>                      # optional
  env:                                        # injected at session start
    APEX_WORKSPACE_ID: ws-2026-06-02-001
    APEX_WORKSPACE_CWD: /abs/path/to/project
  ports:                                      # allocated, free
    http: 5180
    mcp:  5181
  mcp_servers: [apex-mem, github]             # names only; config in .mcp.json
```

## Procedure

### 1. Detect need

If the current `cwd` is not a workspace root (no `.apex-workspace` marker), and the user invokes a long task, create one.

### 2. Create the worktree (if requested)

```bash
git worktree add -b feat/<task-id> ../<task-id>-worktree
cd ../<task-id>-worktree
```

### 3. Allocate resources

- Pick ports from a free pool (`5180-5280` for HTTP, `5281-5380` for MCP).
- Write `.apex-workspace` (YAML) to the worktree root.

### 4. Inject env vars

The `session-start.sh` hook reads `.apex-workspace` and exports the env vars above.

### 5. Clean up

When the user says "done with this workspace" or "merge the worktree":

```bash
git checkout main
git merge feat/<task-id> --no-ff
git worktree remove ../<task-id>-worktree
rm .apex-workspace
```

## Anti-Patterns

- ❌ Don't create a workspace for a 5-minute task (overhead > benefit).
- ❌ Don't skip the worktree (then you can't `git worktree remove` cleanly).
- ❌ Don't reuse ports across workspaces (collisions).
- ❌ Don't forget to clean up the worktree (disk leak).

## Output Contract

```yaml
workspace:
  id: ws-<iso>-<n>
  cwd: <abs path>
  worktree: <abs path or null>
  branch: <name or null>
  env: {APEX_WORKSPACE_ID, APEX_WORKSPACE_CWD, ...}
  ports: {http, mcp}
  next_skill: <caller's choice>
```
