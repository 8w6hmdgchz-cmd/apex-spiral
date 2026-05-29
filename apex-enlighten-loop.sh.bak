#!/bin/bash
# ApexSpiral 全模块开智循环
# 每15分钟执行一次apex-enlighten.py

SCRIPT_DIR="/Users/lihongxin/.openclaw/workspace/apex-enlightenment"
PYTHON_SCRIPT="$SCRIPT_DIR/apex-enlighten.py"
LOG_DIR="$SCRIPT_DIR"
LOG_FILE="$LOG_DIR/enlighten-loop.log"
STATE_DIR="$LOG_DIR/state"

mkdir -p "$STATE_DIR"

echo "[$(date '+%Y-%m-%d %H:%M:%S')] 启动全模块开智循环" >> "$LOG_FILE"

while true; do
    ITER=$(date +%Y%m%d-%H%M)
    echo "[$ITER] === 循环开始 ===" >> "$LOG_FILE"
    
    # 运行开智脚本
    python3 "$PYTHON_SCRIPT" >> "$LOG_FILE" 2>&1
    
    echo "[$ITER] 循环完成，休眠15分钟" >> "$LOG_FILE"
    
    # 休眠15分钟
    sleep 900
done
