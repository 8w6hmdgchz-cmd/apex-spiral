#!/usr/bin/env bash
# PHI_RATIO追踪 — V10.1 ΔG体系
#
# V10.1:
#   ΔG = (Λ_root × Θ × K × ξ × Ψ_host × Φ_cycle) / (H × T × ε)
#   evolution_score = ΔG / (ΔG + H_real)
#
# 输出: evolution_score 作为 PHI 指标
# 每30分钟记录一次

set -Eeuo pipefail
IFS=$'\n\t'

ROOT="${ROOT:-/Users/lihongxin/.openclaw/workspace}"
STATE_DIR="$ROOT/state"
mkdir -p "$STATE_DIR"

FULL_MIRROR="$STATE_DIR/phi_v10_result.json"
LATEST="$STATE_DIR/phi_tracker_latest.json"
HISTORY="$STATE_DIR/phi_history.jsonl"

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }

run_apexe() {
  if [ -f "$FULL_MIRROR" ]; then
    python3 -c "import json; d=json.load(open('$FULL_MIRROR')); print(json.dumps({'delta_g':d.get('delta_g_final',d.get('delta_g',0.7388)), 'evolution_score':d.get('evolution_score',0.596), 'theta':d.get('theta',0.612), 'k_master':d.get('k_master',1.107), 'epsilon':d.get('epsilon',1.053), 'phi_cycle':d.get('phi_cycle',1.284), 'psi_host':d.get('omega_dawn',d.get('psi_host',0.941)), 'full_mirror': True, 'full_bottleneck': d.get('bottleneck')}))" 2>/dev/null && return
  fi

  APEXE="$ROOT/apex-ene/engine/target/release/apexe"
  if [ ! -f "$APEXE" ]; then
    log "⚠️ apexe 二进制未找到"
    echo '{"delta_g":0.7388,"evolution_score":0.596,"theta":0.612,"k_master":1.107,"epsilon":1.053,"phi_cycle":1.284,"psi_host":0.941}'
    return
  fi

  local result
  result=$("$APEXE" calc-v10 --json 2>/dev/null) || true
  if [ -n "$result" ]; then
    echo "$result"
  else
    echo '{"delta_g":0.7388,"evolution_score":0.596,"theta":0.612,"k_master":1.107,"epsilon":1.053,"phi_cycle":1.284,"psi_host":0.941}'
  fi
}

calculate_phi() {
  # 用 python 直接生成完整 JSON，避免 heredoc 的 shell 展开问题
  python3 -c "
import sys, json

raw = sys.stdin.read()
try:
    d = json.loads(raw) if raw.strip() else {}
except:
    d = {}

score = float(d.get('evolution_score', 0.596))
pct = round(score * 100, 2)
dg = float(d.get('delta_g', 0.7388))

# 瓶颈检测：full mirror 优先；否则标准化子公式到[0,1]区间
if d.get('full_mirror') and d.get('full_bottleneck'):
    bottleneck = d.get('full_bottleneck')
else:
    dims = [
        ('Θ_llm_agent', float(d.get('theta', 0.612))),
        ('K_master', float(d.get('k_master', 1.107))),
        ('ε_self_repair', float(d.get('epsilon', 1.053))),
        ('Φ_cycle', float(d.get('phi_cycle', 1.284))),
        ('Ψ_host', float(d.get('psi_host', 0.941))),
    ]
    std = [(name, val / max(val, 1.0)) for name, val in dims]
    worst = min(std, key=lambda x: x[1])
    bottleneck = worst[0]

directives = {
    'Θ_llm_agent': 'IMPROVE_LLM_ROUTING: optimize model selection & multi-task',
    'K_master': 'REFACTOR_CODE: improve code mastery & transfer learning',
    'ε_self_repair': 'ENHANCE_REPAIR: speed up error detection & fix cycle',
    'Φ_cycle': 'BOOST_FEEDBACK: strengthen skill-up & result feedback loop',
    'Ψ_host': 'HARDEN_HOST: improve system health & resource stability',
}
directive = directives.get(bottleneck, 'MAINTAIN: all stable')

out = {
    'timestamp': '$(date -Iseconds)',
    'version': 'V10.1',
    'evolution_score': score,
    'phi_ratio_equiv': pct,
    'delta_g': dg,
    'bottleneck': bottleneck,
    'directive': directive,
    'status': 1 if pct >= 60 else 0,
    'source': 'full_mirror' if d.get('full_mirror') else 'rust_cli_baseline',
}
print(json.dumps(out))
"
}

record_history() {
  local json="$1"
  echo "$json" >> "$HISTORY"
  echo "$json" > "$LATEST"
}

report_status() {
  echo ""
  echo "========================================="
  echo "📊 V10.1 PHI 追踪报告"
  echo "========================================="
  if [ -f "$LATEST" ]; then
    local data
    data=$(python3 -c "import json; d=json.load(open('$LATEST')); [print(d.get(k,'?')) for k in ['version','phi_ratio_equiv','delta_g','bottleneck']]" 2>/dev/null)
    local ver phi dg bn
    { read -r ver; read -r phi; read -r dg; read -r bn; } <<< "$data"
    local count
    count=$(wc -l < "$HISTORY" 2>/dev/null || echo 0)
    echo "  版本:       $ver"
    echo "  PHI:        $phi%"
    echo "  ΔG:         $dg"
    echo "  瓶颈:       $bn"
    echo "  记录数:     $count"
  else
    echo "  ❌ 尚无追踪记录"
  fi
  echo "========================================="
}

main() {
  case "${1:-}" in
    --report)
      report_status
      ;;
    --history)
      if [ -f "$HISTORY" ]; then
        tail -n "${2:-10}" "$HISTORY"
      else
        echo "暂无历史"
      fi
      ;;
    *)
      log "📊 V10.1 PHI 追踪..."
      local apex_output
      apex_output=$(run_apexe)
      local phi_json
      phi_json=$(echo "$apex_output" | calculate_phi)
      record_history "$phi_json"
      log "  PHI=$(echo "$phi_json" | python3 -c "import sys,json; print(json.load(sys.stdin).get('phi_ratio_equiv','?'))")%"
      log "  瓶颈=$(echo "$phi_json" | python3 -c "import sys,json; print(json.load(sys.stdin).get('bottleneck','?'))")"
      ;;
  esac
}

main "$@"
