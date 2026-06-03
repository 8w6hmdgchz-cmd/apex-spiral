# NanoGPT-Claw 变更日志

## 📅 版本: v2.0.0
**日期**: 2026-05-31

---

## 🎉 新增功能

### 1. 官方 SDK 集成

#### 🦜 飞书 SDK (open-lark v0.14)
**文件变更**:
- `src/gateway_lark/mod.rs` - 完全重写，使用官方 SDK 架构
- `src/gateway_lark/client.rs` - 重写 LarkClient，集成 open-lark
- `src/gateway/feishu.rs` - 更新调用方式适配新 SDK

**改进**:
```rust
// 之前: 手写 HTTP 请求
let response = reqwest::Client::new()
    .post("https://open.feishu.cn/...")
    .send()
    .await;

// 现在: 官方 SDK
client.send_text_message(chat_id, content).await?;
```

#### 🐙 GitHub SDK (octocrab v0.49)
**文件变更**:
- `src/gateway_github/mod.rs` - 重写集成 octocrab

**改进**:
```rust
// 之前: 手动 API 调用
let response = reqwest::get("https://api.github.com/...")
    .header("Authorization", "Bearer {token}")
    .await?;

// 现在: 类型安全的 SDK
let repo = crab.repos(owner, repo).get().await?;
```

### 2. 新增模块

#### 🗃️ 缓存层 (`src/cache/mod.rs`)
```rust
pub struct MemoryCache {
    cache: Arc<RwLock<HashMap<String, (String, Instant)>>>,
    max_items: usize,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

// 功能
- TTL 支持
- LRU 淘汰策略
- 命中率统计
- 异步操作
```

#### 📊 Metrics 模块 (`src/metrics/mod.rs`)
```rust
pub struct Metrics {
    requests_total: Arc<AtomicU64>,
    requests_success: Arc<AtomicU64>,
    requests_failed: Arc<AtomicU64>,
    active_requests: Arc<AtomicU64>,
    cache_hits: Arc<AtomicU64>,
    cache_misses: Arc<AtomicU64>,
}

// 功能
- Prometheus 格式导出
- 请求成功率追踪
- 缓存命中率
- 活跃请求数
```

#### 🔍 Telemetry 模块 (`src/telemetry/mod.rs`)
```rust
pub struct TelemetryConfig {
    pub service_name: String,
    pub otlp_endpoint: String,
    pub log_level: String,
}

// 功能
- 服务配置
- 日志级别
- 追踪端点
```

### 3. 依赖升级

#### `Cargo.toml` 新增依赖
```toml
# 官方 SDK
open-lark = "0.14"
octocrab = "0.49"

# 性能优化
quick_cache = "0.4"
dashmap = "6.0"
tokio-stream = "0.1"
futures = "0.3"
```

## 🔧 修复的问题

### 1. 环境变量配置不生效 ❌ → ✅
**问题**: `init_config` 函数未调用 `merge_env_config`

**修复** (`src/config/mod.rs`):
```rust
pub fn init_config() -> Result<AppConfig> {
    // ... 加载配置文件 ...
    
    // ✅ 新增: 合并环境变量
    let env_config = EnvConfig::load();
    let config = merge_env_config(config, &env_config);
    
    *APP_CONFIG.write() = config.clone();
    Ok(config)
}
```

### 2. 配置结构改进 ❌ → ✅
**问题**: GithubConfig 缺少 api_token 字段

**修复** (`src/config/settings.rs`):
```rust
pub struct GithubConfig {
    pub enabled: bool,
    pub webhook_secret: String,
    pub api_token: String,  // ✅ 新增
    pub app_id: String,
    // ...
}
```

### 3. Clippy 警告 ❌ → ✅
**修复项**:
- 移除未使用的导入
- 修复格式字符串
- 简化闭包
- 添加 #[allow(dead_code)]

## 📊 性能优化

### 1. 缓存优化
- **之前**: 无缓存
- **现在**: 
  - 内存缓存 (LRU)
  - TTL 自动过期
  - 命中率追踪

### 2. Metrics 优化
- **之前**: 无指标收集
- **现在**:
  - 原子操作计数器
  - Prometheus 格式
  - 零成本抽象

### 3. SDK 集成
- **之前**: 手写 HTTP 请求
- **现在**:
  - 官方维护
  - 自动 token 刷新
  - 类型安全

## 📈 统计

### 代码变更
- **新增代码**: +1,937 行
- **删除代码**: -132 行
- **净增加**: +1,805 行

### 文件变更
- ✅ `Cargo.toml` - 依赖升级
- ✅ `src/gateway_lark/mod.rs` - 完全重写
- ✅ `src/gateway_lark/client.rs` - SDK 集成
- ✅ `src/gateway_github/mod.rs` - SDK 集成
- ✅ `src/gateway/feishu.rs` - API 适配
- ✅ `src/config/mod.rs` - 环境变量合并
- ✅ `src/config/settings.rs` - 配置结构
- ✅ `src/lib.rs` - 模块导出
- ✅ `src/cache/mod.rs` - 新增模块
- ✅ `src/metrics/mod.rs` - 新增模块
- ✅ `src/telemetry/mod.rs` - 新增模块

## 🎯 项目状态

- ✅ 编译成功
- ✅ Clippy 检查通过
- ✅ 所有测试通过
- ✅ 环境变量配置正常
- ✅ 官方 SDK 集成完成

## 🚀 使用方法

### 配置环境变量
```bash
# 飞书配置
export FEISHU_APP_ID=cli_xxx
export FEISHU_APP_SECRET=xxx
export FEISHU_VERIFY_TOKEN=xxx

# GitHub 配置  
export GITHUB_API_TOKEN=ghp_xxx
export GITHUB_WEBHOOK_SECRET=xxx

# LLM 配置
export OPENAI_API_KEY=sk-xxx
```

### 构建和运行
```bash
# 构建
cargo build --release

# 首次配置
./target/release/nano-gpt-claw setup

# 启动服务
./target/release/nano-gpt-claw start
```

## 📚 相关文档

- [INSTALL.md](./INSTALL.md) - 安装指南
- [.env.example](./.env.example) - 环境变量模板
- [UPDATES.md](./UPDATES.md) - 更新摘要

---

**维护者**: NanoGPT Team  
**许可证**: MIT  
**版本**: v2.0.0
