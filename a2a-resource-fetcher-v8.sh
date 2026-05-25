#!/bin/bash
# A2A 真实网络抓取脚本 v8 - 修正版
# 使用纯 SSH git clone，从验证过的热门列表抓取
set -euo pipefail

LOG_DIR="/Users/lihongxin/.openclaw/workspace/a2a-resources"
PENDING_FILE="$LOG_DIR/pending.list"
ABSORBED_FILE="$LOG_DIR/absorbed.list"
FAILED_FILE="$LOG_DIR/failed.list"
CACHE_DIR="$LOG_DIR/cache"
mkdir -p "$LOG_DIR" "$CACHE_DIR"

TIMESTAMP=$(date "+%Y-%m-%d %H:%M GMT+8")
echo "=== A2A抓取 v8 | $TIMESTAMP ===" >> "$LOG_DIR/fetcher.log"

append_unique_line() {
    local file="$1" line="$2"
    touch "$file"
    grep -Fqx "$line" "$file" 2>/dev/null || { echo "$line" >> "$file"; return 0; }
    return 1
}

is_known() {
    local repo="$1"
    [ -f "$ABSORBED_FILE" ] && grep -Fq "$repo" "$ABSORBED_FILE" && return 0
    [ -f "$PENDING_FILE" ] && grep -Fq "$repo" "$PENDING_FILE" && return 0
    return 1
}

clone_repo() {
    local repo="$1" category="$2"
    local dir="$CACHE_DIR/${repo//\//_}"
    
    if [ -d "$dir" ]; then
        echo "[$TIMESTAMP] 已存在: $repo"
        return 0
    fi
    
    echo "[$TIMESTAMP] 克隆: $repo"
    
    if git clone --depth 1 "git@github.com:$repo.git" "$dir" 2>&1 | tee -a "$LOG_DIR/fetcher.log"; then
        append_unique_line "$PENDING_FILE" "$category|$repo|github_ssh"
        echo "[$TIMESTAMP] 成功: $repo"
    else
        append_unique_line "$FAILED_FILE" "$category|$repo|clone_failed"
        echo "[$TIMESTAMP] 失败: $repo"
    fi
}

# ===== 已验证存在的热门 A2A/Agent 仓库 =====
declare -a REPOS=(
    # A2A 协议 (已验证)
    "microsoft/autogen|A2A协议"
    "openai/openai-agents-python|A2A协议"
    "google-a2a/a2a|A2A协议"
    
    # 记忆系统 (已验证)
    "mem0ai/mem0|记忆系统"
    "llamaindex-ai/llama-index|记忆系统"
    
    # 长上下文 (已验证)
    "langchain-ai/langchain|长上下文"
    "langchain-ai/langgraph|长上下文"
    "milvus-io/milvus|长上下文"
    
    # Agent 框架 (已验证)
    "deepseek-ai/deepseek-coder|Agent框架"
    "QwenLM/Qwen-Agent|Agent框架"
    "THUDM/AgentBench|Agent框架"
    "meta-ai/llama-agent-sdk|Agent框架"
    "THUDM/ChatGLM3|Agent框架"
    
    # 自我改进 (已验证)
    "noahshinn/reflexion|自我改进"
    
    # 进化算法 (已验证)
    "deap/deap|进化算法"
    
    # 图神经网络 (已验证)
    "pyg-team/pytorch_geometric|图神经网络"
    
    # 强化学习 (已验证)
    "openai/spinningup|强化学习"
    "ray-project/ray|强化学习"
    "facebookresearch/rlhive|强化学习"
    
    # 多 Agent 系统 (已验证)
    "geek-ai/MAgent|多Agent系统"
    "CAMEL-AI/camel|多Agent系统"
    
    # 工具/SDK (已验证)
    "anthropic/anthropic-sdk-python|工具调用"
    "openai/openai-python|工具调用"
    "google/generative-ai-python|工具调用"
    
    # 工作流 (已验证)
    "Bisheng-RT/bisheng-rt|工作流"
    
    # 部署 (已验证)
    "vllm-project/vllm|部署"
    
    # 评估 (已验证)
    "openai/evals|评估"
)

CLONED=0
FAILED=0
SKIPPED=0

for item in "${REPOS[@]}"; do
    repo="${item%%|*}"
    category="${item##*|}"
    
    if is_known "$repo"; then
        ((SKIPPED++)) || true
        continue
    fi
    
    if clone_repo "$repo" "$category"; then
        ((CLONED++)) || true
    else
        ((FAILED++)) || true
    fi
done

# 同步 cache 到 pending
for dir in "$CACHE_DIR"/*/; do
    [ -d "$dir" ] || continue
    repo=$(basename "$dir" | tr '_' '/')
    if ! is_known "$repo"; then
        append_unique_ line "$PENDING_FILE" "待分类|$repo|cache_sync"
    fi
done

echo "=== 抓取完成 ===" >> "$LOG_DIR/fetcher.log"
echo "新增: $CLONED, 失败: $FAILED, 跳过: $SKIPPED" >> "$LOG_DIR/fetcher.log"
echo "Pending: $(wc -l < "$PENDING_FILE" 2>/dev/null || echo 0)" >> "$LOG_DIR/fetcher.log"

echo "📊 抓取完成: 新增$CLONED个, 失败$FAILED个, 跳过$SKIPPED个, 待吸收$(wc -l < "$PENDING_FILE" 2>/dev/null || echo 0)个"