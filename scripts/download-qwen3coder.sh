#!/bin/bash
# 下载 Qwen3-Coder-Next-MLX-6bit 损坏文件 (00002-00008)
# 每个文件约 4.8GB，目标: hf-mirror.com

MODEL_DIR=~/models/Qwen3-Coder-Next-MLX-6bit
URL_BASE="https://hf-mirror.com/lmstudio-community/Qwen3-Coder-Next-MLX-6bit/resolve/main"

for i in 02 03 04 05 06 07 08; do
    FILE="model-0000${i}-of-00013.safetensors"
    DEST="$MODEL_DIR/$FILE"
    URL="$URL_BASE/$FILE"
    
    echo "[$(date '+%H:%M:%S')] Starting $FILE..."
    
    # 检查是否已完整
    python3 -c "
from safetensors import safe_open
import os, sys
try:
    with safe_open('$DEST', framework='mlx') as sf:
        list(sf.keys())
    print('ALREADY_VALID')
except:
    print('NEED_DOWNLOAD')
" > /tmp/dl_status_$i 2>/dev/null
    
    if grep -q "ALREADY_VALID" /tmp/dl_status_$i; then
        echo "  -> $FILE already valid, skipping"
        continue
    fi
    
    # 下载
    curl -L --max-time 600 "$URL" -o "$DEST" 2>/dev/null
    RESULT=$?
    
    # 验证
    python3 -c "
from safetensors import safe_open
try:
    with safe_open('$DEST', framework='mlx') as sf:
        list(sf.keys())
    print('VALID')
except Exception as e:
    print(f'INVALID: {e}')
" > /tmp/dl_verify_$i 2>/dev/null
    
    if grep -q "VALID" /tmp/dl_verify_$i; then
        echo "  -> $FILE OK"
    else
        echo "  -> $FILE FAILED (curl exit: $RESULT)"
        echo "  -> $(cat /tmp/dl_verify_$i)"
    fi
done

echo "[$(date '+%H:%M:%S')] All downloads done"
