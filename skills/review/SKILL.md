---
name: review
description: Self code review before / instead of asking a human. Karpathy-style: simplicity, surgical, goal-aligned. Inspired by obra/superpowers (requesting-code-review, receiving-code-review), karpathy (minimal, teach through code).
trigger: diff exists, "review my code", after execute-plan
priority: high
tier: guard
depends-on: [verify]
---

# review

> **The best reviewer is the one who wrote it — if they wait 5 minutes and forget.**

## When To Use

- After a `verify` passes.
- Before opening a PR.
- Whenever the model is about to declare a feature "ready".

## Procedure

### 1. The 5-minute rule

After the last commit, **do not review immediately.** Read the diff as if a stranger wrote it. (`git diff HEAD~1` or `git diff main`.)

### 2. Karpathy Checklist

For each hunk in the diff, ask:

```markdown
- [ ] **Simplicity first** — Is this the simplest possible solution? Could I delete code and the test still passes?
- [ ] **Surgical** — Does this change touch only what it should? (No drive-by reformatting.)
- [ ] **Goal-aligned** — Does this advance the `decision.goal` from brainstorm?
- [ ] **No speculative generality** — Did I add an "in case we need it later" feature? (YAGNI.)
- [ ] **Tested** — Is the new code covered? Are edge cases tested?
- [ ] **Documented** — Is the *why* clear in a comment or commit message? (Not the *what* — git diff shows the what.)
- [ ] **Reversible** — Can I revert this commit cleanly? (Single concern, no entanglements.)
- [ ] **Names** — Would a new team member understand `<variable>` without context?
```

If any answer is "no", fix it in a follow-up commit. Do not amend.

### 3. superpowers Requesting-Code-Review heuristic

Ask yourself:
- "If I sent this PR to a senior engineer, what would they flag?"
- List the top 3 risks. If any is not addressed, fix or document.

### 4. The "explain it" test

Can you explain the diff in 3 sentences to a non-engineer on the project? If not, simplify or add a comment.

## Anti-Patterns

- ❌ Don't review your own code in the same commit (you'll rationalize it).
- ❌ Don't add "TODO: review" comments (review is the skill, not a comment).
- ❌ Don't accept a long diff (long diffs hide bugs; split or revert).
- ❌ Don't rubber-stamp ("looks good" without the checklist).

## Output Contract

```yaml
review:
  diff_size: <lines changed, files changed>
  karpathy_checklist: {simplicity: ok|fix, surgical: ok|fix, ...}
  top_3_risks: [<risk>, <risk>, <risk>]
  fixes_applied: [<commit sha>, ...]
  verdict: approved | needs-fixes
  next_skill: memory (to record the pattern)
```
