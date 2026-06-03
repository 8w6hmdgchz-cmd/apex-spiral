---
name: verify
description: Refuses to call a task "done" without observable evidence. Runs the project's actual test/lint/typecheck commands. Inspired by karpathy (proof > assertion), superpowers (verification before completion).
trigger: "are we done?", "does it work?", "verify", before any "completed" claim
priority: critical
tier: guard
depends-on: [execute-plan | debug]
---

# verify

> **"It works" is not evidence. The test passing is.**

## When To Use

- Before claiming a task, layer, or plan is complete.
- After any `execute-plan` or `debug` skill.
- Whenever the model is tempted to say "should work" or "looks good".

## Procedure

### 1. Inventory the verification surface

Ask the project what *it* considers proof:

- `package.json` → `npm test`, `npm run lint`, `npm run typecheck`
- `Cargo.toml` → `cargo test --all`, `cargo clippy -- -D warnings`, `cargo fmt --check`
- `pyproject.toml` → `pytest`, `ruff check`, `mypy .`
- `Makefile` → `make test`, `make lint`
- CI config in `.github/workflows/` → the exact commands there

**The CI is the source of truth. Local "tests pass" is necessary but not sufficient.**

### 2. Run each command

```bash
<command 1> && <command 2> && ...
```

Capture stdout/stderr. If any fails:

- The task is **not verified**.
- Hand off to `using-apex-skill` → `debug`.

### 3. Cross-check the `done_when` of every task

Re-read the plan. For each `done_when`, point to the specific output that proves it. If you cannot, the task is not done — go back to `execute-plan`.

### 4. (Optional) Smoke test in a clean environment

If a CI runner / Docker / `nix develop` is available, run the same suite there. CI sometimes hides local environment dependencies.

## Anti-Patterns

- ❌ Don't accept "should work" / "looks right" / "trust me" (these are not verification).
- ❌ Don't run only the test you just wrote (run the full suite).
- ❌ Don't skip the lint/typecheck (catches what tests miss).
- ❌ Don't claim "done" before all 3 above pass.

## Output Contract

```yaml
verify:
  commands_run: [<cmd>, ...]
  results: {<cmd>: pass|fail, ...}
  done_when_checked: [{task: T1, proof: "<line of output>"}, ...]
  verdict: pass | fail
  next_skill: review (if pass) | debug (if fail)
```
