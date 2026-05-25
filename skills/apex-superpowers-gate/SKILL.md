---
name: apex-superpowers-gate
description: APEX Superpowers Engineering Gate. Use to enforce requirements, architecture, tests, implementation artifacts, verification evidence, and delivery readiness before promoting any devoured capability.
metadata: { "openclaw": { "emoji": "🛡️", "requires": { "bins": ["go", "git"] } } }
---

# APEX Superpowers Gate Skill

## Source Devoured

Verified by GitHub SSH:

| Repo | Commit | Status |
|---|---|---|
| `guardrails-ai/guardrails` | `28d74af02215f3d09e6527238f783c561218d539` | installed sanitized source/docs snapshot |
| `BerriAI/litellm` | `f45909cb81e698ea9172d3622ed90da42af3345d` | SSH HEAD verified |
| `great-expectations/great_expectations` | `19813b79613cf19e3d26cd5d7beaa36626ef9448` | SSH HEAD verified |

No stars are recorded because no verified star count was retrieved.

## Local Core

```bash
/Users/lihongxin/.openclaw/workspace/scripts/apex-superpowers-gate/apex-superpowers-gate
```

Gate stages:

```text
requirements -> architecture -> tests -> implementation -> verification -> promote/block
```

## CLI

```bash
cd scripts/apex-superpowers-gate
./apex-superpowers-gate --mode schema
./apex-superpowers-gate --input gate.json
./apex-superpowers-gate --mode tests --input gate.json
```

## Hard Rules

Promotion requires:

- task defined
- requirements with acceptance criteria
- architecture sketch
- concrete test command
- implementation artifact path
- verification evidence
- evidence path/output

Weak evidence such as TODO, not-run, or assumed outputs becomes warning or blocker.
