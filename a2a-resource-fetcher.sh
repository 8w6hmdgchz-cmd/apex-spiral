#!/bin/bash
# A2A网络资源获取脚本 v2
# 按短板关键词自动触发资源获取
# v2: 移除无效repo映射，只保留真实可下载的资源

set -euo pipefail

LOG_DIR="/Users/lihongxin/.openclaw/workspace/apex-enlightenment/a2a-resources"
PENDING_FILE="$LOG_DIR/pending.list"
REQUESTS_FILE="$LOG_DIR/requests.log"
mkdir -p "$LOG_DIR"

TIMESTAMP=$(date "+%Y-%m-%d %H:%M GMT+8")
echo "=== A2A资源获取 | $TIMESTAMP ===" >> "$LOG_DIR/fetcher.log"

append_unique_line() {
    local file="$1"
    local line="$2"
    touch "$file"
    if ! grep -Fqx "$line" "$file" 2>/dev/null; then
        echo "$line" >> "$file"
    fi
}

map_keyword_to_resource() {
    local keyword="$1"
    case "$keyword" in
        protocol_submission|task_complete|decision_flow|report_message|a2a_protocol)
            # 有效替代: microsoft/autogen
            printf '%s\n' \
              "任务编排|microsoft/autogen"
            ;;
        memory_consolidation|retrieval|long_context|memory_system)
            printf '%s\n' \
              "记忆系统|mem0ai/mem0" \
              "长上下文|langchain-ai/langgraph"
            ;;
        reflection_checklist|debugging|failure_analysis|repair_capsule)
            # 有效替代: deap/deap
            printf '%s\n' \
              "进化算法|deap/deap"
            ;;
        routing|orchestration|multi_agent|workflow)
            # 无有效替代，跳过避免污染
            ;;
        adaptive_loop|feedback_control|self_improvement|resource_orchestration)
            # 有效替代: pyg-team/pytorch_geometric
            printf '%s\n' \
              "图神经网络|pyg-team/pytorch_geometric"
            ;;
        capability_gap|missing_module|environment_fix|dependency_repair)
            printf '%s\n' \
              "进化算法|deap/deap"
            ;;
        *)
            ;;
    esac
}

trigger_from_keywords() {
    local keywords_csv="${1:-}"
    if [ -z "$keywords_csv" ]; then
        echo "[$TIMESTAMP] 无关键词，跳过触发" >> "$LOG_DIR/fetcher.log"
        return 0
    fi

    IFS=',' read -r -a keywords <<< "$keywords_csv"
    for raw in "${keywords[@]}"; do
        keyword="$(printf '%s' "$raw" | xargs)"
        [ -z "$keyword" ] && continue
        echo "[$TIMESTAMP] 触发关键词: $keyword" >> "$REQUESTS_FILE"
        while IFS= read -r mapping; do
            [ -z "$mapping" ] && continue
            name="${mapping%%|*}"
            repo="${mapping##*|}"
            append_unique_line "$PENDING_FILE" "$name|$repo|$keyword"
            echo "[$TIMESTAMP] 资源映射: $keyword -> $name ($repo)" >> "$LOG_DIR/fetcher.log"
        done < <(map_keyword_to_resource "$keyword")
    done

    echo "=== 资源列表已更新 ===" >> "$LOG_DIR/fetcher.log"
    echo "待获取: $(wc -l < "$PENDING_FILE" 2>/dev/null || echo 0) 个条目" >> "$LOG_DIR/fetcher.log"
}

if [ "${1:-}" = "--from-keywords" ]; then
    trigger_from_keywords "${2:-}"
    exit 0
fi

# v2: 只保留有效repo，移除所有无效的
BASE_SHORTAGES=(
    "进化算法|deap/deap|bootstrap"
    "记忆系统|mem0ai/mem0|bootstrap"
    "长上下文|langchain-ai/langgraph|bootstrap"
    "任务编排|microsoft/autogen|bootstrap"
    "图神经网络|pyg-team/pytorch_geometric|bootstrap"
)

for item in "${BASE_SHORTAGES[@]}"; do
    append_unique_line "$PENDING_FILE" "$item"
done

echo "=== 基础资源已写入（仅有效repo）===" >> "$LOG_DIR/fetcher.log"
echo "待获取: $(wc -l < "$PENDING_FILE" 2>/dev/null || echo 0) 个条目"
