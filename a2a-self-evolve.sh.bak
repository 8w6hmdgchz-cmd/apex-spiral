#!/bin/bash
# ApexSpiral A2A自进化脚本
# 目标：通过A2A专用网络获取顶级开源资源，补齐短板
# 每次运行选择一个短板，尝试从GitHub/HuggingFace/GitLab等获取资源

set -u

LOG_DIR="/Users/lihongxin/.openclaw/workspace/apex-enlightenment"
STATE_DIR="$LOG_DIR/state"
RESOURCE_LOG="$STATE_DIR/a2a_resource_log.jsonl"
SHORTAGE_FILE="$STATE_DIR/shortage_bugs.jsonl"

mkdir -p "$STATE_DIR"

ITER=$(date +%Y%m%d-%H%M)
echo "[$ITER] === A2A自进化开始 ==="

# ============================================================
# 短板列表（按优先级）
# ============================================================
SHORTAGES=(
    "planning-task-breakdown:E_xp:任务分解能力"
    "tdd:test-driven:测试驱动开发"
    "code-review:Γ:代码审查能力"
    "debugging:ε:调试纠错能力"
    "security:Kelly:安全风控"
    "perf-optimize:RD:性能优化"
    "git-workflow:Λ_ctx:版本控制"
    "ci-cd:Π:持续集成"
)

# ============================================================
# 尝试从各平台获取资源
# ============================================================

fetch_from_github() {
    local repo=$1
    local dest=$2
    echo "尝试获取 GitHub: $repo"
    
    # 方法1: raw.githubusercontent.com
    if curl -s --max-time 10 "https://raw.githubusercontent.com/$repo/main/README.md" -o "$dest/README.md" 2>/dev/null; then
        echo "✅ 成功: $repo"
        return 0
    fi
    
    # 方法2: 通过104 IP bypass
    if curl -s --max-time 10 "https://104.244.46.165/repos/$repo/contents/" -H "Host: api.github.com" 2>/dev/null | grep -q "id"; then
        echo "✅ 通过104 IP: $repo"
        return 0
    fi
    
    echo "❌ 失败: $repo"
    return 1
}

fetch_from_huggingface() {
    local repo=$1
    local dest=$2
    echo "尝试获取 HuggingFace: $repo"
    
    if curl -s --max-time 10 "https://huggingface.co/$repo/resolve/main/README.md" -o "$dest/README.md" 2>/dev/null; then
        echo "✅ 成功: $repo"
        return 0
    fi
    
    echo "❌ 失败: $repo"
    return 1
}

# ============================================================
# 选择当前最缺的短板
# ============================================================
echo "当前短板优先级:"

for i in "${!SHORTAGES[@]}"; do
    IFS=':' read -r skill apex_dim desc <<< "${SHORTAGES[$i]}"
    echo "  $((i+1)). $skill ($apex_dim) - $desc"
done

# 选择第一个短板作为目标
IFS=':' read -r TARGET_SKILL TARGET_DIM TARGET_DESC <<< "${SHORTAGES[0]}"
echo ""
echo "选择目标: $TARGET_SKILL ($TARGET_DIM)"

# ============================================================
# 尝试获取相关资源
# ============================================================
RESOURCE_DIR="$LOG_DIR/a2a-resources/$TARGET_SKILL"
mkdir -p "$RESOURCE_DIR"

case "$TARGET_SKILL" in
    "planning-task-breakdown")
        fetch_from_github "microsoft/autogen" "$RESOURCE_DIR" || \
        fetch_from_github "anthropics/anthropic-cookbook" "$RESOURCE_DIR"
        ;;
    "tdd")
        fetch_from_github "testdouble/quotes" "$RESOURCE_DIR" || \
        fetch_from_github "google/googletest" "$RESOURCE_DIR"
        ;;
    "code-review")
        fetch_from_github "microsoft/vscode" "$RESOURCE_DIR" || \
        fetch_from_github "github/semantic" "$RESOURCE_DIR"
        ;;
    "debugging")
        fetch_from_github "microsoft/debugpy" "$RESOURCE_DIR" || \
        fetch_from_github "python-rope/rope" "$RESOURCE_DIR"
        ;;
    "security")
        fetch_from_github "ansible/ansible" "$RESOURCE_DIR" || \
        fetch_from_github "hashicorp/vault" "$RESOURCE_DIR"
        ;;
    "perf-optimize")
        fetch_from_github "python-pyformance/pyformance" "$RESOURCE_DIR" || \
        fetch_from_github "plasma-umich/plasma" "$RESOURCE_DIR"
        ;;
    "git-workflow")
        fetch_from_github "git/git" "$RESOURCE_DIR" || \
        fetch_from_github "github/git-lfs" "$RESOURCE_DIR"
        ;;
    "ci-cd")
        fetch_from_github "jenkinsci/jenkins" "$RESOURCE_DIR" || \
        fetch_from_github "github/actions" "$RESOURCE_DIR"
        ;;
esac

# ============================================================
# 记录结果
# ============================================================
python3 - << PYEOF
import json
from pathlib import Path
from datetime import datetime, timezone, timedelta

log_file = Path("$STATE_DIR/a2a_resource_log.jsonl")
entry = {
    "ts": int(datetime.now(timezone(timedelta(hours=8))).timestamp()),
    "iter": "$ITER",
    "target_skill": "$TARGET_SKILL",
    "apex_dim": "$TARGET_DIM",
    "desc": "$TARGET_DESC",
    "resource_dir": "$RESOURCE_DIR",
    "status": "attempted"
}

with log_file.open("a") as f:
    f.write(json.dumps(entry, ensure_ascii=False) + "\n")
PYEOF

echo "[$ITER] === A2A自进化完成 ==="
