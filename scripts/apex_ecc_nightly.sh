#!/usr/bin/env bash
# APEX ECC Nightly Controlled Evolution
# Incremental, reversible, evidence-gated. No destructive actions.
set -Eeuo pipefail
IFS=$'\n\t'

ROOT="${ROOT:-/Users/lihongxin/.openclaw/workspace}"
STATE="$ROOT/state"
LOG="$STATE/apex-ecc-nightly.log"
mkdir -p "$STATE"

log(){ echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG"; }
run(){ log "▶ $*"; "$@" 2>&1 | tee -a "$LOG"; }

cd "$ROOT"
log "===== APEX ECC nightly cycle start ====="

# Build only local helpers touched by ECC. Do not install global packages.
for d in scripts/apex-ecc-runtimeos scripts/apex-fusion-engine scripts/apex-praison-chain scripts/apex-dawn-gate scripts/apex-hygiene scripts/apex-evidence-validator; do
  if [ -f "$ROOT/$d/go.mod" ]; then
    (cd "$ROOT/$d" && go build -o "$(basename "$d")" .) 2>&1 | tee -a "$LOG"
  fi
done

# Gates: ECC -> fusion -> evidence -> hygiene -> PHI full mirror tracker.
run "$ROOT/scripts/apex-ecc-runtimeos/apex-ecc-runtimeos" --mode cycle --root "$ROOT" --out "$STATE/apex-ecc-runtimeos-latest.json"
run "$ROOT/scripts/apex-fusion-engine/apex-fusion-engine" --mode selftest --root "$ROOT" --out "$STATE/apex-fusion-engine-latest.json"
run "$ROOT/scripts/apex-evidence-validator/apex-evidence-validator" --mode validate --input "$STATE/apex-fusion-evidence.json" --out "$STATE/apex-fusion-evidence-report.json"
run "$ROOT/scripts/apex-hygiene/apex-hygiene" --root "$ROOT" --out "$STATE/apex-hygiene-latest.json"
run "$ROOT/scripts/phi_tracker.sh"

# Append observability record.
printf '{"timestamp":"%s","task":"apex_ecc_nightly_cycle","result":"pass","evidence":"state/apex-ecc-runtimeos-latest.json","fusion":"state/apex-fusion-engine-latest.json","evidence_report":"state/apex-fusion-evidence-report.json"}\n' "$(date -Iseconds)" >> "$ROOT/memory/metrics/task_runs.jsonl"

# Commit only intentional ECC/runtime artifacts. Runtime ignored files may remain dirty by design.
git add \
  scripts/apex-ecc-runtimeos scripts/apex-fusion-engine scripts/apex-praison-chain scripts/apex-dawn-gate scripts/apex-hygiene scripts/apex-evidence-validator \
  skills/apex-ecc-runtimeos skills/apex-praison-chain \
  state/apex-ecc-runtimeos-latest.json state/apex-fusion-engine-latest.json state/apex-fusion-evidence.json state/apex-fusion-evidence-report.json state/apex-praison-activation.json state/phi_tracker_latest.json state/phi_v10_result.json state/phi_history.jsonl state/sigma_memory.json \
  memory/ecc memory/praison memory/metrics/task_runs.jsonl \
  2>/dev/null || true

if git diff --cached --quiet; then
  log "No intentional staged changes; skip commit."
else
  git commit -m "nightly: ECC RuntimeOS gated evolution $(date '+%Y-%m-%d')" 2>&1 | tee -a "$LOG"
  if git pull --rebase 2>&1 | tee -a "$LOG"; then
    git push 2>&1 | tee -a "$LOG" || log "push failed; leaving commit local"
  else
    log "rebase failed; leaving commit local for manual review"
  fi
fi

log "===== APEX ECC nightly cycle complete ====="
