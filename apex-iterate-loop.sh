#!/bin/bash
# ApexSpiral 高级长期循环调度器
set -u
BASE_DIR="/Users/lihongxin/.openclaw/workspace/apex-enlightenment"
ITERATE_SCRIPT="$BASE_DIR/apex-iterate.sh"
LOOP_LOG="$BASE_DIR/apex-iterate-loop.log"
PID_FILE="$BASE_DIR/apex-iterate-loop.pid"
INTERVAL=900

echo $$ > "$PID_FILE"
echo "[$(date '+%Y-%m-%d %H:%M GMT+8')] advanced loop started pid=$$" >> "$LOOP_LOG"

while true; do
  echo "[$(date '+%Y-%m-%d %H:%M GMT+8')] running apex-iterate.sh" >> "$LOOP_LOG"
  bash "$ITERATE_SCRIPT" >> "$LOOP_LOG" 2>&1
  echo "[$(date '+%Y-%m-%d %H:%M GMT+8')] sleep ${INTERVAL}s" >> "$LOOP_LOG"
  sleep "$INTERVAL"
done
