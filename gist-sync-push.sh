#!/bin/bash
# Gist 状态同步推送脚本 - 当 GitHub API 不可用时的降级方案
# 璇玑帝国APEX · Gist降级同步器
# 使用 gist.githubusercontent.com (CDN) 推送状态

set -uo pipefail

# ===== 配置 =====
LOG_DIR="/Users/lihongxin/.openclaw/workspace/apex-enlightenment"
STATE_DIR="$LOG_DIR/state"
GIST_ID_FILE="$LOG_DIR/.gist_id"
GIST_DESCRIPTION="APEX Evolver 状态同步 | $(date '+%Y-%m-%d %H:%M')"

# ===== Gist API 调用 =====
GIST_API="https://api.github.com/gists"
GH_TOKEN="${GH_TOKEN:-${FREEMODEL_API_KEY_BACKUP:-}}"

check_token() {
    if [ -z "$GH_TOKEN" ] || [ "$GH_TOKEN" = "***" ] || [ "$GH_TOKEN" = "github…fZFJ" ]; then
        echo "[GIST_PUSH] Token未配置，跳过"
        return 1
    fi
    return 0
}

load_gist_id() {
    if [ -f "$GIST_ID_FILE" ]; then
        cat "$GIST_ID_FILE" 2>/dev/null
    fi
}

save_gist_id() {
    echo "$1" > "$GIST_ID_FILE"
    echo "[GIST_PUSH] Gist ID: $1"
}

build_payload() {
    local gist_id="$1"
    python3 - <<PYEOF
import json, os

gist_id = "$gist_id" if "$gist_id" and "$gist_id" != "None" else None

files = {}
for filepath in [
    "$LOG_DIR/counter.txt",
    "$LOG_DIR/score-state.env",
    "$LOG_DIR/latest-report.md",
    "$STATE_DIR/phi_history.jsonl",
    "$STATE_DIR/defect_history.jsonl",
    "$STATE_DIR/repair_history.jsonl",
    "$STATE_DIR/bug_streak.jsonl",
    "$STATE_DIR/consistency_log.jsonl",
    "$STATE_DIR/lesson_bank.jsonl",
    "$STATE_DIR/metacognition_log.jsonl",
]:
    if os.path.exists(filepath):
        filename = os.path.basename(filepath)
        try:
            with open(filepath, 'r', errors='replace') as f:
                content = f.read()
            if content:
                files[filename] = {"content": content}
        except Exception:
            pass

result = {
    "description": "$GIST_DESCRIPTION",
    "public": False,
    "files": files
}

action = "update" if gist_id else "create"
print(json.dumps({"action": action, "gist_id": gist_id, "payload": result}))
PYEOF
}

push_gist() {
    local gist_id
    gist_id=$(load_gist_id)
    [ -z "$gist_id" ] || [ "$gist_id" = "None" ] && gist_id=""

    local payload
    payload=$(build_payload "$gist_id")

    local action actual_gist_id
    action=$(echo "$payload" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['action'])")
    actual_gist_id=$(echo "$payload" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('gist_id') or '')")

    local api_url http_method
    if [ "$action" = "update" ] && [ -n "$actual_gist_id" ]; then
        api_url="${GIST_API}/${actual_gist_id}"
        http_method="PATCH"
    else
        api_url="${GIST_API}"
        http_method="POST"
    fi

    local actual_payload
    actual_payload=$(echo "$payload" | python3 -c "import json,sys; d=json.load(sys.stdin); print(json.dumps(d['payload']))")

    local response http_code
    response=$(curl -sL -w "\nHTTP_CODE:%{http_code}" \
        -X "$http_method" \
        -H "Authorization: Bearer $GH_TOKEN" \
        -H "Content-Type: application/json" \
        -d "$actual_payload" \
        "$api_url" 2>/dev/null)

    http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    local body=$(echo "$response" | sed '/HTTP_CODE:/d')

    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        local new_gist_id
        new_gist_id=$(echo "$body" | python3 -c "import json,sys; print(json.load(sys.stdin).get('id', ''))" 2>/dev/null)

        if [ -n "$new_gist_id" ]; then
            save_gist_id "$new_gist_id"
            echo "[GIST_PUSH] ✅ 推送成功"
            echo "[GIST_PUSH] Gist: https://gist.github.com/$new_gist_id"
            return 0
        fi
    fi

    echo "[GIST_PUSH] ❌ HTTP $http_code"
    echo "[GIST_PUSH] ${body:0:200}"
    return 1
}

pull_gist() {
    local gist_id
    gist_id=$(load_gist_id)
    [ -z "$gist_id" ] || [ "$gist_id" = "None" ] && gist_id=""

    if [ -z "$gist_id" ]; then
        echo "[GIST_PULL] 无Gist ID，跳过"
        return 1
    fi

    echo "[GIST_PULL] 从Gist拉取状态..."

    local base_url="https://gist.githubusercontent.com/8w6hmdgchz-cmd/${gist_id}/raw"

    for filepath in \
        "$LOG_DIR/counter.txt" \
        "$LOG_DIR/score-state.env" \
        "$LOG_DIR/latest-report.md" \
        "$STATE_DIR/phi_history.jsonl" \
        "$STATE_DIR/defect_history.jsonl" \
        "$STATE_DIR/repair_history.jsonl" \
        "$STATE_DIR/bug_streak.jsonl" \
        "$STATE_DIR/consistency_log.jsonl" \
        "$STATE_DIR/lesson_bank.jsonl" \
        "$STATE_DIR/metacognition_log.jsonl"; do

        local filename dir content
        filename=$(basename "$filepath")
        dir=$(dirname "$filepath")
        mkdir -p "$dir"

        content=$(curl -sL --connect-timeout 8 "${base_url}/${filename}" 2>/dev/null)

        if [ -n "$content" ] && [ "$content" != "Not Found" ] && [ "${content:0:1}" != "<" ]; then
            echo "$content" > "$filepath"
            echo "[GIST_PULL] ✅ $filename"
        else
            echo "[GIST_PULL] ⏭️ $filename"
        fi
    done

    echo "[GIST_PULL] 完成"
}

MODE="${1:-push}"
echo "=== Gist同步 $(date '+%Y-%m-%d %H:%M:%S') | $MODE ==="

check_token || exit 1

case "$MODE" in
    push)  push_gist ;;
    pull)  pull_gist ;;
    sync)  pull_gist; push_gist ;;
    *)     echo "用法: $0 [push|pull|sync]" ;;
esac
