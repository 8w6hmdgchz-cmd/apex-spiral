---
name: apex-secrets-gate
description: APEX local secrets and supply-chain gate. Use before committing/pushing devoured third-party snapshots or generated artifacts to detect API keys, tokens, and private key material.
metadata: { "openclaw": { "emoji": "🔐", "requires": { "bins": ["go", "git"] } } }
---

# APEX Secrets Gate Skill

## Source Devoured

Verified by GitHub SSH:

| Repo | Commit | Status |
|---|---|---|
| `gitleaks/gitleaks` | `80093b8a7b600e52d96ec5d49e9657f5c74b77fa` | installed sanitized source snapshot |
| `trufflesecurity/trufflehog` | `0ec3634f6cf66a61912a923fee9d20cc45633a67` | SSH HEAD verified |
| `aquasecurity/trivy` | `f2a12375772a93ab9bd6f754c33378f1c0356a76` | SSH HEAD verified |

No stars are recorded because no verified star count was retrieved.

## Local Core

```bash
/Users/lihongxin/.openclaw/workspace/scripts/apex-secrets-gate/apex-secrets-gate
```

## CLI

```bash
cd scripts/apex-secrets-gate
go build -o apex-secrets-gate .
./apex-secrets-gate --root /path/to/snapshot
```

Findings cause nonzero exit. JSON report includes redacted evidence.

## Rules

- OpenAI API key pattern
- GitHub token pattern
- AWS access key pattern
- private key block
- Slack token pattern

Use before `git add` for third-party snapshots and before `git push` when a commit touches vendor/third_party content.
