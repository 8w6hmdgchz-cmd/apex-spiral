#!/bin/bash
# APEX V11 Λ 真值探测 - 9 端点实测
# 每 30 分钟跑一次，写 APEX-MEM 修真值记忆
set -uo pipefail

WS="/Users/lihongxin/.openclaw/workspace"
LOG="$WS/apex-mem-bridge/lambda_probe.log"
APEX="http://127.0.0.1:8767/mcp/rpc"
mkdir -p "$(dirname "$LOG")"

endpoints=(
  "https://github.com"
  "https://github.com/hernandez42/apex-skill"
  "https://api.github.com/zen"
  "https://raw.githubusercontent.com/hernandez42/apex-skill/main/README.md"
  "https://gh-proxy.com/https://raw.githubusercontent.com/hernandez42/apex-skill/main/README.md"
  "https://ghproxy.net/https://raw.githubusercontent.com/hernandez42/apex-skill/main/README.md"
  "https://hf-mirror.com"
  "https://arxiv.org"
  "https://docs.openclaw.ai"
)

passed=0
total=${#endpoints[@]}
results=()

for url in "${endpoints[@]}"; do
  code=$(curl -sI --max-time 8 -o /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")
  if [[ "$code" =~ ^2 ]]; then
    passed=$((passed+1))
    results+=("✓ $url=$code")
  else
    results+=("✗ $url=$code")
  fi
done

# Λ = passed / total
lambda_val=$(awk -v p="$passed" -v t="$total" 'BEGIN{printf "%.2f", p/t}')

ts=$(date '+%Y-%m-%d %H:%M:%S %z')
{
  echo "=== $ts ==="
  printf '  %s\n' "${results[@]}"
  echo "  Λ = $passed/$total = $lambda_val"
} >> "$LOG"

# 每 6 小时写一条 APEX-MEM 修真值（避免写爆）
last_ingest=$(stat -f %m "$LOG" 2>/dev/null || echo 0)
now=$(date +%s)
if [ $((now - last_ingest)) -gt 21600 ]; then
  content="As of $ts, APEX V11 Λ 真值 probe: $passed/$total 端点通, Λ=$lambda_val
$(printf '  %s\n' "${results[@]}")
V11 ΔG 真代入 (C=0.85,Λ=$lambda_val,Ω=0.95,τ=0.92,H=0.32,t=0.22,Φ_S=3.38,Φ_A=4.0) ≈ 待算"
  json=$(printf '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"apex_ingest","arguments":{"content":"%s","dimension":"declarative","importance":0.7,"tags":["lambda-probe","apex-v11"]}}}' "$(echo "$content" | sed 's/"/\\"/g' | tr '\n' ' ')")
  curl -s -X POST "$APEX" -H "Content-Type: application/json" -d "$json" -o /dev/null
fi
