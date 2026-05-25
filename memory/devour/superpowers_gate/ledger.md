# Fifth Devour: Superpowers Engineering Gate

## Objective

Add a hard promotion gate for devoured capabilities so nothing enters APEX as "done" without requirements, architecture, tests, implementation, and verification evidence.

## Candidate Evidence

Verified by GitHub SSH:

| Repo | Commit | Status |
|---|---|---|
| `guardrails-ai/guardrails` | `28d74af02215f3d09e6527238f783c561218d539` | installed sanitized source/docs snapshot |
| `BerriAI/litellm` | `f45909cb81e698ea9172d3622ed90da42af3345d` | SSH HEAD verified |
| `great-expectations/great_expectations` | `19813b79613cf19e3d26cd5d7beaa36626ef9448` | SSH HEAD verified |

No virtual stars.

## Local Implementation

- `scripts/apex-superpowers-gate/main.go`
- `scripts/apex-superpowers-gate/apex-superpowers-gate`
- `skills/apex-superpowers-gate/SKILL.md`

## Verification

Fail case: input only has task -> blocked with missing requirements/architecture/tests/implementation/verification/evidence.

Pass case: harness bridge gate input with selftest evidence -> `ok=true`, `score=1`, `next_action=promote`.
