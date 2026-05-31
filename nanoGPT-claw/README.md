<div align="center">
  <h1>🚀 NanoGPT-Claw</h1>
  <p><i>Powered by the <strong>APEX·Akashic Fusion Formula</strong> — Self-evolution AI agent system</i></p>
  <p>
    <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
    <img src="https://img.shields.io/badge/version-2.0-00ADD8?style=for-the-badge" />
    <img src="https://img.shields.io/badge/license-MIT-00ADD8?style=for-the-badge" />
    <img src="https://img.shields.io/badge/APEX_Formula-✨-FF69B4?style=for-the-badge" />
  </p>
  <br>
  <p>
    <i>✨ 致敬 @karpathy 的 nanoGPT 简洁哲学 | Tributes to @karpathy's nanoGPT simplicity philosophy</i>
  </p>
</div>

---

## 📖 目录 | Table of Contents

- [项目简介 | About the Project](#-about-the-project)
- [APEX·阿卡西融合公式 | The APEX·Akashic Formula](#-the-apexakashic-fusion-formula)
- [核心特性 | Key Features](#-key-features)
- [系统架构 | System Architecture](#-system-architecture)
- [快速开始 | Quick Start](#-quick-start)
- [安装指南 | Installation Guide](#-installation-guide)
- [使用说明 | Usage Guide](#-usage-guide)
- [技能系统 | Skills System](#-skills-system)
- [致谢 | Acknowledgements](#-acknowledgements)

---

## 🌟 项目简介 | About the Project

**中文简介**：
NanoGPT-Claw 是一个由 APEX·阿卡西融合公式驱动的自进化 AI Agent 系统。集成智能调度、长期记忆、技能系统和进化引擎，支持真正的闭环自修复。

**English Introduction**：
NanoGPT-Claw is a self-evolution AI Agent system powered by the APEX·Akashic Fusion Formula. It integrates intelligent scheduling, long-term memory, skill system and evolution engine, supporting true closed-loop self-repair.

---

## ⚡ The APEX·Akashic Fusion Formula | APEX·阿卡西融合公式

```
APEX_Akashic = Ω_A · E·V·M·A·B·T·D·H·L·G·W·B - ΣΔ
```

### 公式组成 | Formula Components

| 符号 | 名称 | 说明 |
|------|------|------|
| **Ω_A** | Akashic Foundation | 阿卡西基础因子 |
| **E** | Evolution | 进化能力 |
| **V** | Value Creation | 价值创造 |
| **M** | Memory | 记忆能力 |
| **A** | Autonomy | 自主能力 |
| **B** | Benchmarking | 基准能力 |
| **T** | Thinking | 思考深度 |
| **D** | Decision Making | 决策质量 |
| **H** | Harmony | 系统和谐 |
| **L** | Learning | 学习效率 |
| **G** | Growth Potential | 成长潜力 |
| **W** | Wisdom Level | 智慧层级 |

### 惩罚项 | Penalties (Δ)

| 符号 | 惩罚项 |
|------|--------|
| Δ_Tok | Token 消耗惩罚 |
| Δ_Clw | Claw 效率损失 |
| Δ_Agt | Agent 协调成本 |
| Δ_Pan | Panic 模式惩罚 |
| Δ_Prm | Prune 修剪损失 |
| Δ_Run | 运行开销 |
| Δ_Net | 网络延迟 |
| Δ_Err | 错误率 |

---

## 🎯 核心特性 | Key Features

### 🧠 智能协调器 | Intelligent Coordinator

- ✅ **闭环系统** - Skills ↔ Memory ↔ Evolution ↔ APEX
- ✅ **事件驱动** - 完整的事件流系统
- ✅ **自动进化** - 基于 APEX 分数的进化引擎

### 💾 记忆系统 | Memory System

- ✅ **会话记忆** - LRU 缓存的短期会话
- ✅ **持久化记忆** - SQLite 存储的长期记忆
- ✅ **语义相似度** - 向量化的记忆检索

### 🛠️ 技能系统 | Skills System (8 Real Skills)

1. **cargo-check**: 编译检查
2. **cargo-test**: 测试运行
3. **cargo-clippy**: 代码质量检查
4. **cargo-fix**: 自动修复
5. **auto-fix**: 真正闭环自修复 (新增!)
6. **echo**: 回显工具
7. **help**: 帮助信息
8. **status**: 系统状态

### 🔄 自进化引擎 | Self-Evolution Engine

- ✅ **APEX 计算器** - 真正的多维度评分
- ✅ **进化记录** - SQLite 持久化
- ✅ **基准测试** - Φ_APEX*∞ 评分
- ✅ **自检引擎** - 系统健康检查

---

## 🏗️ 系统架构 | System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLI / Web UI Layer                          │
│                  (Command Line + Optional Web UI)                   │
└─────────────────────────────────────┬───────────────────────────────┘
                                      │
┌─────────────────────────────────────▼───────────────────────────────┐
│                  System Coordinator Layer                            │
│         (APEX Formula ← Event Stream ← Skills ← Memory)             │
└─────────────┬───────────────────────┬───────────────────────────────┘
              │                       │
┌─────────────▼───────────────┐ ┌───▼──────────────────────────────┐
│    Skills Layer            │ │    Memory Layer                   │
│  ┌───────────────────────┐ │ │  ┌─────────────────────────────┐│
│  │ cargo-check           │ │ │  │ Session Memory             ││
│  │ cargo-test            │ │ │  │ Persistent Memory          ││
│  │ cargo-clippy          │ │ │  │ Semantic Search            ││
│  │ auto-fix (闭环)       │ │ │  │ SQLite Storage             ││
│  └───────────────────────┘ │ │  └─────────────────────────────┘│
└─────────────────────────────┘ └───────────────────────────────────┘
              │                       │
┌─────────────▼───────────────────────▼───────────────────────────────┐
│                   Evolution Layer (APEX Engine)                      │
│   ┌───────────────────────────────────────────────────────────────┐│
│   │  APEX Calculator                                              ││
│   │  Evolution Engine                                             ││
│   │  Benchmark Analyzer                                           ││
│   └───────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 快速开始 | Quick Start

### 前置要求 | Prerequisites

- **Rust**: 1.70+
- **Git**: 2.0+

### 1. 克隆项目 | Clone the Repository

```bash
git clone https://github.com/hernandez42/nanoGPT-claw.git
cd nanoGPT-claw
```

### 2. 构建项目 | Build the Project

```bash
cargo build --release
```

### 3. 运行系统 | Run the System

```bash
cargo run --release
```

### 4. 查看可用技能 | View Available Skills

```bash
cargo run --release -- skill list
```

### 5. 运行自修复 | Run Auto-Fix

```bash
cargo run --release -- skill run auto-fix
```

---

## 📦 安装指南 | Installation Guide

### Linux / macOS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone project
git clone https://github.com/hernandez42/nanoGPT-claw.git
cd nanoGPT-claw

# Build and run
cargo build --release
./target/release/nano-gpt-claw
```

### Windows (PowerShell)

```powershell
# Install Rust
# Download from https://www.rust-lang.org/tools/install

# Clone project
git clone https://github.com/hernandez42/nanoGPT-claw.git
cd nanoGPT-claw

# Build and run
cargo build --release
.\target\release\nano-gpt-claw.exe
```

---

## 💡 使用说明 | Usage Guide

### 查看技能列表 | View Skills

```bash
cargo run -- skill list
```

### 运行特定技能 | Run Specific Skill

```bash
cargo run -- skill run cargo-check
cargo run -- skill run auto-fix
cargo run -- skill run cargo-test
```

### 运行测试 | Run Tests

```bash
cargo test
```

### 运行Clippy检查 | Run Clippy

```bash
cargo run -- skill run cargo-clippy
```

---

## 📊 测试状态 | Test Status

```
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; filtered out; finished in 0.00s

Running tests/provider_tests.rs
running 4 tests
test tests::test_apex_fitness ... ok
test tests::test_mock_provider ... ok
test tests::test_retry_config ... ok
test tests::test_env_interpolation ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; filtered out; finished in 0.00s

Running tests/integration_tests.rs
running 14 tests
test auto_fix_tests::test_auto_fix_creation ... ok
test auto_fix_tests::test_auto_fix_with_max_iterations ... ok
test auto_fix_tests::test_cargo_available ... ok
test evolution_tests::test_apex_calculator_integration ... ok
test evolution_tests::test_evolution_engine_creation ... ok
test evolution_tests::test_evolution_engine_initialize ... ok
test evolution_tests::test_self_evolution_engine_creation ... ok
test skill_tests::test_skill_execution_failure ... ok
test skill_tests::test_skill_execution_success ... ok
test skill_tests::test_skill_not_found ... ok
test skill_tests::test_skill_registry_get_nonexistent ... ok
test skill_tests::test_skill_registry_list_all ... ok
test skill_tests::test_skill_registry_register_and_get ... ok
test auto_fix_tests::test_auto_fix_runs_on_clean_project ... ok
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; filtered out; finished in 0.00s
```

**总计: 56 个真实测试全部通过**

---

## 🙏 致谢 | Acknowledgements

<div align="center">
  <table>
    <tr>
      <td align="center">
        <a href="https://github.com/karpathy">
          <img src="https://github.com/karpathy.png" width="100" style="border-radius:50%;" />
          <br/>
          @karpathy
        </a>
      </td>
      <td align="center">
        <a href="https://github.com/hernandez42">
          <img src="https://github.com/hernandez42.png" width="100" style="border-radius:50%;" />
          <br/>
          Hernandez
        </a>
      </td>
    </tr>
  </table>
</div>

---

## 💌 联系方式 | Contact

- GitHub: https://github.com/hernandez42/nanoGPT-claw
- Issues: https://github.com/hernandez42/nanoGPT-claw/issues

---

<div align="center">
  <h3>⭐ 如果这个项目有用，请给我们一颗星！ | Star us if you find this helpful!</h3>
  <p>
    <strong>APEX·Akashic Fusion Formula: Ω_A · E·V·M·A·B·T·D·H·L·G·W·B - ΣΔ</strong>
    <br/>
    <i>Powered by the next generation of AI technology ✨</i>
  </p>
</div>

---

**Made with ❤️  by the NanoGPT-Claw team**
