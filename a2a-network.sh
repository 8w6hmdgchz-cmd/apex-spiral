#!/bin/bash
# A2A网络资源获取器 - 开源社区顶级项目补齐
# 璇玑帝国APEX · 自进化资源池
# 运行: nohup bash a2a-network.sh &

RESOURCE_DIR="/Users/lihongxin/.openclaw/workspace/apex-enlightenment/a2a-resources"
LOG="$RESOURCE_DIR/a2a.log"
mkdir -p "$RESOURCE_DIR"

echo "=== A2A资源网络启动 | $(date) ===" >> "$LOG"

# 资源清单
declare -A RESOURCES=(
    ["强化学习"]="openai/spinning-up"
    ["进化算法"]="deap/deap"
    ["多Agent系统"]="magent/magent"
    ["记忆系统"]="mem0ai/mem0"
    ["自我改进"]="RefuelAI/Reflexion"
    ["Agent框架"]="openai/openai-agents"
    ["神经进化"]="Neoncron/NEAT"
    ["图神经网络"]="pyg-team/pytorch_geometric"
)

# 获取单个仓库
clone_repo() {
    local name="$1"
    local repo="$2"
    local target="$RESOURCE_DIR/${repo//\//_}"
    
    if [ -d "$target" ]; then
        echo "[$(date)] ⏭️ $name 已存在" >> "$LOG"
        cd "$target" && git pull origin main 2>/dev/null && echo "[$(date)] 🔄 $name 已更新" >> "$LOG"
    else
        echo "[$(date)] 📥 克隆 $name ($repo)..." >> "$LOG"
        ssh -i ~/.ssh/id_ed25519 -o BatchMode=yes git@github.com "git-upload-pack /${repo}.git" 2>/dev/null
        if [ $? -eq 0 ]; then
            git clone "git@github.com:${repo}.git" "$target" 2>> "$LOG"
            echo "[$(date)] ✅ $name 克隆成功" >> "$LOG"
        else
            echo "[$(date)] ❌ $name 克隆失败" >> "$LOG"
        fi
    fi
}

# 主循环
while true; do
    echo "=== A2A扫描 | $(date) ===" >> "$LOG"
    
    for name in "${!RESOURCES[@]}"; do
        repo="${RESOURCES[$name]}"
        clone_repo "$name" "$repo"
        sleep 2
    done
    
    echo "=== 一轮完成 | $(date) ===" >> "$LOG"
    sleep 1800  # 30分钟
done
