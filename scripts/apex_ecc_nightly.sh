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

safe_rebase_push(){
  local stash_name="ecc-runtime-noise-$(date +%s)"
  local stashed=0

  # Recompute hygiene before touching git history. This is evidence, not a scoring shortcut.
  if [ -x "$ROOT/scripts/apex-hygiene/apex-hygiene" ]; then
    "$ROOT/scripts/apex-hygiene/apex-hygiene" --root "$ROOT" --out "$STATE/apex-hygiene-latest.json" >/dev/null 2>&1 || true
  fi

  if [ -n "$(git status --porcelain)" ]; then
    log "Runtime/managed dirty files exist before rebase; stashing them temporarily."
    git stash push -u -m "$stash_name" 2>&1 | tee -a "$LOG" || return 1
    stashed=1
  fi

  local ok=0
  if git pull --rebase 2>&1 | tee -a "$LOG"; then
    if git push 2>&1 | tee -a "$LOG"; then
      ok=1
    else
      log "push failed; leaving commit local"
    fi
  else
    log "rebase failed; leaving commit local for manual review"
  fi

  if [ "$stashed" -eq 1 ]; then
    git stash pop 2>&1 | tee -a "$LOG" || log "stash pop had conflicts; manual review needed"
  fi

  [ "$ok" -eq 1 ]
}

cd "$ROOT"
log "===== APEX ECC nightly cycle start ====="

# Build only local helpers touched by ECC. Do not install global packages.
for d in scripts/apex-ecc-runtimeos scripts/apex-fusion-engine scripts/apex-praison-chain scripts/apex-dawn-gate scripts/apex-hygiene scripts/apex-evidence-validator scripts/apex-12factor-agent scripts/apex-phasor-llm scripts/apex-agent-dispatch scripts/apex-cmmi-delivery scripts/apex-memory-admission; do
  if [ -f "$ROOT/$d/go.mod" ]; then
    (cd "$ROOT/$d" && go build -o "$(basename "$d")" .) 2>&1 | tee -a "$LOG"
  fi
done

# Gates: ECC -> fusion -> evidence -> hygiene -> PHI full mirror tracker.
run "$ROOT/scripts/apex-ecc-runtimeos/apex-ecc-runtimeos" --mode cycle --root "$ROOT" --out "$STATE/apex-ecc-runtimeos-latest.json"
run "$ROOT/scripts/apex-fusion-engine/apex-fusion-engine" --mode selftest --root "$ROOT" --out "$STATE/apex-fusion-engine-latest.json"
run "$ROOT/scripts/apex-evidence-validator/apex-evidence-validator" --mode validate --input "$STATE/apex-fusion-evidence.json" --out "$STATE/apex-fusion-evidence-report.json"
run "$ROOT/scripts/apex-hygiene/apex-hygiene" --root "$ROOT" --out "$STATE/apex-hygiene-latest.json"
run "$ROOT/scripts/apex-12factor-agent/apex-12factor-agent" --mode selftest --root "$ROOT" --out "$STATE/apex-12factor-agent-latest.json"
run "$ROOT/scripts/apex-phasor-llm/apex-phasor-llm" --mode selftest --root "$ROOT" --out "$STATE/apex-phasor-llm-latest.json"
run "$ROOT/scripts/apex-agent-dispatch/apex-agent-dispatch" --mode selftest --root "$ROOT" --out "$STATE/apex-agent-dispatch-latest.json"
run "$ROOT/scripts/apex-cmmi-delivery/apex-cmmi-delivery" --mode cycle --root "$ROOT" --task "APEX CMMI industrial delivery closed loop" --out "$STATE/apex-cmmi-delivery-latest.json"
run "$ROOT/scripts/apex-memory-admission/apex-memory-admission" --mode admit --root "$ROOT" --input state/apex-fusion-evidence.json --out "$STATE/apex-memory-admission-latest.json"
run "$ROOT/scripts/phi_tracker.sh"

# Append observability record.
printf '{"timestamp":"%s","task":"apex_ecc_nightly_cycle","result":"pass","evidence":"state/apex-ecc-runtimeos-latest.json","fusion":"state/apex-fusion-engine-latest.json","evidence_report":"state/apex-fusion-evidence-report.json","twelve_factor":"state/apex-12factor-agent-latest.json","phasor_llm":"state/apex-phasor-llm-latest.json","agent_dispatch":"state/apex-agent-dispatch-latest.json","cmmi_delivery":"state/apex-cmmi-delivery-latest.json","memory_admission":"state/apex-memory-admission-latest.json"}\n' "$(date -Iseconds)" >> "$ROOT/memory/metrics/task_runs.jsonl"

# Commit only intentional ECC/runtime artifacts. Runtime ignored files may remain dirty by design.
git add \
  scripts/apex-ecc-runtimeos scripts/apex-fusion-engine scripts/apex-praison-chain scripts/apex-dawn-gate scripts/apex-hygiene scripts/apex-evidence-validator scripts/apex-12factor-agent scripts/apex-phasor-llm scripts/apex-agent-dispatch scripts/apex-cmmi-delivery scripts/apex-memory-admission \
  skills/apex-ecc-runtimeos skills/apex-praison-chain \
  state/apex-ecc-runtimeos-latest.json state/apex-fusion-engine-latest.json state/apex-fusion-evidence.json state/apex-fusion-evidence-report.json state/apex-praison-activation.json state/apex-12factor-agent-latest.json state/apex-phasor-llm-latest.json state/apex-agent-dispatch-latest.json state/apex-cmmi-delivery-latest.json state/apex-memory-admission-latest.json state/apex-memory-admission-evidence-report.json state/phi_tracker_latest.json state/phi_v10_result.json state/phi_history.jsonl state/sigma_memory.json \
  memory/ecc memory/praison memory/metrics/task_runs.jsonl \
  2>/dev/null || true

if git diff --cached --quiet; then
  log "No intentional staged changes; skip commit."
else
  git commit -m "nightly: ECC RuntimeOS gated evolution $(date '+%Y-%m-%d')" 2>&1 | tee -a "$LOG"
  safe_rebase_push || true
fi

log "===== APEX ECC nightly cycle complete ====="
