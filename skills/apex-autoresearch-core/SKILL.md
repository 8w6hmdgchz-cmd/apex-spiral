---
name: apex-autoresearch-core
description: APEX AutoResearch core. Use when the user needs evidence-ledger research planning, hypothesis generation, critique, and verification plans without fabricated scientific claims.
metadata: { "openclaw": { "emoji": "🔬", "requires": { "bins": ["go", "git"] } } }
---

# APEX AutoResearch Core Skill

## Source Devoured

Verified by GitHub SSH:

| Repo | Commit | Status |
|---|---|---|
| `Future-House/paper-qa` | `d2c3c698fdf06986aa021812ab3186d3696438d8` | installed README/docs snapshot |
| `assafelovic/gpt-researcher` | `92bfc0388c5f7a03b6cb34eaf6ae14298a4b458e` | SSH HEAD verified, bulk install stalled |
| `stanford-oval/storm` | `fb951af7744dab086e34962e9bc6fe878e145f83` | SSH HEAD verified, bulk install stalled |

Stars are omitted because no verified star counts were retrieved.

## Local Core

```bash
/Users/lihongxin/.openclaw/workspace/scripts/apex-autoresearch-core/apex-autoresearch-core
```

Purpose:

```text
question -> evidence ledger -> hypotheses -> critique -> verification plan
```

## CLI

```bash
cd scripts/apex-autoresearch-core
./apex-autoresearch-core --question "PHN IFI6 biomarker validation"
./apex-autoresearch-core --mode ledger --question "..." --evidence evidence.json
./apex-autoresearch-core --mode hypotheses --question "..." --evidence evidence.json
./apex-autoresearch-core --mode critique --question "..." --evidence evidence.json
```

## No-Fake-Science Rules

- No claim without source evidence.
- Separate verified finding, hypothesis, and unknown.
- Unknown effect size/p-value remains unknown.
- Local data results require executable script and artifact path.
- Biomarker claims require effect size, uncertainty, multiple testing control, and external validation.
