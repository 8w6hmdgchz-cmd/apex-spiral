# Fourth Devour: AutoResearch Core

## Objective

Add an evidence-ledger research core for scientific tasks and repo-devour selection.

## Candidate Evidence

Verified with GitHub SSH:

| Repo | Commit | Status |
|---|---|---|
| `Future-House/paper-qa` | `d2c3c698fdf06986aa021812ab3186d3696438d8` | installed README/docs snapshot |
| `assafelovic/gpt-researcher` | `92bfc0388c5f7a03b6cb34eaf6ae14298a4b458e` | SSH HEAD verified, bulk install stalled |
| `stanford-oval/storm` | `fb951af7744dab086e34962e9bc6fe878e145f83` | SSH HEAD verified, bulk install stalled |

No virtual stars or unverified benchmark values.

## Local Implementation

- `scripts/apex-autoresearch-core/main.go`
- `scripts/apex-autoresearch-core/apex-autoresearch-core`
- `skills/apex-autoresearch-core/SKILL.md`

## Verification

```bash
cd scripts/apex-autoresearch-core
go build -o apex-autoresearch-core .
./apex-autoresearch-core --question "PHN IFI6 biomarker validation"
```

The core produced a provisional plan and refused to promote a finding without evidence ledger sources.
