# Sixth Devour: Secrets / Supply-chain Gate

## Trigger

GitHub push protection blocked the Superpowers Gate commit because a third-party Guardrails example notebook contained an OpenAI API key pattern. We removed the risky examples and pushed a sanitized snapshot. This devour moves that protection earlier into local workflow.

## Candidate Evidence

Verified with GitHub SSH:

| Repo | Commit | Status |
|---|---|---|
| `gitleaks/gitleaks` | `80093b8a7b600e52d96ec5d49e9657f5c74b77fa` | installed sanitized source snapshot |
| `trufflesecurity/trufflehog` | `0ec3634f6cf66a61912a923fee9d20cc45633a67` | SSH HEAD verified |
| `aquasecurity/trivy` | `f2a12375772a93ab9bd6f754c33378f1c0356a76` | SSH HEAD verified |

## Local Implementation

- `scripts/apex-secrets-gate/main.go`
- `scripts/apex-secrets-gate/apex-secrets-gate`
- `skills/apex-secrets-gate/SKILL.md`

## Verification

Fail fixture:

```text
OPENAI_API_KEY=sk-redacted-example
```

Result: blocked with redacted evidence.

Pass fixture:

```text
hello=world
```

Result: `ok=true`.
