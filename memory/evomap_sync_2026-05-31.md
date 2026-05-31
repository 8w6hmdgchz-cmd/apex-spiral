# EvoMap Sync Report — 2026-05-31

## Sync Execution

**Time:** 2026-05-31 13:32 GMT+8
**Script:** `apex-github-evolution/scripts/evomap_audit.py`
**Root:** `/Users/lihongxin/.openclaw/workspace/`

## Audit Result

| Field | Value |
|--------|-------|
| `trace_id` | `evomap-1780205606146` |
| `timestamp_ms` | 1780205606146 |
| `file_count` | **680** |
| `secret_hit_count` | **0** |
| `external_sync_allowed` | `false` |
| `git_status` | (clean, no uncommitted changes) |

## Safe Dirs Covered

- `apex_token_rs`
- `clawg-mvp`
- `apex-unified-engine`
- `skills/hetu-luoshu`
- `skills/apex-token-optimizer`

## Secret Scan

**Result: PASS** — 0 secret hits across 680 files.

Scanned patterns:
- `fe_oa_[A-Za-z0-9]{16,}`
- `sk-[A-Za-z0-9_\-]{16,}`
- `ghp_[A-Za-z0-9]{20,}`
- `github_pat_[A-Za-z0-9_]{20,}`

## Latest Artifacts

- **EvoMap manifest:** `apex-github-evolution/evomap/latest.json`
- **Report archive:** `apex-github-evolution/reports/evomap-1780205606146.json`
- **Safe ledger:** `research/apex/ledgers/apex-safe-evomap-summary.json` (integrated trace_id: `evomap-1779521434960`, 288 files)

## Notes

- `external_sync_allowed` is `false` — per policy, no automatic gist/commit/push. User confirmation required before any external export.
- Git working tree is clean — no pending changes in safe dirs.
- File count increased from 288 (last ledger) to 680 (current), primarily from growing `apex-unified-engine/reports/` archive.
- `phi_ratio.log` present in reports dir — phi tracking is active.
