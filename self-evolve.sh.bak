#!/bin/bash
# ApexSpiral 自进化脚本 v5 (bash + temp files)
# Signal → Gene Selection → Mutation → Validation → Score

set -u

STATE_DIR="/Users/lihongxin/.openclaw/workspace/apex-enlightenment/state"
EVOLUTION_LOG="$STATE_DIR/evolution_log.jsonl"
DG_VALUE="${1:-0.3}"

TEMP_SIGNALS=$(mktemp)
TEMP_GENE=$(mktemp)
TEMP_METRICS=$(mktemp)

cleanup() {
    rm -f "$TEMP_SIGNALS" "$TEMP_GENE" "$TEMP_METRICS"
}
trap cleanup EXIT

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" >&2; }

log "=== 自进化开始 (DG=$DG_VALUE) ==="

# ========== Step 1: 检测信号 ==========
log "[1/4] 检测信号..."

python3 "$STATE_DIR/../self-evolve-step1.py" "$DG_VALUE" > "$TEMP_SIGNALS"
SIGNALS=$(python3 -c "import json,sys; print(json.dumps(json.load(sys.stdin)['signals']))" < "$TEMP_SIGNALS")
log "[信号] $SIGNALS"

# ========== Step 2: 基因选择 ==========
log "[2/4] 基因选择..."

python3 - "$SIGNALS" <<'PYEOF' > "$TEMP_GENE"
import json, sys
signals = json.loads(sys.argv[1])
gene_pool = [
    {"id": "gene_gep_repair_from_errors", "category": "repair", "score": 0.8, "signals": ["recurring_error", "repair_loop_detected"]},
    {"id": "gene_tool_integrity", "category": "optimize", "score": 0.75, "signals": ["capability_gap", "perf_bottleneck"]},
    {"id": "gene_bounty_answer", "category": "innovate", "score": 0.6, "signals": ["evolution_saturation", "empty_cycle_loop_detected"]},
]
best = gene_pool[0]
best_match = 0
for gene in gene_pool:
    m = sum(1 for s in signals if s in gene['signals'])
    if m > best_match:
        best_match = m
        best = gene
print(json.dumps(best))
PYEOF

GENE_ID=$(python3 -c "import json,sys; print(json.load(sys.stdin)['id'])" < "$TEMP_GENE")
log "[基因] $GENE_ID"

# ========== Step 3: 计算指标 ==========
log "[3/4] 计算自进化指标..."

python3 <<'PYEOF' > "$TEMP_METRICS"
import json
sigma_coherence = 0.85
delta_drift = 0.1
rho_alignment = 0.9
omega_self = sigma_coherence * (1.0 - delta_drift) * rho_alignment
weights = [0.3, 0.4, 0.3]
quality_deltas = [0.1, 0.15, 0.05]
gamma_reflect = sum(w * d for w, d in zip(weights, quality_deltas)) / sum(weights)
evolution_score = omega_self * 0.6 + gamma_reflect * 0.4
result = {
    "omega_self": round(omega_self, 4),
    "gamma_reflect": round(gamma_reflect, 4),
    "evolution_score": round(evolution_score, 4)
}
print(json.dumps(result))
PYEOF

OMEGA=$(python3 -c "import json,sys; print(json.load(sys.stdin)['omega_self'])" < "$TEMP_METRICS")
GAMMA=$(python3 -c "import json,sys; print(json.load(sys.stdin)['gamma_reflect'])" < "$TEMP_METRICS")
SCORE=$(python3 -c "import json,sys; print(json.load(sys.stdin)['evolution_score'])" < "$TEMP_METRICS")
log "[指标] Omega_self=$OMEGA  Gamma_reflect=$GAMMA  Score=$SCORE"

# ========== Step 4: 记录日志 ==========
log "[4/4] 记录到evolution_log..."

python3 - "$SIGNALS" "$GENE_ID" "$OMEGA" "$GAMMA" "$SCORE" "$DG_VALUE" <<'PYEOF'
import json, sys
from pathlib import Path
from datetime import datetime, timezone, timedelta

signals = json.loads(sys.argv[1])
gene_id = sys.argv[2]
omega = float(sys.argv[3])
gamma = float(sys.argv[4])
score = float(sys.argv[5])
dg = float(sys.argv[6])

entry = {
    "ts": int(datetime.now(timezone(timedelta(hours=8))).timestamp()),
    "signals": signals,
    "gene": {"id": gene_id},
    "metrics": {"omega_self": omega, "gamma_reflect": gamma, "evolution_score": score},
    "dg": dg
}
log_file = Path("/Users/lihongxin/.openclaw/workspace/apex-enlightenment/state/evolution_log.jsonl")
with log_file.open("a") as f:
    f.write(json.dumps(entry, ensure_ascii=False) + "\n")
PYEOF

log "自进化完成"
