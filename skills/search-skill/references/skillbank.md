# SearchSkill SkillBank v1

## keyword_expand
- Trigger: user query has broad/ambiguous concept or likely synonyms.
- Query template: `{core term} {alias/synonym} {domain}`; `{framework} {feature} docs`; `{paper/repo} equivalent alternatives`.
- Stop: at least one canonical term set found.
- Verify: canonical term appears in authoritative docs, repo, paper, or multiple independent sources.
- Output: `canonical_terms`, `aliases`, `excluded_terms`.

## entity_trace
- Trigger: query references a repo, person, org, product, model, paper, or standard.
- Query template: `{entity} official`; `{entity} GitHub`; `{entity} docs`; `{entity} release notes`.
- Stop: canonical URL and owner identified.
- Verify: prefer official domain/GitHub org/package registry; avoid mirrored pages as primary evidence.
- Output: `entity`, `canonical_url`, `owner`, `evidence`.

## time_bound
- Trigger: current status, recent change, version-specific behavior, deadlines, dates.
- Query template: `{topic} after:YYYY-MM-DD`; `{topic} {version} changelog`; `{topic} 2026`.
- Stop: evidence matches requested time window.
- Verify: source publish/update date or release tag is visible.
- Output: `time_window`, `matched_sources`, `staleness_risk`.

## multi_source_verify
- Trigger: high-stakes facts, conflicting claims, numbers, security, medicine, legal, public actions.
- Query template: combine official source + independent source + primary data if available.
- Stop: 2 independent sources agree, or conflict is explicitly described.
- Verify: note agreement/conflict and confidence.
- Output: `claim`, `source_a`, `source_b`, `confidence`.

## context_backtrack
- Trigger: prior work, user preferences, local project state, OpenClaw behavior/config.
- Query template: local file search, memory search, session history, local docs before web.
- Stop: relevant local evidence found or explicitly absent.
- Verify: cite file path/line when useful.
- Output: `local_evidence`, `decision`, `remaining_unknowns`.

## failure_recover
- Trigger: empty search, blocked API, clone failure, stale/duplicate result, wrong repo name.
- Query template: SSH instead HTTPS; alternate org casing; package registry; docs mirror; repo topics; web search by exact error.
- Stop: working path found or blocker isolated.
- Verify: run minimal command/test; capture error if blocked.
- Output: `failure`, `fallback`, `result`, `next_action`.
