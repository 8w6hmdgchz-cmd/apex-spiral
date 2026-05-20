#!/bin/bash
# Gist 自动拉取脚本 - 用raw URL直接拉取，不需要token
# 璇玑帝国APEX · Gist神级基因吸收器

LOG_DIR="/Users/lihongxin/.openclaw/workspace/apex-enlightenment/state"
ABSORBED="$LOG_DIR/absorbed_hub"
ALLOWED_USER="8w6hmdgchz-cmd"

mkdir -p "$ABSORBED"

# 拉取函数
pull_gist() {
    local gist_id="$1"
    local label="$2"
    local expected_conf="$3"
    
    local url="https://gist.githubusercontent.com/${ALLOWED_USER}/${gist_id}/raw"
    local out_file="$ABSORBED/gist_${gist_id}.json"
    
    if [ -f "$out_file" ]; then
        echo "[$(date '+%H:%M')] ⏭️ $label 已存在"
        return 0
    fi
    
    echo -n "[$(date '+%H:%M')] 📥 $label... "
    content=$(curl -sL --connect-timeout 10 "$url" 2>/dev/null)
    
    if [ -n "$content" ]; then
        echo "$content" > "$out_file"
        echo "✅ (conf=$expected_conf)"
        return 0
    else
        echo "❌"
        return 1
    fi
}

echo "=== Gist自动拉取 | $(date '+%Y-%m-%d %H:%M') ==="

# 已知Gist资源
pull_gist "57fa0d7fc0247f91f9bb744c253c13ff" "防幻觉与自我反思" "0.95"

echo "=== 完成 ==="
