#!/bin/bash
# ============================================
# Apex-Spiral 全自动安装脚本
# ============================================
set -e

echo "🚀 初始化 Apex-Spiral Evolver..."

# 1. 检查SSH密钥
echo "📡 检查GitHub SSH连接..."
if ! ssh -T git@github.com 2>/dev/null | grep -q "successfully authenticated"; then
    echo "❌ GitHub SSH未认证，请先配置SSH密钥"
    exit 1
fi
echo "✅ GitHub SSH已认证"

# 2. 克隆仓库
echo "📦 克隆仓库..."
if [ ! -d "apex-spiral" ]; then
    git clone git@github.com:8w6hmdgchz-cmd/apex-spiral.git
fi
cd apex-spiral

# 3. 初始化子模块
echo "🔗 初始化子模块..."
git submodule update --init --recursive

# 4. 构建Docker镜像
echo "🐳 构建Docker镜像..."
docker build -t ghcr.io/8w6hmdgchz-cmd/apex-spiral:latest .

# 5. 推送到GitHub Container Registry
echo "📤 推送镜像到GHCR..."
echo "$GITHUB_TOKEN" | docker login ghcr.io -u "$GITHUB_ACTOR" --password-stdin
docker push ghcr.io/8w6hmdgchz-cmd/apex-spiral:latest

# 6. 设置GitHub Actions Runner
echo "🏃 设置GitHub Actions Runner..."
mkdir -p runner && cd runner
container_name=apex-spiral-runner

# 7. 初始化演进基因库
echo "🧬 初始化演进基因库..."
cd ..
./evolver-hub-sync.sh

# 8. 验证安装
echo "✅ 验证安装..."
bash apex-iterate.sh --dry-run

echo ""
echo "🎉 Apex-Spiral 安装完成!"
echo ""
echo "📊 访问:"
echo "  - GitHub仓库: https://github.com/8w6hmdgchz-cmd/apex-spiral"
echo "  - Gist备份: https://gist.github.com/8w6hmdgchz-cmd/57fa0d7fc0247f91f9bb744c253c13ff"
echo "  - Docker镜像: ghcr.io/8w6hmdgchz-cmd/apex-spiral:latest"
echo ""
echo "🚀 启动命令:"
echo "  docker-compose up -d"
echo "  ./apex-iterate.sh"
