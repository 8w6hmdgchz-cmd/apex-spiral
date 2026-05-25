---
name: apex-evolver-core
description: APEX Evolver Core. Use when the system needs observe→diagnose→patch-plan→verify self-evolution over local artifacts, driven by installed open-source evolver references and guarded by no-fake-data rules.
metadata: { "openclaw": { "emoji": "🧬", "requires": { "bins": ["go", "git"] } } }
---

# APEX Evolver Core Skill

## Source Devoured

Verified by GitHub SSH:

| Repo | Commit | Status |
|---|---|---|
| `yoheinakajima/babyagi` | `fa8930ebe72a82e5ad57b356e7cbec96290e5bb2` | installed snapshot |
| `significant-gravitas/AutoGPT` | `127a0fa96a31076dbb297d585de2faacc0c7a890` | SSH HEAD verified, install blocked by fetch/checkout stall |
| `microsoft/autogen` | `027ecf0a379bcc1d09956d46d12d44a3ad9cee14` | SSH HEAD verified, install blocked by batch fetch stall |

No stars are recorded because no verified star count was retrieved.

## Local Core

```bash
/Users/lihongxin/.openclaw/workspace/scripts/apex-evolver-core/apex-evolver-core
```

Pipeline:

```text
observe -> diagnose -> patch-plan -> verify -> archive
```

## CLI

```bash
cd scripts/apex-evolver-core
./apex-evolver-core --mode observe
./apex-evolver-core --mode diagnose
./apex-evolver-core --mode cycle
./apex-evolver-core --mode verify
```

## Evidence Rules

- Findings come from scanning local files.
- Patch plans are proposals until applied by a separate guarded edit.
- Verify reports local checks only.
- Blocked installs are recorded as blocked, not claimed as absorbed.
