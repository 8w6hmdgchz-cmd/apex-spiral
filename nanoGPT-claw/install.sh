#!/bin/bash
# ==============================================================================
# NanoGPT-Claw 全自动安装脚本 v0.9.1
#
# 一键安装：环境检查、Rust 安装、依赖配置、编译、运行
# ==============================================================================

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}  $1"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════╝${NC}"
}

# 检查系统要求
check_system() {
    print_header "系统要求检查"

    # 检查操作系统
    OS=$(uname -s)
    if [[ "$OS" == "Linux" ]]; then
        print_success "操作系统: Linux"
    elif [[ "$OS" == "Darwin" ]]; then
        print_success "操作系统: macOS"
    else
        print_warning "操作系统: $OS (未完全测试，可能存在兼容性问题)"
    fi

    # 检查系统架构
    ARCH=$(uname -m)
    print_info "系统架构: $ARCH"

    # 检查内存
    if [[ "$OS" == "Darwin" ]]; then
        MEM=$(sysctl -n hw.memsize | awk '{print int($1/1024/1024/1024)}')
    else
        MEM=$(free -g | awk '/^Mem:/{print $2}')
    fi
    if [[ "$MEM" -ge 4 ]]; then
        print_success "内存: ${MEM}GB (≥4GB)"
    else
        print_warning "内存: ${MEM}GB (建议 ≥4GB)"
    fi

    # 检查磁盘空间
    DISK=$(df -BG . | awk 'NR==2 {print $4}' | sed 's/G//')
    if [[ "$DISK" -ge 5 ]]; then
        print_success "磁盘空间: ${DISK}GB 可用 (≥5GB)"
    else
        print_warning "磁盘空间: ${DISK}GB 可用 (建议 ≥5GB)"
    fi

    print_success "系统检查完成"
}

# 检查并安装 Rust
install_rust() {
    print_header "Rust 环境检查"

    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version)
        print_success "Rust 已安装: $RUST_VERSION"
    else
        print_warning "Rust 未安装，正在安装..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        print_success "Rust 安装完成"
    fi

    # 确保 cargo 在 PATH 中
    if ! command -v cargo &> /dev/null && [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi

    CARGO_VERSION=$(cargo --version)
    print_info "Cargo: $CARGO_VERSION"
}

# 克隆仓库（如果需要）
clone_repo() {
    print_header "获取代码"

    if [ -d ".git" ]; then
        print_info "当前目录已是 Git 仓库"
    elif [ -d "nanoGPT-claw" ]; then
        print_info "发现 nanoGPT-claw 目录"
        cd nanoGPT-claw
    else
        print_info "克隆仓库..."
        git clone https://github.com/hernandez42/nanoGPT-claw.git
        cd nanoGPT-claw
    fi
}

# 创建必要的目录
setup_directories() {
    print_header "创建目录结构"

    # 数据目录
    mkdir -p data
    print_success "创建: data/"

    # 日志目录
    mkdir -p logs
    print_success "创建: logs/"

    # 技能目录
    mkdir -p skills
    print_success "创建: skills/"

    # 配置目录
    mkdir -p "$HOME/.config/nanoGPT-claw"
    print_success "创建: $HOME/.config/nanoGPT-claw/"
}

# 配置环境变量
setup_environment() {
    print_header "配置环境变量"

    ENV_FILE="$HOME/.config/nanoGPT-claw/.env"
    ENV_TEMPLATE="$HOME/.config/nanoGPT-claw/.env.template"

    # 生成环境变量模板
    cat > "$ENV_TEMPLATE" << 'EOF'
# NanoGPT-Claw 环境变量配置
# 复制此文件为 .env 并填入你的配置
# 然后运行: export $(cat .env | xargs)

# ==============================================================================
# LLM Providers
# ==============================================================================

# OpenAI (推荐)
OPENAI_API_KEY=sk-your-openai-key-here

# Anthropic (Claude)
# ANTHROPIC_API_KEY=sk-ant-your-anthropic-key-here

# Ollama (本地模型)
# OLLAMA_BASE_URL=http://localhost:11434/v1

# ==============================================================================
# 飞书 (Lark) - 可选
# ==============================================================================

# FEISHU_APP_ID=cli_your-app-id
# FEISHU_APP_SECRET=your-app-secret
# FEISHU_VERIFY_TOKEN=your-verify-token
# FEISHU_ENCRYPT_KEY=your-encrypt-key-optional

# ==============================================================================
# GitHub - 可选
# ==============================================================================

# GITHUB_WEBHOOK_SECRET=your-webhook-secret
# GITHUB_API_TOKEN=ghp_your-github-token
EOF

    print_success "创建: $ENV_TEMPLATE"

    # 如果 .env 不存在，创建一个空的
    if [ ! -f "$ENV_FILE" ]; then
        cp "$ENV_TEMPLATE" "$ENV_FILE"
        print_warning "⚠️  请编辑 $ENV_FILE 填入你的配置"
        echo ""
        echo "  可选："
        echo "    1) 编辑 $ENV_FILE"
        echo "    2) 或者直接在当前 shell 中设置环境变量"
        echo "       export OPENAI_API_KEY=sk-xxx"
        echo ""
    else
        print_success "配置文件已存在: $ENV_FILE"
    fi
}

# 编译项目
build_project() {
    print_header "编译项目"

    print_info "正在编译 release 版本 (需要 2-5 分钟)..."

    if cargo build --release; then
        print_success "编译成功!"
    else
        print_error "编译失败!"
        exit 1
    fi

    # 检查二进制文件
    if [ -f "target/release/nano-gpt-claw" ]; then
        print_success "二进制文件: target/release/nano-gpt-claw"
    fi
}

# 创建快速启动脚本
create_launcher() {
    print_header "创建快速启动脚本"

    # 创建 run.sh
    cat > run.sh << 'EOF'
#!/bin/bash
# NanoGPT-Claw 快速启动脚本

cd "$(dirname "$0")"

# 加载环境变量
if [ -f "$HOME/.config/nanoGPT-claw/.env" ]; then
    export $(grep -v '^#' "$HOME/.config/nanoGPT-claw/.env" | xargs)
fi

# 检查是否已安装
if [ ! -f "target/release/nano-gpt-claw" ]; then
    echo "请先运行: ./install.sh"
    exit 1
fi

# 运行
exec ./target/release/nano-gpt-claw "$@"
EOF
    chmod +x run.sh
    print_success "创建: run.sh"

    # 创建 start-daemon.sh
    cat > start-daemon.sh << 'EOF'
#!/bin/bash
# NanoGPT-Claw Daemon 启动脚本

cd "$(dirname "$0")"

# 加载环境变量
if [ -f "$HOME/.config/nanoGPT-claw/.env" ]; then
    export $(grep -v '^#' "$HOME/.config/nanoGPT-claw/.env" | xargs)
fi

# 检查是否已安装
if [ ! -f "target/release/nano-gpt-claw" ]; then
    echo "请先运行: ./install.sh"
    exit 1
fi

echo "🚀 启动 NanoGPT-Claw Daemon..."
echo ""

# 先停止已有的 daemon
./target/release/nano-gpt-claw stop 2>/dev/null || true

# 启动新的 daemon
exec ./target/release/nano-gpt-claw start
EOF
    chmod +x start-daemon.sh
    print_success "创建: start-daemon.sh"

    # 创建 restart.sh
    cat > restart.sh << 'EOF'
#!/bin/bash
# NanoGPT-Claw 重启脚本

cd "$(dirname "$0")"

echo "🔄 重启 NanoGPT-Claw..."

./target/release/nano-gpt-claw stop 2>/dev/null || true
sleep 2

if [ -f "$HOME/.config/nanoGPT-claw/.env" ]; then
    export $(grep -v '^#' "$HOME/.config/nanoGPT-claw/.env" | xargs)
fi

./target/release/nano-gpt-claw start
EOF
    chmod +x restart.sh
    print_success "创建: restart.sh"
}

# 运行测试（可选）
run_tests() {
    print_header "运行测试"

    echo "是否运行完整测试套件？(需要额外时间)"
    read -p "[Y/n]: " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        print_info "正在运行测试..."
        cargo test
        print_success "测试完成"
    fi
}

# 显示使用说明
show_usage() {
    print_header "安装完成！使用说明"

    echo ""
    echo -e "  ${GREEN}✅ 安装成功！${NC}"
    echo ""
    echo "  下一步："
    echo ""
    echo "  1. 配置环境变量（重要）"
    echo "     ${CYAN}vim $HOME/.config/nanoGPT-claw/.env${NC}"
    echo "     填入你的 OpenAI API Key 等配置"
    echo ""
    echo "  2. 加载环境变量"
    echo "     ${CYAN}export \$(cat $HOME/.config/nanoGPT-claw/.env | xargs)${NC}"
    echo ""
    echo "  3. 启动向导配置（可选）"
    echo "     ${CYAN}./target/release/nano-gpt-claw setup${NC}"
    echo ""
    echo "  4. 启动 Daemon"
    echo "     ${CYAN}./start-daemon.sh${NC}"
    echo "     或 ${CYAN}./target/release/nano-gpt-claw start${NC}"
    echo ""
    echo "  5. 检查状态"
    echo "     ${CYAN}./target/release/nano-gpt-claw status${NC}"
    echo ""
    echo "  6. 停止 Daemon"
    echo "     ${CYAN}./target/release/nano-gpt-claw stop${NC}"
    echo ""
    echo "  快速命令："
    echo "    ${CYAN}./run.sh help${NC}     - 查看帮助"
    echo "    ${CYAN}./run.sh setup${NC}    - 运行配置向导"
    echo "    ${CYAN}./run.sh start${NC}    - 启动"
    echo "    ${CYAN}./restart.sh${NC}       - 重启"
    echo ""

    print_header "项目目录"
    echo ""
    echo "  $(pwd)"
    echo "    ├── target/release/nano-gpt-claw  ${GREEN}(可执行文件)${NC}"
    echo "    ├── data/                         (数据文件)"
    echo "    ├── logs/                         (日志文件)"
    echo "    ├── skills/                       (自定义技能)"
    echo "    └── $HOME/.config/nanoGPT-claw/   (配置目录)"
    echo ""
}

# 主函数
main() {
    echo ""
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}              NanoGPT-Claw 全自动安装脚本 v0.9.1                ${BLUE}║${NC}"
    echo -e "${BLUE}║${NC}          1 Main + N Aux LLM | CoT | Self-Evolution            ${BLUE}║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    # 检查是否使用 sudo
    if [[ "$EUID" -eq 0 ]]; then
        print_warning "检测到以 root 运行，建议以普通用户运行"
        echo "  提示: 以普通用户运行安装，权限问题会提示 sudo"
        echo ""
    fi

    # 执行安装步骤
    check_system
    install_rust
    clone_repo
    setup_directories
    setup_environment
    build_project
    create_launcher
    run_tests
    show_usage

    print_success "安装流程结束！"
    echo ""
}

# 运行主函数
main
