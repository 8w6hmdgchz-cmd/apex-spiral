# Search trajectories

Append compact reusable search lessons here.

## 2026-05-23 A2A GitHub acquisition
- Problem: GitHub HTTPS/API reset; gh failed because it called api.github.com GraphQL.
- Selected cards: failure_recover, entity_trace, context_backtrack.
- Worked: direct SSH clone `git@github.com:owner/repo.git`; local cache sync to pending; absorber then hunt.
- Failed: gh repo clone, api.github.com, malformed filenames with spaces.
- Reusable improvement: for GitHub resource acquisition in blocked networks, prefer SSH clone and canonical `pending.list`/`absorbed.list` names; normalize legacy typo artifacts before absorb.
