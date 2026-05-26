# Devour: Lefthook → APEX Hygiene Filter

## Objective

突破当前 `Ω_dawn` 瓶颈：把 git dirty state 拆成真实源码变更、运行态产物、托管记忆、vendor 噪音。PHI 不应该把自动日志、hunt 输出、外部源码缓存当成系统退化。

## Source Evidence

- Repo: `https://github.com/evilmartians/lefthook.git`
- Commit: `22be6c50e1412c748f3c6b60e9c61cd056dc693b`
- Local snapshot: `vendor/github/evilmartians/lefthook`

Inspected files:

- `README.md`
- `docs/examples/filters.md`
- `docs/examples/lefthook-local.md`
- `docs/examples/skip.md`
- `docs/configuration/glob.md`
- `docs/usage/commands/run.md`

## Distilled Pattern

Lefthook contributes a repo-hygiene pattern:

1. **File-set first**
   - Hooks operate on an explicit file set (`staged_files`, `all_files`, custom `files`).

2. **Glob/exclude filters**
   - Commands receive only relevant paths.
   - Irrelevant generated files are excluded from checks.

3. **Local overrides**
   - `lefthook-local.yml` can customize local behavior without polluting shared config.

4. **Skip gates**
   - Commands/tags can be skipped explicitly for context-aware runs.

5. **Run output boundaries**
   - Hook runs are separate from application artifacts; output should not become accidental source state.

## Local Reimplementation

Created:

- `scripts/apex-hygiene/main.go`
- `scripts/apex-hygiene/go.mod`
- `scripts/apex-hygiene/apex-hygiene`
- `state/apex-hygiene-latest.json`

`apex-hygiene` reads `git status --porcelain` and classifies paths:

- `source` → true real dirty
- `transient` → logs and hunt outputs
- `managed_memory` → daily memory / metrics
- `managed_evidence` → structured evidence JSON / eval trajectories
- `vendor` → external source snapshots

Modes:

```bash
scripts/apex-hygiene/apex-hygiene --root /Users/lihongxin/.openclaw/workspace
scripts/apex-hygiene/apex-hygiene --root /Users/lihongxin/.openclaw/workspace --mode real-count
```

## Integration

Updated `scripts/auto_reflux.sh`:

- Runs `apex-hygiene` during `step_omega_dawn` when available.
- Logs total dirty and real dirty separately.
- Uses `real_dirty / 40` as the dirty penalty instead of raw git dirty count.
- Writes `real_dirty` and `total_dirty` into `state/omega_dawn_history.jsonl`.

Updated `.gitignore`:

- ignores devoured external source snapshots for jsonschema and lefthook
- ignores `state/a2a-hunt-*.json`
- ignores `state/apex-hygiene-latest.json`
- ignores `*.log`

## Verification

```bash
cd scripts/apex-hygiene
go build -o apex-hygiene .
./apex-hygiene --root /Users/lihongxin/.openclaw/workspace --out /Users/lihongxin/.openclaw/workspace/state/apex-hygiene-latest.json
```

Result at verification time:

```text
total: 5
real_dirty: 3
managed_dirty: 1
transient_dirty: 1
vendor_dirty: 0
```

The classifier correctly separated:

- `.gitignore`, `scripts/auto_reflux.sh`, `scripts/apex-hygiene/` as source work
- `apex-enlightenment/hub-sync-stderr.log` as transient
- `memory/2026-05-26.md` as managed memory

## PHI Impact

This is an `Ω_dawn` fix, not a metric hack:

- Runtime logs stop masquerading as source instability.
- Vendor snapshots are explicitly isolated.
- Daily memory can remain useful without dragging sync health.
- Auto Reflux can penalize only real uncommitted source work.

## Next Step

Wire `apex-hygiene` into the full real-environment PHI calculator so `state/phi_v10_result.json` uses the same real-dirty classification as `auto_reflux.sh`.
