# OpenHuman v0.54.10 — 深度架构分析

> 分析对象：`/Users/lihongxin/Desktop/开智/openhuman/`（Fork 自 `tinyhumansai/openhuman`）
> 分析时点：2026-06-02
> 分析者：xuanji-apex sub-agent
> 实际文件数：1469 Rust + 597 TS + 472 TSX = **2538 源文件**（任务说 3404 包含 docs/scripts/tests/e2e）

---

## 1. 项目定位

### 一句话

**OpenHuman 是"个人 AI 超级智能"桌面代理** — 一款 Tauri + Rust + React 桌面 App，把"本地知识库 + 第三方集成 + 多 Agent 编排"整合到一个有"脸"（桌面吉祥物）的常驻 AI 助手里。

### 详细定位

- **形态**：桌面软件（Win/macOS/Linux），不是 CLI/SaaS。Tauri v2 + CEF（Chromium Embedded Framework）渲染。
- **定位**："Personal AI super intelligence: local memory, managed services where needed" — 本地数据主权，云端做模型路由、账户、Web Search 代理、Composio OAuth 接入。
- **差异化卖点**：
  - **Memory Tree**（本地 SQLite 层级摘要树）+ **Obsidian Vault**（兼容的 `.md` 文件）— 灵感来自 Karpathy 2026-03 obsidian-wiki 推文
  - **118+ 集成**（通过 Composio OAuth 一键接入），每 20 分钟 auto-fetch 拉新数据
  - **TokenJuice** 智能 Token 压缩（最多减 80%）
  - **16 个内置子 Agent**（Orchestrator / Researcher / Planner / Code Executor / Archivist / Critic / Help / Skill Creator / Crypto / Markets / MCP Setup / Morning Briefing / Trigger Reactor / Triage / Summarizer / Tool Maker / Integrations Agent）
  - **Meet Agent**（吉祥物可作为真实参会者加入 Google Meet）
  - **Native Voice**（whisper-rs STT + ElevenLabs TTS + 口型同步）
  - **WhatsApp Web 0.5**（替代 fork，0.2 版的 LID/skmsg 丢消息问题已修）
- **商业模式**：GNU 开源，**默认使用 OpenHuman 托管后端**（`api.tinyhumans.ai`），但可 BYO（自备 LLM key / 搜索 key / Composio key）。竞品对比表明确把 OpenHuman 和 Claude Cowork / OpenClaw / Hermes Agent 放在一起。

---

## 2. 技术栈全景

### 2.1 Rust 核心（root `Cargo.toml`，crate 名 `openhuman_core`）

**1 个 lib + 6 个 bin**：
- `openhuman-core`（主 daemon）— `src/main.rs`（234 行，初始化 Sentry + dotenv + 派发到 `cli::run_from_cli_args`）
- `slack-backfill`（`src/bin/slack_backfill.rs`）
- `gmail-backfill-3d`（`src/bin/gmail_backfill_3d.rs`）
- `memory-tree-init-smoke`（`src/bin/memory_tree_init_smoke.rs`）
- `inference-probe`（`src/bin/inference_probe.rs`）
- `test-mcp-stub`（`src/bin/test_mcp_stub.rs`）

**核心依赖族**（按域分组）：

| 域 | 关键 crates |
|---|---|
| **网络/异步** | `tokio` (full+sync), `reqwest` 0.12 (rustls+native-tls, socks, multipart, http2), `axum` 0.8, `tower`, `socketioxide` 0.15, `tokio-tungstenite` (Windows native-tls / Unix rustls) |
| **持久化** | `rusqlite` 0.37 (bundled, FTS5), `directories` 6, `chrono` |
| **加密/安全** | `aes-gcm` 0.10, `chacha20poly1305`, `argon2`, `ring` 0.17, `rustls` 0.23, `webpki-roots`, `keyring` 3 (apple/windows/linux native), `x25519-dalek` |
| **可观测** | `tracing` 0.1, `tracing-subscriber`, `opentelemetry` 0.32 (OTLP), `sentry` 0.47 (with TestTransport in dev), `prometheus` 0.14 |
| **LLM 推理** | `whisper-rs` 0.16 (Mac Metal + Windows MSVC fix fork), `cpal` 0.15, `hound`, `enigo`, `arboard`, `rdev` |
| **多链钱包** | `ethers-core/signers` 2.0, `bitcoin` 0.32 (P2WPKH PSBT), `ed25519-dalek`, `bs58`, `ripemd`, `coins-bip39` 0.8, `curve25519-dalek` |
| **平台 SDK** | `windows-sys` 0.61 (AppContainer/ACL/jail), `objc2` + `objc2-contacts` (macOS Contacts), `landlock` 0.4 (Linux sandbox, opt), `rppal` (RPi, opt) |
| **集成** | `matrix-sdk` 0.16 (E2E+rustls+markdown, opt), `whatsapp-rust` 0.5 + `wacore` (opt, 替换 wa-rs 0.2), `fantoccini` 0.22 (WebDriver, opt), `lettre` 0.11 (SMTP+rustls), `mail-parser` 0.11, `async-imap` 0.11 |
| **文本** | `regex` 1.10, `unicode-segmentation`（grapheme 安全！CJK/emoji）, `unicode-width`, `urlencoding`, `html2md` **已移除**（comment 详述 Ottern.ai 894MB 堆爆）|
| **CLI** | `clap` 4.5 (derive + complete), `dialoguer` 0.12 (fuzzy-select), `console` 0.16, `dotenvy` |
| **RPC/序列化** | `serde`, `serde_json`, `serde_yaml`, `toml`, `schemars` 1.2, `prost` 0.14, `postgres` 0.19 |
| **归档** | `tar` 0.4, `xz2` 0.1 (static), `zip` 2 (deflate), `flate2` |
| **其他** | `cpal`, `hound`, `enigo`, `arboard`, `rdev`, `sysinfo`, `starship-battery` 0.10 (替代废弃 `battery` crate), `fs2`, `tempfile`, `wait-timeout` |

**Feature flags**（精心设计）：
- `sandbox-landlock` / `sandbox-bubblewrap`（沙箱）
- `channel-matrix`（E2E Matrix）
- `peripheral-rpi`（RPi GPIO）
- `browser-native`（fantoccini WebDriver）
- `rag-pdf`（`pdf-extract`）
- `whatsapp-web`（whatsapp-rust）
- `e2e-test-support`（暴露破坏性 `openhuman.test_reset` RPC — **仅 E2E build 打开**）

**Patch**：`whisper-rs-sys` 用 fork 加 `static_crt(true)` 修 Windows MSVC LNK2038。

### 2.2 Tauri 桌面壳（`app/src-tauri/Cargo.toml`）

Tauri 2.10 (`cef` + `common-controls-v6` + `devtools` + `macos-private-api` + `tray-icon` + `unstable` + `webview-data-url` feature) + `tauri-plugin-deep-link` / `global-shortcut` / `notification` / `opener` / `single-instance` (with `deep-link`)。所有 tauri crate 用 `[patch.crates-io]` 锁到 `feat/cef` 分支，`cef-dll-sys` 首次构建自动下载 CEF 运行时（~300MB）。

### 2.3 TypeScript / React 前端

- **pnpm workspace**：`app/` (`openhuman-app` v0.53.45) + `packages/tauri-plugin-ptt/`（PTT 按键即说插件）。**不**包含 `packages/npm/`（其 postinstall 下载 v0.0.0 不存在，会让 CI 失败）
- **核心栈**：Vite + React + Redux Toolkit (`accounts/channelConnections/chatRuntime/coreMode/deepLinkAuth/mascot/notification/providerSurface/socket/thread` + `userScopedStorage`) + `redux-persist` + Tailwind（自定义 design tokens：ocean `#4A83DD` + sage/amber/coral + Inter/Cabinet Grotesk/JetBrains Mono）
- **i18n**：强制 `useT()` 包裹所有 UI 字符串（注释明确禁止硬编码）
- **MCP**：`app/src/lib/mcp/` — 但**只是元数据**，QuickJS 运行时已被移除（PR #1061 风格的重构），不再执行 skill 包
- **测试**：Vitest 1000+ 用例，ESLint+Prettier+Husky，pre-push 跑 `pnpm rust:check`，**合并门槛 80% diff coverage**（`diff-cover` 在 `coverage.yml`）

### 2.4 端到端服务

- **Vite dev server** :1420 + **核心 JSON-RPC server** :7788
- **后端**：`staging-api.tinyhumans.ai` / `api.tinyhumans.ai`（**远程**，无本地 backend）
- **认证**：core 启动时写 hex bearer 到 `${OPENHUMAN_WORKSPACE}/core.token`（`0o600`），Tauri `core_rpc_token` 命令把它给渲染层
- **进程模型**（**PR #1061 重构后**）：core **不再是 sidecar**，而是 Tauri 内的 tokio task，由 `core_process::CoreProcessHandle` 管理；前端仍走 HTTP `127.0.0.1:7788/rpc`

---

## 3. 核心模块拆解

### 3.1 顶层四模块

```
src/
├── lib.rs           # lib 入口，re-export 关键类型
├── main.rs          # openhuman-core bin 入口（Sentry+dotenv+CLI dispatch）
├── core/            # 【传输层】仅做协议/调度
│   ├── cli.rs       # 命令行解析+派发
│   ├── dispatch.rs  # JSON-RPC 路由器（旧方法名 → 规范名 → 业务域）
│   ├── jsonrpc.rs   # Axum 上的 JSON-RPC 1800 行（含 CORS/SSE）
│   ├── socketio.rs  # 实时事件流（930 行）
│   ├── auth.rs      # JWT/bearer 验证
│   ├── event_bus/   # 进程内 pub/sub + 强类型 request/response
│   ├── observability.rs  # Sentry before_send 过滤器 (3097 行！)
│   ├── rpc_log.rs   # 参数脱敏
│   ├── shutdown.rs  # 优雅停机
│   └── legacy_aliases.rs # 旧 RPC 名兼容
│
├── api/             # 【客户端】HTTP 调用后端的封装
│   ├── rest.rs      # 后端 REST（含错误分类）
│   ├── jwt.rs       # Bearer token 处理
│   ├── socket.rs    # 后端 WebSocket
│   └── config.rs / models/
│
├── rpc/             # 【路由层】传统 hand-rolled RPC 路由（逐步被 controller 模式取代）
│
└── openhuman/       # 【业务域】98 个子模块！
    ├── agent/         # 多 agent 编排核心（含 17 个 built-in sub-agent）
    ├── agent_experience/ # Hermes 风格程序化经验记忆
    ├── memory/        # 统一记忆抽象（trait + 多种 backend）
    ├── memory_tree/   # 新一代 Memory Tree（Karpathy-style）
    ├── channels/      # 消息渠道（20+ provider：telegram/discord/slack/email/matrix/signal/whatsapp/irc/lark/dingtalk/qq/mattermost/linq/imessage/web/...）
    ├── composio/      # 118+ 集成（OAuth + 工具执行）
    ├── inference/     # LLM 调用（多 provider + 本地 Ollama + 语音 + OpenAI OAuth）
    ├── embeddings/    # OpenAI/Ollama/noop 多 provider
    ├── skills/        # Skill 元数据（QuickJS 运行时已删）
    ├── integrations/  # brave/searxng/twilio/stock/tinyfish/apify/google_places
    ├── tools/         # 工具基类 + 注册表
    ├── tool_registry/ # 工具发现
    ├── tokenjuice/    # 工具输出压缩
    ├── security/      # prompt injection 防护
    ├── encryption/    # 本地加密
    ├── cost/          # 用量计费
    ├── cron/          # cron 调度
    ├── scheduler_gate/# 基于电量/活动的 throttle
    ├── heartbeat/     # 后台思考
    ├── subconscious/  # 潜意识（背景思考）
    ├── wallet/        # 多链钱包（ETH/BTC/SOL/Tron）
    ├── meet/ + meet_agent/  # Google Meet 集成
    ├── audio_toolkit/ # 音频 I/O
    ├── voice/         # TTS 编排
    ├── service/       # daemon 启动编排
    ├── desktop_companion/ # 桌面吉祥物
    ├── devices/       # 设备发现
    ├── tls/, cwd_jail/ # 平台沙箱
    └── ... 50+ 其他
```

### 3.2 五个 bin 各自做什么

| Bin | 行数估算 | 用途 |
|---|---|---|
| **`gmail-backfill-3d`** | ~150 | 走 Composio OAuth 拉过去 N 天 Gmail 邮件 → `EmailThread` → `ingest_page_into_memory_tree` 写 `.md` + SQLite → drain worker pool → SHA-256 校验文件完整性。支持 `--days`/`--page-size`/`--wipe`/`--skip-verify` |
| **`slack-backfill`** | ~120 | 走 Composio OAuth 调 `SlackProvider::sync()` 一次性同步活跃 Slack 连接。**和 15 分钟 cron 走同一路径**，便于操作员先看一次再信任调度。需配 Ollama 端点（embed/extract/summarise） |
| **`memory-tree-init-smoke`** | ~80 | 压测 SQLite schema init 竞态：N 线程并发调 `memory::tree::store::with_connection`。**修前**会出现 SQLITE_CANTOPEN/IOERR_TRUNCATE/IOERR_SHMMAP，**修后** mutex-gated init 全部 Ok |
| **`inference-probe`** | ~100 | 直接调 orchestrator harness 跑单轮（`--mode harness`）或手搓 request 直发 provider（`--mode raw --raw-mode pformat`），用于验证 tool call 是否真的 fire |
| **`test-mcp-stub`** | ~60 | 极简 MCP stdio server（仅 `initialize`/`tools/list`/`tools/call` + `echo` 玩具工具），供 `tests/mcp_registry_e2e.rs` 使用，**零外部依赖**以保 CI 速度 |

**所有 5 个 bin 都设计成"运维可独立调用的诊断/迁移/压测工具"** —— 这是非常成熟的工程化做法。

### 3.3 `core` vs `api` vs `rpc` vs `openhuman` 分工

- `core/`：**只**做传输（HTTP/WS/JSON-RPC/CLI/调度/认证/可观测性/事件总线）。AGENTS.md 明确 "No heavy business logic here"。
- `api/`：**只**做"调用后端"（HTTP client），是 core → 后端的胶水。
- `rpc/`：**只**做"老式手工路由"（每个方法一个 match arm），正在被 controller 模式**逐步替代**（AGENTS.md 列出 "Controller migration checklist"）。
- `openhuman/`：**所有业务域**。每个域按统一模板组织：
  - `mod.rs`（轻量、re-export）
  - `schemas.rs`（`ControllerSchema` 元数据）
  - `rpc.rs`（域 RPC handler）
  - `ops.rs` / `store.rs` / `bus.rs` / `types.rs` / `tools.rs`（**unix 风格单职责**）
  - 测试以 `*_tests.rs` / `*_test.rs` 与实现 sibling 存在

**关键架构约定**：
- **Controller registry 模式**：`schemas.rs` 暴露 `all_controller_schemas()` / `all_registered_controllers()`，在 `src/core/all.rs` 统一注册，**禁止**在 `cli.rs`/`jsonrpc.rs` 加业务分支
- **Event bus**（`core/event_bus/`）：双 API —— `publish_global` 广播 + `request_native_global` 强类型 R/R（**零序列化**，能传 `Arc<dyn Provider>`/`mpsc::Sender<T>`/`oneshot::Sender`）。**单例**用 module-level fn
- **Legacy aliases**：`src/core/legacy_aliases::resolve_legacy` 在 dispatch 入口改写旧方法名，与前端 `app/src/services/rpcMethods.ts::normalizeRpcMethod` **对称**

---

## 4. "Personal AI super intelligence" 怎么实现

### 4.1 "Local memory" — 数据主权在端侧

**两层记忆架构**（并存，逐步迁移到新层）：

#### 旧层 `openhuman/memory/`
- 统一 trait `Memory`（15+ 实现）
- 子模块：`chunker / conversations / global / ingestion (parse/queue/regex/rules/state) / ops / preferences / safety / schemas / stm_recall / store / sync_status / tool_memory / traits`
- 多种 backend：本地 SQLite、远程 REST、`agentmemory`（rohitg00/agentmemory 兼容 backend）
- 支持 vector search + keyword (FTS5) + relational

#### 新层 `openhuman/memory_tree/`（**这是创新重点**）
- **Karpathy obsidian-wiki 思路**实现：每条数据 → canonical Markdown chunk（≤3k token）→ 打分 → 折叠到层级摘要树 → SQLite
- **子模块**：
  - `canonicalize/{chat, document, email, email_clean}.rs`（邮件专门写了 `email_clean.rs` 剥离 reply chains / marketing footer / 法律免责声明）
  - `chunker.rs`、`content_store/`（**写两份**：SQLite + 实际 `.md` 文件，**带 SHA-256 校验**）
  - `ingest.rs`（723 行，Phase 1）
  - `score/`（432 行主 + 265 行 resolver + 464 行 store + 233 行 embed sub）
  - `summarizer/`（异步 worker）
  - `retrieval/`（728 行 `benchmarks.rs` + 591 行 `drill_down.rs` + 689 行 `source.rs` + 628 行 `topic.rs` + 621 行 `rpc.rs`）—— 6 个 LLM-callable retrieval 工具
  - `tree_global`、`tree_source`、`tree_topic`（三个独立树）
  - `jobs/`（drain worker pool 直到 idle）
  - `tools/`（retrieve/extract/score/summarize）
- **关键文件 `.md`**：数据**双写**到 `~/.openhuman/vault/` 的 Obsidian 兼容 vault，用户可手动编辑 → 重新被 ingest

**`agent_experience/`（Hermes-style 经验学习）**：
- 374 行 `capture.rs` 钩子在 agent loop 结束后自动抓取 tool sequence / outcome
- 415 行 `store.rs` 用 namespace `agent_experience` 写进统一 memory
- 187 行 `prompt.rs` 在下轮 prompt 顶部 `prepend_experience_block`，agent 看见自己过去的 "lesson learned"
- 这是 README 表格里 Hermes Agent "✅ Self-learning" 列的同款能力

### 4.2 "Managed services where needed" — 哪些走云

**OpenHuman 默认托管后端负责**：
- **账户登录**（OAuth flow）
- **LLM 模型路由**（一个订阅，所有模型；reasoning/fast/vision 角色由后端选）
- **Web Search 代理**（Brave / SearXNG 由后端代理，避免暴露 key）
- **Composio 集成 OAuth + 工具调用**（118+ 集成全部走 managed OAuth）

**完全本地的能力**（BYOK 模式）：
- 可用 `OLLAMA` endpoint 跑本地 embed / extract / summarise
- 可配 `OPENHUMAN_MEMORY_EMBED_*` 等 env 全部走本地
- 可配自备 `OPENHUMAN_API_URL` 指向自托管 backend
- `fantoccini` 浏览器自动化（feature 开启）— 自管 WebDriver

**Webhook/实时触发器**：`webhooks/bus.rs` 把外部 HTTP 推到本地 skills 模块，**需要自己 hosted & wired**（README 明确警告）

### 4.3 "Simple and powerful" — UI 优先体验

- 一键安装脚本（`install.sh`/`install.ps1`）
- **桌面吉祥物**（`desktop_companion` + `mascot_native_window`）：说话、对环境反应、加入 Google Meet
- **20 分钟 auto-fetch** 循环：`core/heartbeat/` + `scheduler_gate/`（基于电池电量 throttle 后台 LLM 工作 — `starship-battery` crate）
- **Native voice**：whisper-rs STT → ElevenLabs TTS → lip-sync → live Google Meet agent

### 4.4 多 Agent 编排（核心引擎）

`openhuman/agent/agents/` 下 17 个 built-in sub-agent（每个 = `agent.toml` + `prompt.rs` + `prompt.md` + `mod.rs`）：

| Agent | Tier | 角色 |
|---|---|---|
| **orchestrator** | chat | 路由+判断+合成（"Staff Engineer"），最高 15 iter，**subagents 列表在 TOML** 动态展开为 `delegate_*` 工具 |
| **researcher** | reasoning | 长程深度研究 |
| **planner** | reasoning | 计划生成 |
| **code_executor** | worker | 写代码+执行 |
| **archivist** | background | 后台图书管理员，提取 lesson → 写 MEMORY.md → FTS5 索引（cheap & slow，max 3 iter）|
| **critic** | worker | 质量审视 |
| **help** | worker | 帮助 |
| **integrations_agent** | worker | Composio 集成调用 |
| **mcp_setup** | worker | MCP 配置 |
| **morning_briefing** | background | 早间简报 |
| **summarizer** | worker | 摘要 |
| **skill_creator** | worker | 创建新 skill |
| **tool_maker** | worker | 制造新工具 |
| **tools_agent** | worker | 工具发现 |
| **crypto_agent** / **markets_agent** | worker | 链上+市场数据 |
| **trigger_reactor** / **trigger_triage** | worker | 触发器分类响应 |

**关键创新**：**`agent.toml` 是数据而非代码** — `loader.rs` 用静态 slice 注册，新增 built-in agent **只需创建一个子目录 + 加一行到 `BUILTINS`**，无 match arm、无 enum variant。

**Dispatcher 抽象**（`agent/dispatcher.rs`，518 行）：`ToolDispatcher` trait 屏蔽不同 LLM 的工具调用方言（XML / JSON / P-Format），agent loop 与具体 provider 解耦。

**Triage 管线**（`agent/triage/`）：用本地小模型**快速分类外部触发器**（webhook/cron），决定是否升级到 reasoning 模型或直接回复。

---

## 5. 架构图（数据流）

```
┌────────────────────────────────────────────────────────────────────┐
│                    ┌────────────────────────────┐                  │
│                    │   Tauri 2.10 + CEF Shell    │                  │
│                    │   (app/src-tauri)           │                  │
│                    │                             │                  │
│                    │  ┌──────────────────────┐   │                  │
│                    │  │  React + Redux + RTK │   │                  │
│                    │  │  (app/src)           │   │                  │
│                    │  │  HashRouter          │   │                  │
│                    │  │  /home /chat /...    │   │                  │
│                    │  │  coreRpcClient       │   │                  │
│                    │  │  socketService       │   │                  │
│                    │  └──────────┬───────────┘   │                  │
│                    └─────────────┼───────────────┘                  │
│                                  │  invoke('core_rpc_relay', ...) │
│                                  │  WebSocket (Socket.io)          │
│                                  ▼                                  │
│         ┌────────────────────────────────────────┐                 │
│         │  core_process::CoreProcessHandle       │                 │
│         │  (Tauri-internal tokio task)           │                 │
│         └──────────────┬─────────────────────────┘                 │
│                        │  http://127.0.0.1:7788/rpc                │
│                        │  Bearer: ${OPENHUMAN_WORKSPACE}/core.token│
│                        ▼                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │           Rust Core (openhuman_core) — src/main.rs           │  │
│  │                                                              │  │
│  │  ┌────────────────────────────────────────────────────────┐  │  │
│  │  │ core/  (Transport only, no business logic)             │  │  │
│  │  │  cli → dispatch (legacy aliases resolve) → all.rs      │  │  │
│  │  │  jsonrpc.rs (Axum + CORS, 1800L)                       │  │  │
│  │  │  socketio.rs (Socket.io 930L)                          │  │  │
│  │  │  event_bus/ (pub/sub + typed R/R singletons)           │  │  │
│  │  │  observability.rs (Sentry before_send, 3097L)          │  │  │
│  │  └─────────────────┬──────────────────────────────────────┘  │  │
│  │                    │                                         │  │
│  │                    ▼ controller registry (src/core/all.rs)   │  │
│  │  ┌────────────────────────────────────────────────────────┐  │  │
│  │  │ openhuman/  (98 domains, each with schemas/rpc/ops)    │  │  │
│  │  │                                                        │  │  │
│  │  │  ┌─ agent ──────────────────────────────────────────┐  │  │  │
│  │  │  │  harness/ session::Agent run_single turn loop    │  │  │  │
│  │  │  │  agents/ 17 built-ins (orchestrator + 16 subs)   │  │  │  │
│  │  │  │  dispatcher ToolDispatcher trait (XML/JSON/P)     │  │  │  │
│  │  │  │  triage/ cheap local-model classifier             │  │  │  │
│  │  │  │  task_board/ parallel subagent coordination       │  │  │  │
│  │  │  │  memory_loader/ tree_loader/ profiles/ prompts/   │  │  │  │
│  │  │  └──────────────────────────────────────────────────┘  │  │  │
│  │  │  ┌─ memory_tree (Karpathy-style) ────────────────────┐  │  │  │
│  │  │  │  canonicalize → chunker → content_store (md+SQLite)│  │  │  │
│  │  │  │  score/ summarizer/ → tree_global/tree_source/     │  │  │  │
│  │  │  │       tree_topic/                                   │  │  │  │
│  │  │  │  retrieval/ 6 LLM-callable tools (drill_down etc) │  │  │  │
│  │  │  └──────────────────────────────────────────────────┘  │  │  │
│  │  │  ┌─ channels/ (20+ providers) ───────────────────────┐  │  │  │
│  │  │  │  telegram/discord/slack/email/matrix/signal/whatsapp│ │  │  │
│  │  │  │  imessage/irc/lark/dingtalk/qq/mattermost/linq/web  │  │  │  │
│  │  │  └──────────────────────────────────────────────────┘  │  │  │
│  │  │  ┌─ composio/ (118+ integrations) ───────────────────┐  │  │  │
│  │  │  │  providers/{gmail,slack,notion,github,stripe,..}    │  │  │  │
│  │  │  │  client + execute_dispatch + execute_prepare        │  │  │  │
│  │  │  └──────────────────────────────────────────────────┘  │  │  │
│  │  │  ┌─ inference/ (LLM + voice) ────────────────────────┐  │  │  │
│  │  │  │  provider/{openai,anthropic,google,ollama,...}     │  │  │  │
│  │  │  │  voice/ + openai_oauth/ + local/ + http/           │  │  │  │
│  │  │  └──────────────────────────────────────────────────┘  │  │  │
│  │  │  ┌─ tokenjuice/ (tool output compression) ───────────┐  │  │  │
│  │  │  │  reduce.rs (928L) + classify.rs + rules/         │  │  │  │
│  │  │  └──────────────────────────────────────────────────┘  │  │  │
│  │  │  ... 90+ other domains ...                              │  │  │
│  │  └────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                        │                                          │
│                        │  outbound HTTP/WebSocket                 │
│                        ▼                                          │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  api.tinyhumans.ai / staging-api.tinyhumans.ai (REMOTE)   │    │
│  │  - Auth (JWT)                                              │    │
│  │  - LLM routing (one sub, all models)                       │    │
│  │  - Web search proxy (Brave/SearXNG)                        │    │
│  │  - Composio OAuth + 118+ integrations                      │    │
│  │  - Webhook ingress (real-time triggers)                    │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                    │
│  ┌──── Bin 工具 (可独立调) ───────────────────────────────────┐   │
│  │ gmail-backfill-3d  slack-backfill  memory-tree-init-smoke  │   │
│  │ inference-probe    test-mcp-stub                            │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                    │
│  ┌──── 本地资源 ──────────────────────────────────────────────┐   │
│  │  ~/.openhuman/                                              │   │
│  │    ├── core.token (0600)                                    │   │
│  │    ├── auth-profiles.json (Composio JWT)                    │   │
│  │    ├── config.toml                                          │   │
│  │    ├── workspace.db (rusqlite, FTS5)                        │   │
│  │    ├── vault/*.md (Obsidian 兼容 vault)                     │   │
│  │    ├── node-runtime/ (按需下载的 Node.js tar.xz/.zip)       │   │
│  │    └── agents/*.toml (workspace 覆盖 built-in)              │   │
│  └────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────┘
```

---

## 6. 与 APEX 生态的关系

用户已有 **APEX-AGI 主控台**（`apex_token_optimizer` + `apex_memory_bridge` + Σ_memory + SWRs RingBuffer + Gini 路径选择 + X_real 坐标校正等）。**openhuman 在这个生态的哪个层？**

### 层级定位

| 层 | 项目 | 职责 |
|---|---|---|
| **L0 — 基础模型** | 各种 LLM（OAI/Anthropic/Ollama） | 推理能力 |
| **L1 — 代理运行时** | **OpenHuman** | 多 agent 编排 + 工具执行 + 状态管理 + 集成 |
| **L2 — 协调主控** | **APEX-AGI** | 跨代理/跨任务调度 + Σ 自我迭代 + 路径选择 |
| **L3 — 用户层** | Tauri 桌面 App / Web Chat | UI/UX |

**OpenHuman 是 APEX 调度 LLM 行为的"代理运行时"层**：当 APEX 决定 "下一步让某个 agent 用工具 X 干 Y" 时，OpenHuman 是那个接收指令、执行工具调用、回报结构化结果、并维护 memory 的执行体。

### 互补关系（不是替代）

| 能力 | OpenHuman | APEX-AGI |
|---|---|---|
| LLM 调度 | ✅ 17 内置 agent + dispatcher | ❌ 不直接调 LLM |
| 工具执行 | ✅ 完整 tool loop + registry | ❌ 工具元数据 |
| 记忆 | ✅ Memory Tree + Obsidian vault | ✅ Mem0 long-term + RingBuffer |
| 多源集成 | ✅ 118+ 集成 | ❌ 无 |
| 路径优化 | ❌ 无 | ✅ Gini 选择 + ΔG 公式 |
| 自我迭代 | ❌ 无 | ✅ APEX TOKEN_OPTIMIZATION + 25-step purification |
| 跨代理协调 | ❌ 单进程 | ✅ 主控台 |

### 可能的整合点

1. **APEX 作为"上层 meta-agent"** 通过 OpenHuman 的 `delegate_*` 工具（`agent/agents/loader.rs` 的 `collect_orchestrator_tools` 动态展开）路由任务
2. **OpenHuman 的 memory_loader**（782 行）可以把 APEX 的 Σ_memory 注入到 agent 上下文
3. **APEX 的 Gini 路径选择**可以在 OpenHuman 的 dispatcher 选择前做一次"该用哪个 agent / 哪个 provider"预判
4. **OpenHuman 的 tokenjuice/reduce.rs**（928 行）**非常值得 APEX 借鉴** — 工具输出压缩可作为 APEX 的 `Effort_valid` 计算基础

### 冲突/风险

- **APEX 已有 Mem0**（`memory_add`/`memory_search`），OpenHuman 也有自己的 `memory` 抽象。**双记忆库**会出现同步问题，需要选定一个作为 source of truth
- **OpenHuman 默认走 `api.tinyhumans.ai`**，APEX 主控台可能希望统一路由到自托管 LLM（必须 BYO `OPENHUMAN_API_URL`）
- **OpenHuman 的 17 个 built-in agent 是"硬编码"的 TOML**，APEX 若想动态注入 sub-agent 需要走 `$OPENHUMAN_WORKSPACE/agents/*.toml` workspace 覆盖路径

---

## 7. v0.54.10 成熟度评估

### 7.1 整体状态

- **README 徽章**：`status-early beta-orange`（early beta，主动开发中，"expect rough edges"）
- **README 中英文版本同步维护**（英/简中/日/韩/德 5 语言 README）
- **iOS/Android Tauri 移动端存在**（`app/src-tauri-mobile/`、scripts 里有 `tauri:ios:*` / `tauri:android:*`） — AGENTS.md 警告 "Tauri host is desktop-only. Do not add Android/iOS branches"（**与移动脚本存在**矛盾，可能是渐进式迁移中）
- **5800+ Rust tests + 1000+ Vitest**（从 CLAUDE.md 提到）
- **80% diff coverage 合并门槛**（在 `coverage.yml` 强制）
- **2026-04-22 内部分析**（claude-mem context 引用）显示项目仍在高频迭代：refactor memory namespace（trait + 15 impls）、webhooks ingress、config runtime dir 等多个 draft PR 同时进行

### 7.2 核心 vs 实验区分

**核心稳态（生产可用）**：
- 桌面 app 启动 + Tauri 壳 + CEF 渲染
- in-process core（PR #1061 后的新架构）
- JSON-RPC + Socket.io 传输
- Composio OAuth + 118+ 集成
- Memory Tree 写入路径（chunker / canonicalize / content_store / SQLite）
- 17 个 built-in agent + dispatcher 抽象
- 5 个 bin 工具
- tokenjuice/reduce（929 行 + 487 行 classify + 规则文件）
- Sentry + OTel + Prometheus 可观测栈
- 多链钱包（ETH/BTC/SOL/Tron）
- whisper-rs STT + ElevenLabs TTS

**实验/重构中（不稳定）**：
- **QuickJS 技能运行时已删**（AGENTS.md 反复强调） — `src/openhuman/skills/` 现在**只**有元数据（`ops_create/ops_discover/ops_install/ops_parse/inject/schemas/types`），不再执行 skill 包
- **Memory 抽象重构**（claude-mem 提到 "Memory Refactor: Trait Shape, L1 Pointer"）— 旧 `memory/` 与新 `memory_tree/` 并存，15+ impls 的 trait 还没稳定
- **Webhooks ingress**（claude-mem 提到 spec 在写）— 实时 webhook 入口在迭代
- **Memory 命名空间**（"3 separate auto-inject implementations" 提示历史债务）
- **iOS/Android 移动端**（脚本存在但 desktop-only 限制相互矛盾）

**开发体验半成品**：
- 内存警告 `#![allow(dead_code)]` 在 `openhuman/mod.rs`（"Many types/functions are intended for future use or integration with the frontend"）
- 大量域还在加 controller schemas（旧 `src/rpc/dispatch.rs` match arms 迁移中）
- `eval infra does not exist`（claude-mem 明确指出）

### 7.3 已知坑（来自 AGENTS.md / Cargo.toml 注释）

- **Linux Wayland** 下 AppImage 启动崩溃（#2463）
- **Arch Linux `sharun: Interpreter not found!`**（pacman 包变通）
- **whisper-rs-sys Windows MSVC LNK2038**（已 patch fork）
- **Sentry OPENHUMAN-TAURI-2E / -84 / -T** 等大量 transient 事件洪水（已用多层 before_send 过滤器）
- **OPENHUMAN-TAURI-A** 第二次启动 CEF init cache-lock panic（用 `tauri-plugin-single-instance` 解）
- **OPENHUMAN-TAURI-R7** PATCH/DELETE 404（防御性 filter）
- **SSL/TLS backend** Windows native-tls vs Unix rustls 的差异（`tokio-tungstenite` 按 target 条件依赖）
- **macOS Tahoe (Apple Silicon)** `whisper-rs` / `llama.cpp` `-mcpu=native` 失败（workaround `GGML_NATIVE=OFF`）
- **html2md 894MB 堆爆**（已移除，改用 `fast_html_to_text` + 优先 MIME text/plain）

---

## 8. 代码质量

### 8.1 规模

- **1469 .rs + 597 .ts + 472 .tsx = 2538 源文件**（与 README 文档化的项目"3404 文件"差距 ≈ 866 是 docs / scripts / tests / e2e / examples / gitbooks / design-previews / assets / Remotion 视频项目）
- 顶级 `openhuman/` 域 = 98 个子模块
- 单文件行数普遍 < 500（AGENTS.md 明确 "Prefer ≤ ~500 lines"），最长是 `core/observability.rs` (3097L)、`core/jsonrpc.rs` (1800L)、`core/jsonrpc_tests.rs` (1163L)、`core/socketio.rs` (930L)、`tokenjuice/reduce.rs` (928L)、`agent_experience/capture.rs` (374L)

### 8.2 模块化

**优点**：
- **Unix 风格单职责**（AGENTS.md 哲学 "each should do one thing really well"）
- **域子目录模板**：`mod.rs` 轻量 + 导出 + `schemas.rs` 注册 + `rpc.rs` 路由 + `ops.rs`/`store.rs`/`bus.rs`/`types.rs` 分散职责
- **新功能必须放子目录**（禁止在 `src/openhuman/` 根加 `.rs`）
- **测试与实现 sibling**（`*_tests.rs` / `*_test.rs`，AGENTS.md 给出统一模式 `#[cfg(test)] #[path = "..._test.rs"] mod tests;`）
- **Controller registry 模式**取代手工 match arm 路由
- **Built-in agent 是 TOML 数据**而非 Rust 代码
- **Strong typing**：`RpcOutcome<T>`、`ControllerSchema`、`DomainEvent`（`#[non_exhaustive]`）

**可改进**：
- `openhuman/mod.rs` 顶部 `#![allow(dead_code)]` —— 大量未使用代码
- `observability.rs` 3097 行是技术债（Sentry 过滤规则堆积）
- `jsonrpc.rs` 1800 行 + `socketio.rs` 930 行 — 传输层可拆更细
- `embeddings/`、`inference/` 下子目录命名略不一致（`http/`、`local/`、`openai_oauth/`、`provider/`、`voice/`）
- `channels/providers/{telegram,discord,whatsapp,whatsapp_web}.rs` 与 `channels/providers/{telegram,discord,whatsapp,whatsapp_web}_tests.rs` 混排 — 测试散在 `providers/` 根，平铺不够

### 8.3 测试

- **Rust**：5800+ tests（CLAUDE.md 提到），213 个 `*_tests.rs`/`*_test.rs` 文件（实测 `find`）
- **TS/TSX**：343 个 `*.test.ts(x)` 文件
- **E2E**：WDIO + Appium Mac2 + tauri-driver（Linux CI），独立 mock backend
- **Coverage**：80% diff coverage 强制门槛（`diff-cover` + `cargo-llvm-cov` + Vitest lcov）
- **Debug runners**：`scripts/debug/{cli,unit,e2e,rust,logs,lib}.sh` 把 stdout 限流 + tee 完整日志到 `target/debug-logs/`
- **Pre-push hook**：跑 `pnpm rust:check`（Husky）

### 8.4 可观测性

- **3 个 trace 系统并行**：Sentry（错误聚合，含 `TestTransport` for dev）、OpenTelemetry OTLP（trace+metrics，http-proto）、Prometheus（指标）
- **Structured logging**：`tracing` + `tracing-subscriber` + `tracing-appender`（env-filter, ansi, fmt）
- **Domain event bus** 内置 `TracingSubscriber`（`event_bus/tracing.rs`）
- **Sentry before_send 6 层防御性过滤**（`main.rs` 注释详尽解释每个 issue ID）

### 8.5 安全

- **JWT/bearer** 认证（`core/auth.rs` + `api/jwt.rs`）
- **进程内 token 文件** `core.token` mode 0600
- **Keyring**（apple/windows/linux native）存密码
- **AES-GCM + ChaCha20-Poly1305 + X25519** 本地加密
- **Argon2** 派生
- **prompt_injection/** 防护域
- **Cwd jail**（Windows AppContainer / Linux Landlock / Bubblewrap，feature flag）
- **Prompt scrubbing**（Sentry before_send + `core/rpc_log::redact_params_for_log`）
- **CORS 严格控制**（`OPENHUMAN_CORE_ALLOWED_ORIGINS`）

---

## 9. 核心创新点（3 个最值得借鉴）

### 9.1 ⭐ Memory Tree + Obsidian Vault 双写（**最高价值**）

**位置**：`src/openhuman/memory_tree/`（约 6000+ 行）

**创新点**：
- **数据双写**到 SQLite（带 provenance + content_sha256）+ 实际 `.md` 文件（Obsidian 兼容）—— 用户既能用 SQL 检索也能用 Obsidian 浏览编辑
- **Karpathy obsidian-wiki 思路**的 Rust 工业化实现：canonicalize (chat/email/document) → chunker (≤3k token, 稳定 ID) → score (multi-signal) → summarizer (异步 worker pool) → tree_global/tree_source/tree_topic 三棵树
- **6 个 LLM-callable retrieval 工具**（`drill_down`、`fetch_leaves`、`query_source`/`query_global`/`query_topic`/`search_entities`）—— agent 可以**层级下钻**而非平铺检索
- **SHA-256 内容校验**（`content_store::read::verify_chunk_file`）—— 数据完整性可验证
- **Backfill binary**（`gmail-backfill-3d`）做端到端压力测试

**借鉴价值**：完全可以抽出来作为独立 crate 给 APEX 生态用，APEX 的 RingBuffer 是"热数据层"，Memory Tree 是"温+冷数据层"。

### 9.2 ⭐ Built-in Agent 即数据（agent.toml 模式）

**位置**：`src/openhuman/agent/agents/loader.rs` + `agent/agents/{orchestrator,...}/agent.toml` + `prompt.rs`

**创新点**：
- 新增 sub-agent = 创建子目录 + 2 个文件 + `BUILTINS` 加一行，**无 match arm、无 enum 变体**
- `agent.toml` 描述：id/when_to_use/temperature/max_iterations/sandbox_mode/agent_tier/omit_*/tools/subagents
- `prompt.rs` 暴露 `fn build(ctx: &PromptContext) -> Result<String>`，**prompt 模板**可基于运行时状态（用户 profile、连接的 integrations、模型 hint）动态生成
- **Loader 强制约束**："a `chat` agent must NOT list any other `chat` agent in `subagents`" — 编排层级由 loader 静态校验
- `collect_orchestrator_tools` 把 `subagents = [...]` 动态展开为 N 个 `delegate_*` 工具（每个 sub-agent 一个），LLM 看见 first-class function-calling schema
- **Workspace override**：`$OPENHUMAN_WORKSPACE/agents/*.toml` 可覆盖 built-in（按 id collision 替换）

**借鉴价值**：APEX 的 ΔG 公式和 Gini 路径选择需要一个"动态可插拔 sub-agent"层，**完全套用这个模式**就能把 17 个 sub-agent 替换为 APEX 自己的"演化候选策略"。

### 9.3 ⭐ TokenJuice 工具输出压缩（独立 crate 化潜力）

**位置**：`src/openhuman/tokenjuice/`（reduce.rs 928L + classify.rs 487L + types.rs 318L + rules/）

**创新点**：
- **Rust port** of [`vincentkoc/tokenjuice`](https://github.com/vincentkoc/tokenjuice)
- **JSON-configured rules**：把 `git status` 噪音、npm install warnings、cargo warnings、docker pull 进度条等**可预测格式**压缩成 1-2 行
- **支持任意 tool_name + argv**（识别 `git`/`npm`/`cargo`/`docker`/`pnpm`/...）
- `ToolExecutionInput` → `ReduceResult.inline_text`（928 行 reduce 引擎 + 487 行 classifier）
- **规则可热加载**（`rules::load_builtin_rules` + 用户自定义 overlay）
- **CJK/emoji grapheme 安全**（用 `unicode-segmentation` 显式处理多字节）

**借鉴价值**：APEX 已经在做 token 优化（`Effort_valid = Total - Waste`），但主要在 LLM 输入侧。**TokenJuice 是工具输出侧**（被 `core_call` 收口的 stderr/stdout），**APEX 直接复用这个 crate 就能把"LLM 喂的工具输出"维度也加进 Waste 计算**。

### 9.4 备选：双 EventBus（pub/sub + 强类型 R/R）

**位置**：`src/core/event_bus/{bus.rs, native_request.rs}`

**创新点**：同一个 singleton 暴露两套 API —— `publish_global` 广播 + `request_native_global` 强类型 R/R（**零序列化**，能传 `Arc<dyn Provider>`/`mpsc::Sender<T>`/`oneshot::Sender`），让"模块间调用"和"JSON-RPC 外部调用"**统一方法名 convention**（`"<domain>.<verb>"`）但保持**内部不序列化**的零开销。

**借鉴价值**：APEX 多 agent 协调可能需要类似"内部 R/R 零开销"通道（目前 Mem0 调用是序列化的）。

---

## 10. 风险 / 坑

### 10.1 工程规模风险

1. **2538 源文件 + 5800+ Rust tests** 是个**巨型 monorepo** —— 单 `cargo check` 慢，CI 资源消耗大（已配 `[profile.ci]` 加速：opt-level=1, codegen-units=16, lto=false）
2. **98 个 `openhuman/` 域**对新贡献者**学习曲线陡峭**（AGENTS.md 写 650 行只为讲清楚约定）
3. **#![allow(dead_code)]** 表明有**大量未使用代码**，表面 API 稳定性差

### 10.2 架构债务

1. **`memory/` 与 `memory_tree/` 并存**：双套抽象、3 个 auto-inject 站点不一致（claude-mem 提到正在 refactor）
2. **`src/rpc/dispatch.rs` 与 `src/core/dispatch.rs` 双套** 路由（旧手工 vs 新 controller registry），AGENTS.md 列出 migration checklist 但**未完成**
3. **`observability.rs` 3097 行**：Sentry before_send 规则堆积，每个 issue ID 一个 if 分支
4. **`jsonrpc.rs` 1800 + `socketio.rs` 930**：传输层可拆
5. **Skills runtime 已删但元数据还在**（`src/openhuman/skills/` 现为 7 个 metadata-only 文件，注释里反复解释"Legacy retained after QuickJS runtime removal"），后续要么复活要么彻底拆掉
6. **iOS/Android 移动端**（`app/src-tauri-mobile/`、pnpm scripts 有 `tauri:ios:*` / `tauri:android:*`）vs AGENTS.md 写 "Tauri host is desktop-only. Do not add Android/iOS branches" — **自相矛盾**

### 10.3 业务/产品风险

1. **强烈依赖远程后端**（`api.tinyhumans.ai`）—— README 明确 "real-time triggers and hosted features still require the managed backend"。**完全自托管**有盲点（webhook ingress）
2. **WhatsApp 0.5 是新依赖**（替代 wa-rs 0.2 fork），生产稳定性待验证
3. **17 个 sub-agent 的 TOML 是硬编码**，新增 sub-agent 要走 PR 流程，**用户/开发者难动态扩展**
4. **测试 80% 门槛**严苛但执行良好；claude-mem 提到 "eval infra does not exist" —— 缺端到端 agent 效果评估
5. **多链钱包 + Webhook + Meet Agent + Voice** 等大型 feature 同时存在，**质量长尾**风险

### 10.4 集成到 APEX 的具体风险

1. **双记忆库**：Mem0 + OpenHuman Memory Tree 需要选定一个 source of truth（建议 OpenHuman 作数据层，APEX 通过 `memory_loader` 读取）
2. **APEX 现有 LLM 调用** vs **OpenHuman 的 model routing** — 路由冲突，需要 `OPENHUMAN_API_URL` 指向 APEX 自己的代理
3. **OpenHuman 的 20 分钟 auto-fetch loop** 会**持续**拉数据 + 持续 LLM 调用，**APEX 必须有后台 LLM 用量监控**（`cost/` 域可用）
4. **OpenHuman 的吉祥物/Mascot / Meet Agent** 是"产品 UI"层，APEX 作为后端不直接相关，但 `desktop_companion` 域的 IPC 接口要审查
5. **Lock-in 风险**：把 APEX 接到 OpenHuman 后，未来想换其他 agent runtime（如 Hermes、OpenClaw）会有大量重写

### 10.5 维护/社区风险

1. **README badge**：`status-early beta` + `Expect rough edges` —— 还未到生产稳定
2. **频繁的内部重构**（claude-mem 显示 4 月一周内 3+ draft PR 重构同一内存抽象）—— 短期 API 不稳定
3. **AGENTS.md 极详尽**（650 行）反过来说明**约定复杂**，新人上手成本高

---

## 附录 A：关键数字速查

| 指标 | 值 |
|---|---|
| 版本 | v0.54.10 (core), v0.53.45 (openhuman-app) |
| 总源文件 | 2538（.rs 1469 + .ts 597 + .tsx 472） |
| Rust crates 直接依赖 | 90+ |
| Rust tests | 5800+ |
| TS/TSX tests | 343 个测试文件，1000+ 用例 |
| `src/openhuman/` 子模块 | 98 |
| `src/openhuman/agent/agents/` built-ins | 17 |
| `src/openhuman/channels/providers/` | 20+ 渠道（telegram/discord/slack/email/matrix/signal/whatsapp/irc/lark/dingtalk/qq/mattermost/linq/imessage/web/...） |
| `src/openhuman/composio/providers/` | Gmail/Slack/Notion/GitHub/Stripe/Calendar/Drive/Linear/Jira/...（118+）|
| `src/bin/` 工具 | 5 |
| 单文件最长 | 3097L (`core/observability.rs`) |
| 核心 RPC port | 7788 |
| Vite dev port | 1420 |
| Memory chunk 上限 | 3k token |
| Auto-fetch 周期 | 20 分钟 |
| Coverage 合并门槛 | 80% diff coverage |
| 关键 patch fork | whisper-rs-sys (Windows MSVC LNK2038 fix) |

## 附录 B：可直接复用的 5 个文件

| 借鉴目标 | 源文件 | 行数 |
|---|---|---|
| Memory Tree 双写引擎 | `src/openhuman/memory_tree/ingest.rs` | 723 |
| Memory Tree 检索 6 工具 | `src/openhuman/memory_tree/retrieval/*.rs` | 4000+ |
| Agent TOML 数据驱动 | `src/openhuman/agent/agents/loader.rs` | ~200 |
| Orchestrator 模板 | `src/openhuman/agent/agents/orchestrator/agent.toml` + `prompt.rs` | 50+100 |
| TokenJuice 压缩 | `src/openhuman/tokenjuice/reduce.rs` + `classify.rs` | 928+487 |
| 双 EventBus 模式 | `src/core/event_bus/{bus.rs, native_request.rs}` | ~600 |
| 邮件清理 | `src/openhuman/memory_tree/canonicalize/email_clean.rs` | 411 |

---

**分析完成**。本文档约 12000 字，覆盖任务要求的 10 个维度，可作为团队评审 openhuman 与 APEX 整合的参考。
