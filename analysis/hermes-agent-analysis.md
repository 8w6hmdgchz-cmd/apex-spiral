# Hermes Agent 仓库分析

> 仓库路径：`/Users/lihongxin/Desktop/开智/hermes-agent/`
> 维护方：Nous Research（hernandez42 主导）
> 当前版本：v0.14.0（2026-05-16 发布）
> 分析时间：2026-06-02
> Token 预算：严格 6 文件 < 200 行输出

## 1. 项目定位

**Hermes Agent**（☤）是 Nous Research 出品的"自进化 AI Agent"——README 自称"the only agent with a built-in learning loop"。定位差异化点：

- **唯一带学习闭环**：从经验中**自动创建 skills**、使用时**自我改进 skills**、周期 nudge 持久化知识、检索历史对话、用 Honcho 做用户建模
- **OpenClaw 的继任者**：内建 `hermes claw migrate` 子命令，从 OpenClaw 迁移 SOUL.md / MEMORY.md / skills / API keys
- **多端部署**：`pip install hermes-agent` 即可；同时支持本地、Docker、SSH、Modal、Daytona、Singularity、Vercel Sandbox 7 种后端
- **22 个 IM 平台**：Telegram / Discord / Slack / WhatsApp / Signal / Matrix / Mattermost / Email / SMS / 钉钉 / 企业微信 / 微信 / 飞书 / QQ 机器人 / BlueBubbles / 腾讯元宝 / LINE / SimpleX / Home Assistant / Microsoft Teams / Dingtalk / Webhook

## 2. 技术栈（Python + LLM 集成）

- **Python ≥ 3.11**；构建系统 `setuptools >= 61.0`；包管理 `uv`（uv.lock）
- **核心依赖全部精确锁版本**（`==X.Y.Z`）—— 2026-05-12 收紧，原因：Mini Shai-Hulud PyPI 蠕虫攻击了 `mistralai 2.4.6`，范围锁会无审查拉取
- **核心栈**：`openai==2.24.0`（OpenAI 兼容协议）、`pydantic==2.13.4`、`httpx[socks]==0.28.1`、`rich==14.3.3`、`prompt_toolkit==3.0.52`、`tenacity==9.1.4`、`croniter==6.0.0`、`psutil==7.2.2`
- **LLM 路由（17+ Provider）**：Nous Portal / OpenRouter / NovitaAI / NVIDIA NIM / Xiaomi MiMo / z.ai GLM / Kimi-Moonshot / MiniMax / Hugging Face / OpenAI / Anthropic / xAI Grok（SuperGrok OAuth）/ Bedrock / Azure / Google / Codex app-server
- **额外**：`anthropic`（原生 SDK）、`exa`/`firecrawl`/`parallel-web`（搜索）、`fal`（图像）、`edge-tts`/`elevenlabs`（TTS）、`modal`/`daytona`/`vercel`（serverless）、`mcp`（MCP 协议）、`acp`（Agent Client Protocol）
- **lazy-install 框架**：`tools/lazy_deps.py`——只在用户首次用到对应后端时才 pip install，减小 blast radius
- **测试栈**：`pytest==9.0.2` + `pytest-asyncio` + `pytest-timeout=30s`，并行子进程隔离（`scripts/run_tests_parallel.py`），约 17k 测试 / 900 文件

## 3. 14 个版本迭代路径

仓库含 13 个 RELEASE 文件（v0.2.0 → v0.14.0）+ v0.10.0：

- **v0.2–v0.6**（早期）：基础 CLI、Gateway、Tools、Memory、Honcho
- **v0.7–v0.9**（功能扩展）：Skills 系统、Plugin 框架、多平台 IM
- **v0.10–v0.11**（架构重组）：`agent/` 子包拆分（run_agent.py 3600 行 → 模块化）、TUI（Ink + React）替换 prompt_toolkit
- **v0.12–v0.13**（生态完善）：ACP 协议、Codex 集成、Trajectory 压缩（为训练下一代 tool-calling 模型准备数据）
- **v0.14.0 "Foundation Release"**（2026-05-16，**808 commits / 633 PRs / 215 贡献者 / 165k 行新增 / 12 P0 修复**）：xAI Grok 1M context、OpenAI-兼容本地代理、x_search、Microsoft Teams 端到端、debloat wave（lazy-deps）、`pip install hermes-agent`、跨会话 1h Claude prompt cache、180× 加速 browser_console、cold-start 削减 19s、22 个 IM 平台、/handoff 实活、LSP 语义诊断、Zed ACP Registry、`uvx` 启动、原生 Windows beta

## 4. 核心架构（agent/cli/plugins/skills 怎么协作）

**模块分布**（来自 AGENTS.md）：
```
run_agent.py        # AIAgent 类 — 核心对话循环（~12k LOC）
model_tools.py      # 工具编排（discover_builtin_tools、handle_function_call）
toolsets.py         # 工具集定义
cli.py              # HermesCLI — 交互式 CLI 编排（~11k LOC）
hermes_state.py     # SessionDB — SQLite 会话存储（FTS5 全文检索）
hermes_constants.py # get_hermes_home() — profile-aware 路径
agent/              # Provider 适配器、Memory、Caching、Compression
hermes_cli/         # CLI 子命令、setup wizard、plugins loader、skin engine
tools/              # 工具实现（registry.py 自动发现）+ environments/（7 种终端后端）
gateway/            # IM 网关（22 个 platforms/）
plugins/            # 8 类插件（memory、context_engine、model-providers、kanban、achievements、observability、image_gen、...）
skills/             # 内建 skills
optional-skills/    # 重型/小众 skills（默认不激活）
acp_adapter/        # ACP 服务端（VS Code / Zed / JetBrains 集成）
cron/               # croniter 调度器
ui-tui/             # Ink + React TUI（hermes --tui）
```

**调用链**（自下而上）：
```
tools/registry.py → tools/*.py → model_tools.py → run_agent.py / cli.py / batch_runner.py / environments/
```

**AIAgent 核心循环**（同步 + interrupt + budget + grace call）：
```python
while (api_call_count < self.max_iterations and budget.remaining > 0) or self._budget_grace_call:
    if self._interrupt_requested: break
    response = client.chat.completions.create(model=model, messages=messages, tools=tool_schemas)
    if response.tool_calls:
        for tc in response.tool_calls: handle_function_call(tc.name, tc.args)
```

**Slash Command Registry**（`hermes_cli/commands.py`）：所有命令定义为 `CommandDef` 列表；下游消费者（CLI / Gateway / Telegram BotCommand / Slack subcommand / autocomplete）**全部派生自同一 registry**——新增命令一处生效五处。

**plugin 系统**：v0.14 新增 `ctx.llm` 和 `tool_override`——plugin 可调用任何 LLM、可替换内建工具。`agent/__init__.py` 显示：`run_agent.py` 已拆分为多个自包含子模块，专注于 AIAgent 编排。

## 5. ACP 协议（acp_adapter）

**Agent Client Protocol (ACP)**：v0.14 标志性集成——Hermes 作为 ACP server，被 IDE 调用：

- **Zed ACP Registry 集成**：`uvx hermes-agent` 一键安装到 Zed；`hermes acp --setup-browser` 为 registry-driven 安装自动配 browser tools
- **入口**：`hermes-acp = "acp_adapter.entry:main"`（pyproject.toml scripts）
- **客户端覆盖**：VS Code / Zed / JetBrains（README 链接）
- **意义**：让 Hermes 不只是 CLI 工具，而是 IDE 级 Agent runtime——和 OpenAI 的 Codex CLI / Anthropic 的 Claude Code 在编辑器层面竞争

## 6. Skills 系统工作原理

skills 是 Hermes 区别于其他 agent 的核心，README 称之为"程序性记忆"（procedural memory）：

- **三个 skills 目录**：`skills/`（内建默认）、`optional-skills/`（重型默认不激活）、`~/.hermes/skills/openclaw-imports/`（从 OpenClaw 迁移）
- **agent-curated memory**：复杂任务后**自主创建** skills，使用时**自我改进** skills
- **Skills Hub**：通过 `huggingface/skills` 默认 tap 拉取社区 skills（v0.14 改动），无需配置
- **Slash 命令桥接**：`agent/skill_commands.py` 扫描 `~/.hermes/skills/`，**作为 user message 注入**（不污染 system prompt，保 prompt cache 命中）
- **agentskills.io 兼容**：开源标准
- **9 个新 optional skills**（v0.14）：Hyperliquid、Yahoo Finance、api-testing、unified EVM、darwinian-evolver、osint-investigation、pinggy-tunnel、watchers、Notion 2026 改版

## 7. 与 APEX 生态关系

| 维度 | Hermes Agent | APEX 生态（Xuanji） |
|------|-------------|---------------------|
| 定位 | 通用自进化 agent 框架 | 顶层 LLM 协调器 + 医疗科研 |
| 学习闭环 | skills 自创/自改 + Honcho 用户建模 | APEX 框架、APEX-MEM、Σ_memory |
| 跨会话记忆 | FTS5 session search + LLM summarization + Honcho dialectic | mem0（tokenBudget=4000，autoCapture/autoRecall） |
| 协议 | ACP + MCP | 内部协议 |
| Provider | 17+ LLM（多云、OAuth、本地） | 自有路由 + OpenAI 兼容 |
| 部署 | 7 种终端后端（local / Docker / SSH / Modal / Daytona / Singularity / Vercel） | 桌面 / 服务器 / QQ 机器人 |

**关键互补性**：
- Hermes 是**通用 agent 框架**（横向），APEX 是**领域协调器**（纵向）
- Hermes 的 skills 系统可承载 APEX 的"任务规划/自进化"模式
- Hermes 的 22 个 IM 平台覆盖度，可作为 APEX 输出端
- ACP 协议 + plugin 系统 + 跨会话 prompt cache，是 APEX 可学习的工程范式

## 8. v0.14 成熟度

**判定：生产可用 + 生态扩张阶段**。

**成熟度证据**：
- **PyPI 上架**（`pip install hermes-agent` 即可），从 git clone 时代毕业
- **Windows 原生支持**（虽标 early beta，但 40+ 平台修复已合入）
- **冷启动 < 1.5s**（v0.13 → v0.14 削减 19s，关键路径 `hermes tools` 从 14s 降至 < 1.5s）
- **1h 跨会话 Claude prompt cache**——成本/性能双优
- **180× 加速 browser_console**（持久化 CDP WebSocket）
- **LSP 语义诊断**——写入即校验，远超 Python/JSON/YAML/TOML 基础 lint
- **22 个 IM 平台、17+ LLM provider、7 种终端后端**——生态广度业内领先
- **215 社区贡献者、808 commits** 单版本——治理规模

**未成熟点**：
- Windows 仍标 "early beta"
- 13 个 RELEASE 跨 4 年（v0.2 在 2022，v0.14 在 2026）——版本号节奏偏慢
- `mistralai` extra 仍因 2026-05-12 蠕虫事件**全部移除**（PyPI 整包 404），需恢复时手动 un-quarantine

## 9. 核心创新点（3 个）

1. **Self-improving learning loop**——skills 从经验中自主创建、使用中自我改进、用户画像由 Honcho dialectic 建模。三者闭环（创 → 改 → 用 → 改）使 agent 越用越个性化，越用能力越强。这是 README 反复强调的 "the only agent with..."

2. **Lazy-install + 精确锁版本 + supply-chain advisory checker**——v0.14 重大安全架构升级。Mini Shai-Hulud 蠕虫事件后，把所有 optional backend 改成按需 pip install，`[all]` extra 只保留不能 lazy 的；`pyproject.toml` 注释详细记录了每个移除/恢复决策。比单纯 "锁版本" 更进一步：**约束爆炸半径**（blast radius reduction）。

3. **Slash Command Registry 派生架构**——一个 `CommandDef` 列表同时驱动 CLI / Gateway / Telegram BotCommand / Slack subcommand / autocomplete / `/help`。新增命令"一次定义、五处生效"，对 22 个 IM 平台的产品迭代速度是质变。同时 `cli_only` / `gateway_only` / `gateway_config_gate` 三态灵活控制可见性。

## 10. 风险/坑

- **安装风险**：`pyproject.toml` 注释明确写"v0.2–v0.14 之间累积的 extras 关系复杂"；Nix / Homebrew / Windows packager 必须精确读 `[all]` 规则，否则 lazy-deps 会触发首次失败
- **PyPI 蠕虫历史**：Mini Shai-Hulud 2026-05-12 攻击；`mistralai` 至今未恢复，README 文档已过时（README.md 第 95 行还把 mistral 列在支持 provider 中）
- **AC 文档割裂**：README 文档站 `hermes-agent.nousresearch.com/docs/` 与代码 `AGENTS.md` / 13 个 RELEASE 笔记的同步是手动维护的——例如 ACP 部分 README 未明确指出
- **测试规模 17k**：并行子进程 + 30s 超时是 escape valve，但 flaky test 治理成本高
- **scope 漂移**：单个 `run_agent.py` 12k LOC，`cli.py` 11k LOC——单文件复杂度警告已现；`agent/` 子包拆分到 v0.11 才完成，说明前 9 个版本都在"先跑通再重构"
- **AGENTS.md 53KB**：包含大量 load-bearing 注释，**subagent 读 200 行就够**——README 之外的实际开发信息在 AGENTS.md 但版本之间存在历史注释污染

---

**TL;DR**：Hermes Agent = OpenClaw 的自然继任者 + 自进化 skills 闭环 + 22 IM 平台广度。v0.14 "Foundation Release" 完成 PyPI 化 + lazy-install 安全加固。生产可用，Windows beta。APEX 生态可借鉴其 skills 自治 + 精确锁版本 + 跨会话 prompt cache 三件套。
