#!/bin/bash
# BUILD.sh - APEX Core Skill 编译脚本
# 璇玑帝国 · OpenClaw Native Integration

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== APEX Core Skill 编译 ==="

# 检查 Go
if ! command -v go &> /dev/null; then
    echo "❌ Go 未安装"
    exit 1
fi
echo "✅ Go $(go version | awk '{print $3}')"

# 编译 apex_core
echo ">>> 编译 apex_core..."
go build -o apex_core apex_core.go
echo "✅ apex_core ($(wc -c < apex_core) bytes)"

# 编译 apex_gini
echo ">>> 编译 apex_gini..."
go build -o apex_gini apex_gini.go
echo "✅ apex_gini ($(wc -c < apex_gini) bytes)"

# 安装到 ~/bin
INSTALL_DIR="$HOME/bin"
mkdir -p "$INSTALL_DIR"
cp apex_core "$INSTALL_DIR/"
cp apex_gini "$INSTALL_DIR/"

# ln -sf 覆盖旧版
if [ -f "$INSTALL_DIR/apex_core" ]; then
    echo "✅ apex_core → $INSTALL_DIR/apex_core"
fi
if [ -f "$INSTALL_DIR/apex_gini" ]; then
    echo "✅ apex_gini → $INSTALL_DIR/apex_gini"
fi

# 快速测试
echo ""
echo "=== 快速测试 ==="

echo "--- substitute ---"
"$INSTALL_DIR/apex_core" substitute -t "测试任务" -c 0.8 -h 0.6 -r 0.7

echo ""
echo "--- gini ---"
echo '["路径1内容","路径2内容","路径3内容"]' | "$INSTALL_DIR/apex_gini"

echo ""
echo "--- eval ---"
"$INSTALL_DIR/apex_core" eval -l 0.9 -t 0.8 -k 0.7 -x 0.8 -p 0.6 -f 0.6 -he 0.3 -ti 0.2 -e 0.1

echo ""
echo "=== 编译完成 ==="
