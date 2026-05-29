#!/bin/bash
# ApexSpiral 开智自进化循环 v3
# 每15分钟执行一轮
# 模式: 12354/21354 交替循环 (5轮后切换)
# 目标: 持续排查自身问题与能力短板，代入公式自主解决

set -e

WORK_DIR="/Users/lihongxin/.openclaw/workspace/apex-enlightenment"
LOG_FILE="$WORK_DIR/state/enlight_loop.log"
MODE_FILE="$WORK_DIR/state/loop_mode.json"
ITER_FILE="$WORK_DIR/state/iter_count.json"

mkdir -p "$WORK_DIR/state"

# 获取当前模式
get_mode() {
    if [ -f "$MODE_FILE" ]; then
        cat "$MODE_FILE"
    else
        echo "21354"
    fi
}

# 获取迭代次数
get_iter() {
    if [ -f "$ITER_FILE" ]; then
        python3 -c "import json; print(json.load(open('$ITER_FILE'))['count'])"
    else
        echo "0"
    fi
}

# 记录日志
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

# 主循环
run_loop() {
    local iter=$(get_iter)
    local mode=$(get_mode)
    
    log "=== 开智循环第$((iter+1))轮 (模式: $mode) ==="
    
    # 执行开智
    python3 "$WORK_DIR/apex-enlighten.py" >> "$LOG_FILE" 2>&1
    
    # 检查是否需要切换模式
    local new_iter=$(get_iter)
    if [ "$new_iter" -eq 5 ]; then
        # 5轮后切换
        if [ "$mode" = "21354" ]; then
            echo "12354" > "$MODE_FILE"
            log "模式切换: 21354 → 12354"
        else
            echo "21354" > "$MODE_FILE"
            log "模式切换: 12354 → 21354"
        fi
    fi
    
    log "=== 第$new_iter轮完成 ==="
}

# 持续运行
while true; do
    run_loop
    log "等待15分钟..."
    sleep 900  # 15分钟
done
