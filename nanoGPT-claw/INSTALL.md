# NanoGPT-Claw 完整安装指南 v0.9.1

## 🚀 快速开始（一键安装）

### 最简单：使用全自动安装脚本

```bash
# 1. 下载并运行安装脚本
curl -fsSL https://raw.githubusercontent.com/hernandez42/nanoGPT-claw/main/install.sh -o install.sh
chmod +x install.sh
./install.sh
```

或者，如果你已经克隆了仓库：

```bash
cd nanoGPT-claw
chmod +x install.sh
./install.sh
```

---

## 📋 目录

- [系统要求](#系统要求)
- [手动安装步骤](#手动安装步骤)
- [配置说明](#配置说明)
- [运行方式](#运行方式)
- [故障排查](#故障排查)
- [更新升级](#更新升级)

---

## 💻 系统要求

### 最低配置
- **操作系统**: Linux / macOS / Windows (WSL2)
- **内存**: 4GB RAM (最低)
- **磁盘空间**: 5GB 可用空间
- **Rust**: 1.70+ (自动安装)

### 推荐配置
- **操作系统**: Linux (Ubuntu 20.04+, Debian 11+, CentOS 8+)
- **内存**: 8GB+ RAM
- **磁盘空间**: 20GB+ 可用空间
- **CPU**: 2 核心或更多

---

## 🔧 手动安装步骤

### 1. 安装 Rust

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 加载环境变量
source $HOME/.cargo/env
```

### 2. 克隆仓库

```bash
git clone https://github.com/hernandez42/nanoGPT-claw.git
cd nanoGPT-claw
```

### 3. 创建必要的目录

```bash
mkdir -p data logs skills
mkdir -p $HOME/.config/nanoGPT-claw
```

### 4. 配置环境变量

```bash
# 复制环境变量模板
cp .env.example $HOME/.config/nanoGPT-claw/.env

# 编辑配置文件
vim $HOME/.config/nanoGPT-claw/.env
```

**填入你的配置**（至少需要 OpenAI API Key）：

```bash
# ~/.config/nanoGPT-claw/.env
OPENAI_API_KEY=sk-your-openai-key-here

# 可选：飞书配置
# FEISHU_APP_ID=cli_your-app-id
# FEISHU_APP_SECRET=your-app-secret
# FEISHU_VERIFY_TOKEN=your-verify-token

# 可选：GitHub配置
# GITHUB_WEBHOOK_SECRET=your-secret
```

### 5. 编译项目

```bash
# 编译 release 版本（推荐，性能更好）
cargo build --release

# 编译完成后，二进制文件在 target/release/
ls -lh target/release/nano-gpt-claw
```

---

## ⚙️ 配置说明

### 环境变量配置

所有敏感配置（API Keys、Secrets）都通过环境变量加载，**不会**写入任何配置文件！

| 环境变量 | 说明 | 必填 |
|---------|------|------|
| `OPENAI_API_KEY` | OpenAI API Key | 是（如果使用 OpenAI） |
| `ANTHROPIC_API_KEY` | Anthropic API Key | 否 |
| `OLLAMA_BASE_URL` | Ollama API URL | 否 |
| `FEISHU_APP_ID` | 飞书 App ID | 否 |
| `FEISHU_APP_SECRET` | 飞书 App Secret | 否 |
| `FEISHU_VERIFY_TOKEN` | 飞书 Verify Token | 否 |
| `FEISHU_ENCRYPT_KEY` | 飞书 Encrypt Key | 否 |
| `GITHUB_WEBHOOK_SECRET` | GitHub Webhook Secret | 否 |
| `GITHUB_API_TOKEN` | GitHub API Token | 否 |

### 加载环境变量

```bash
# 方式 1：使用 .env 文件
export $(cat $HOME/.config/nanoGPT-claw/.env | xargs)

# 方式 2：直接在 shell 中设置
export OPENAI_API_KEY=sk-your-key-here

# 方式 3：添加到 ~/.bashrc 或 ~/.zshrc 永久生效
echo 'export OPENAI_API_KEY=sk-your-key-here' >> ~/.bashrc
source ~/.bashrc
```

### 验证环境变量

```bash
# 检查是否已设置
echo $OPENAI_API_KEY

# 运行帮助命令查看状态
./target/release/nano-gpt-claw help
```

---

## 🏃 运行方式

### 方式 1：使用提供的脚本（推荐）

```bash
# 运行配置向导
./run.sh setup

# 启动 Daemon
./start-daemon.sh

# 检查状态
./run.sh status

# 停止 Daemon
./run.sh stop
```

### 方式 2：直接运行

```bash
# 先加载环境变量
export $(cat $HOME/.config/nanoGPT-claw/.env | xargs)

# 查看帮助
./target/release/nano-gpt-claw help

# 首次运行（配置向导）
./target/release/nano-gpt-claw setup

# 启动 Daemon
./target/release/nano-gpt-claw start

# 检查状态
./target/release/nano-gpt-claw status

# 停止 Daemon
./target/release/nano-gpt-claw stop
```

### 方式 3：后台运行（systemd）

创建 `/etc/systemd/system/nano-gpt-claw.service`：

```ini
[Unit]
Description=NanoGPT-Claw Daemon
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/path/to/nanoGPT-claw
Environment="OPENAI_API_KEY=sk-your-key-here"
Environment="FEISHU_APP_ID=cli_your-id"
Environment="FEISHU_APP_SECRET=your-secret"
Environment="FEISHU_VERIFY_TOKEN=your-token"
ExecStart=/path/to/nanoGPT-claw/target/release/nano-gpt-claw start
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启用并运行：

```bash
sudo systemctl daemon-reload
sudo systemctl enable nano-gpt-claw
sudo systemctl start nano-gpt-claw
sudo systemctl status nano-gpt-claw
```

---

## 🔍 故障排查

### 问题：编译失败

```bash
# 更新 Rust
rustup update

# 清理并重新编译
cargo clean
cargo build --release
```

### 问题：找不到 cargo 命令

```bash
# 加载环境变量
source $HOME/.cargo/env

# 或添加到 ~/.bashrc
echo 'source $HOME/.cargo/env' >> ~/.bashrc
source ~/.bashrc
```

### 问题：LLM 不工作

检查环境变量是否正确设置：

```bash
# 查看当前设置的环境变量
env | grep -E "(OPENAI|ANTHROPIC|OLLAMA)"

# 重新加载
export $(cat $HOME/.config/nanoGPT-claw/.env | xargs)

# 再次检查
./target/release/nano-gpt-claw status
```

### 问题：权限错误

```bash
# 确保目录有正确权限
chmod +x install.sh
chmod +x run.sh
chmod +x start-daemon.sh

# 数据目录可写
chmod 755 data logs
```

### 问题：Daemon 无法启动

```bash
# 查看详细日志
RUST_LOG=debug ./target/release/nano-gpt-claw start

# 检查 PID 文件
ls -la /tmp/nano-gpt-claw.pid
rm -f /tmp/nano-gpt-claw.pid  # 清理 stale PID

# 再次启动
./target/release/nano-gpt-claw start
```

---

## 🔄 更新升级

### 从 GitHub 更新

```bash
cd nanoGPT-claw

# 拉取最新代码
git pull origin main

# 重新编译
cargo build --release

# 重启服务（如果正在运行）
./restart.sh
```

### 强制重新编译

```bash
cd nanoGPT-claw
cargo clean
cargo build --release
```

---

## 📞 获取帮助

### 查看命令帮助

```bash
./target/release/nano-gpt-claw help
```

### 查看状态

```bash
./target/release/nano-gpt-claw status
```

### 日志文件

```bash
# 查看日志
tail -f logs/app.log

# 或查看系统日志
journalctl -u nano-gpt-claw -f  # 如果使用 systemd
```

---

## 🎯 下一步

1. ✅ 完成安装
2. ⚙️ 配置环境变量
3. 🚀 运行 `./target/release/nano-gpt-claw setup`
4. ▶️ 启动 Daemon: `./start-daemon.sh`
5. 📊 查看状态: `./run.sh status`

---

## 📝 许可证

本项目使用 MIT 许可证。详见 [LICENSE](./LICENSE) 文件。
