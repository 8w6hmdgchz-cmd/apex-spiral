---
name: debug
description: 4-phase systematic debugging. Reproduce → Isolate → Hypothesize → Fix. Inspired by obra/superpowers (systematic-debugging), karpathy (think before acting).
trigger: failing test, red CI, "bug", "broken", "why does this fail"
priority: critical
tier: do
depends-on: []
---

# debug

> **Never guess. Always prove.**

## When To Use

- A test fails.
- A command returns an unexpected exit code.
- The model has tried 3+ fixes that didn't work (the post-tool-use hook will tell you).

## The 4 Phases

### Phase 1 — Reproduce

- Get the failure on demand, in a clean state.
- The command / test must run from scratch and fail the same way.
- If you can't reproduce, the bug is in your mental model — not in the code. Stop. Re-read the spec.

**Output:** one shell command that reliably reproduces the failure.

### Phase 2 — Isolate

- Bisect. Comment out. Stub. Mock. Print. Disable features.
- Goal: smallest possible code that still fails.
- The isolation must be committed to a branch (so the bisect is reproducible).

**Output:** a `git bisect` log or a `minimal-repro.patch`.

### Phase 3 — Hypothesize

- List 3-5 hypotheses ranked by likelihood.
- For each, design a 1-minute experiment that *disproves* it.
- Run the experiment, record the result.
- Stop hypothesizing when one hypothesis survives.

**Output:** `H3 confirmed: <evidence>`.

### Phase 4 — Fix

- The fix is the **smallest** change that makes the failing test pass.
- Add a regression test (so the bug can never return).
- Run the full test suite — not just the failing test.

**Output:** a commit `fix: <one-line>` with the regression test included.

## Anti-Patterns

- ❌ Don't add print statements and hope (that's not debugging, that's praying).
- ❌ Don't "fix" without a regression test (the bug will return).
- ❌ Don't refactor during a fix (separate commit, separate PR).
- ❌ Don't skip Phase 3 (the wrong fix is worse than no fix).

## Output Contract

```yaml
debug:
  reproduce: <one-line command>
  isolate: <smallest failing case>
  hypothesis: H<n>: <description>
  evidence: <how H<n> was confirmed>
  fix_commit: <sha>
  regression_test: <test path + name>
  next_skill: verify
```
