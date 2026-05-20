#!/bin/bash
# GitHub Actions Secrets 自动配置脚本
# 在本机运行（需要 gh CLI + GitHub 账号权限）
# 用已有的 SSH key 自动配置 GH_SSH_KEY secret

set -euo pipefail

REPO="8w6hmdgchz-cmd/apex-spiral"

echo "=== GitHub Secrets 配置 ==="
echo "目标仓库: $REPO"

# 检查 gh CLI
if ! command -v gh &> /dev/null; then
    echo "❌ gh CLI 未安装"
    echo "安装: brew install gh"
    exit 1
fi

# 检查 gh auth
echo "检查 gh auth 状态..."
gh auth status 2>&1 || {
    echo "❌ gh 未登录"
    echo "运行: gh auth login"
    exit 1
}

# GH_SSH_KEY: 直接用已有私钥内容
echo "配置 GH_SSH_KEY..."
SSH_KEY=$(cat ~/.ssh/id_ed25519)
echo "$SSH_KEY" | gh secret set GH_SSH_KEY --repo "$REPO" --body stdin
echo "✅ GH_SSH_KEY 配置完成"

# OPENAI_API_KEY: 需要用户输入
echo ""
echo "配置 OPENAI_API_KEY..."
echo "(去 https://platform.openai.com/api-keys 获取 API key)"
read -sp "输入 OPENAI_API_KEY: " API_KEY
echo ""
if [ -n "$API_KEY" ]; then
    echo "$API_KEY" | gh secret set OPENAI_API_KEY --repo "$REPO" --body stdin
    echo "✅ OPENAI_API_KEY 配置完成"
else
    echo "⏭️ 跳过 (你之后可以手动配)"
fi

echo ""
echo "=== 配置完成 ==="
echo "Secrets 已写入 $REPO"
echo "现在可以在 GitHub Actions 里触发 workflow 了:"
echo "  https://github.com/$REPO/actions"
