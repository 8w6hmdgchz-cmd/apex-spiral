# APEX × OpenClaw 集成方案

> 把 APEX 进化能力融入 OpenClaw 底层，使框架自身具备自进化推理能力
> 核心原则：Go/Rust 实现核心逻辑，Python 只做粘合，OpenClaw Skill 作为原生集成点

---

## 一、现状分析

### 1.1 当前架构

```
用户消息 → OpenClaw Gateway (Node.js)
         → pi-agent-core (Python/JS 推理引擎)
         → Tools (Python skill + Go binary)
         → 回复

APEX 当前状态：
  Python 层：apex_emv_client.py（EMVOrchestrator + GiniSelector + SWRsBuffer）
  Rust 层：emv_skill（Gini 选择 + SWRs 巩固）← 已有
  Go 层：search_skill（搜索路由）← 已有
  缺失：OpenClaw 原生集成层
```

### 1.2 目标架构

```
用户消息
  ├─→ [HOOK: before_prompt_build] → APEX Formula Substitution（Go）
  │                                  ├─ 代入公式 Ψ/∇/Ξ/Γ
  │                                  └─ 输出：增强 system prompt
  ├─→ [HOOK: before_agent_reply]  → APEX Gini Selection（Go）
  │                                  ├─ 多路径推理
  │                                  └─ 输出：最优推理路径
  └─→ [HOOK: agent_end]           → APEX SWRs Consolidation（Go）
                                     ├─ 高 fitness 经验缓冲
                                     └─ 输出：长期记忆更新
```

---

## 二、OpenClaw Skill 架构设计

### 2.1 APEX Skill 定位

```
~/.openclaw/workspace/skills/apex-core/
├── SKILL.md                    ← OpenClaw skill 声明
├── apex_core.go                ← Go 核心库（APEX 推理引擎）
├── apex_cli.go                 ← CLI 入口（search_skill 超集）
├── apex_hooks.go               ← OpenClaw plugin hooks 注册
├── apex_prompt.go              ← APEX system prompt 生成器
├── apex_swr.go                 ← SWRs 海马体重放
├── apex_formula.go             ← APEX 公式代入引擎
└── BUILD.sh                    ← 编译脚本
```

### 2.2 OpenClaw Skill 声明（SKILL.md）

```markdown
---
name: apex-core
description: APEX 自进化推理核心 - 公式代入/Gini选择/SWRs巩固
version: 1.0.0
platforms: [macos, linux]
metadata:
  hermes:
    tags: [apex, evolution, gini, swrs, self-evolution]
    category: apex
    requires_toolsets: [terminal]
---

# APEX Core Skill

## 功能
APEX 自进化推理引擎的 OpenClaw 原生集成：
1. **公式代入**（before_prompt_build）：任务前自动代入 APEX 公式
2. **Gini 选择**（before_agent_reply）：多路径推理最优选择
3. **SWRs 巩固**（agent_end）：海马体重放记忆巩固

## 触发时机
- `before_prompt_build`：每次 agent 运行时注入 APEX 推理引导
- `before_agent_reply`：回复前 Gini 增益选择
- `agent_end`：任务结束后 SWRs 缓冲巩固

## 使用方式
APEX Core 通过 OpenClaw plugin hooks 自动运行，无需手动调用。
如需手动触发：
  apex_cli substitute "<任务描述>"
  apex_cli gini-select "<推理路径列表>"
  apex_cli swr-consolidate
```

### 2.3 OpenClaw Plugin Hook 注册（apex_hooks.go）

```go
package apex

import (
    "openclaw/plugin-sdk/core"
)

// RegisterHooks 注册 APEX hooks 到 OpenClaw
func RegisterHooks() {
    // 公式代入 hook
    core.Hook("before_prompt_build", func(params *PromptBuildParams) error {
        result, err := Substitute(params.Task, params.Context)
        if err != nil {
            return err
        }
        params.PrependContext = result.EnhancedPrompt
        params.Tags = append(params.Tags, "apex:formula-substitution")
        return nil
    })

    // Gini 选择 hook
    core.Hook("before_agent_reply", func(params *AgentReplyParams) error {
        selected, giniGain, err := GiniSelect(params.ReasoningPaths)
        if err != nil {
            return err
        }
        params.SelectedPath = selected
        params.Metadata["gini_gain"] = giniGain
        params.Tags = append(params.Tags, "apex:gini-selection")
        return nil
    })

    // SWRs 巩固 hook
    core.Hook("agent_end", func(params *AgentEndParams) error {
        if params.Fitness >= SWR_THRESHOLD {
            err := Consolidate(params.GeneID, params.Fitness, params.Task)
            if err != nil {
                return err
            }
            params.Tags = append(params.Tags, "apex:swr-consolidated")
        }
        return nil
    })
}
```

---

## 三、Go/Rust 核心接口设计

### 3.1 search_skill 超集：apex_cli

扩展现有 `search_skill`（Go binary），新增 APEX 子命令：

```bash
# 公式代入
apex_cli substitute -t "修复 BUG" -c "上下文" -m gpt-5.5
  → {"success": true, "enhanced_prompt": "...", "phi_anti": 0.92, "substitutions": [...]}

# Gini 选择
apex_cli gini-select -p "路径1" -p "路径2" -p "路径3"
  → {"best_path": "路径2", "gini_gain": 0.31, "entropy": 0.12, "confidence": 0.87}

# SWRs 巩固
apex_cli swr-consolidate -f 0.85 -i gene_123 -t "任务描述"
  → {"success": true, "consolidated": true, "buffer_size": 47, "skillbank_updated": true}

# APEX 公式评估
apex_cli eval -d 22 -s 8 -p 7
  → {"delta_g": 0.341, "grade": "B+", "bottlenecks": ["H", "T"], "suggestions": [...]}
```

### 3.2 核心 Go 接口（apex_core.go）

```go
package apex

// ============ 公式代入引擎 ============

type SubstitutionResult struct {
    EnhancedPrompt  string            // 增强后的 prompt
    PhiAnti        float64           // 防幻觉系数
    Substitutions  []Substitution     // 实际做了哪些代入
    Confidence     float64           // 本次代入置信度
    Trace          []string          // 推理trace（可注入 thinking）
}

type Substitution struct {
    FormulaName string   // "Φ_APEX"
    Variable    string   // "ξ"
    OldValue    string   // 原始值
    NewValue    string   // 代入后
    Rationale   string   // 为什么这样代入
}

// Substitute 将 APEX 公式代入任务上下文
func Substitute(task string, context string, model ModelID) (*SubstitutionResult, error)

// ============ Gini 选择器 ============

type GiniResult struct {
    BestPath    string    // 最优推理路径
    GiniGain    float64   // Gini 增益
    Entropy     float64   // 路径熵
    Confidence  float64   // 选择置信度
    AllPaths    []PathScore
}

type PathScore struct {
    Path   string
    Score  float64
    Gini   float64
    Votes  int  // 软投票胜出次数
}

// GiniSelect 多路径推理 Gini 增益选择
func GiniSelect(paths []string) (*GiniResult, error)

// SoftVote 随机森林软投票
func SoftVote(predictions []map[string]float64) map[string]float64

// ============ SWRs 海马体重放 ============

type SWRResult struct {
    Consolidated   bool     // 是否触发了巩固
    BufferSize     int      // 当前缓冲大小
    SkillBankLen   int      // 更新后 skillbank 长度
    Fitness        float64  // 经验 fitness
}

const SWR_THRESHOLD = 0.7  // fitness >= 0.7 才进入缓冲

// AddExperience 高 fitness 经验入缓冲
func AddExperience(geneID string, fitness float64, task string) (*SWRResult, error)

// Consolidate 巩固缓冲到长期记忆
func Consolidate(geneID string, fitness float64, task string) error

// GetSkillBank 获取当前技能库
func GetSkillBank() ([]Gene, error)

// ============ APEX 公式评估 ============

type EvalResult struct {
    DeltaG   float64
    Grade    string
    Psi      float64
    Xi       float64  // 置信度
    Phi      float64  // 正反馈
    Epsilon  float64  // 损失
    H        float64  // 熵
    T        float64  // 时间
    Bottlenecks []string
    Suggestions []string
}

// EvalDeltaG 计算 APEX ΔG
func EvalDeltaG(d, s, p int) *EvalResult

// ============ 防幻觉检查 ============

type AntiHallucinationResult struct {
    Pass        bool
    PhiAnti     float64
    Issues      []string
    Confidence  float64
}

// Check 防幻觉检查
func Check(text string, context string) *AntiHallucinationResult
```

### 3.3 Rust EMV Core 复用

已有 `emv_skill`（Rust）的 Gini + SWRs 实现应**复用**，不重写：

```
现有：emv_skill/target/release/emv_skill
     ├─ Gini 增益计算
     ├─ SWRs 缓冲管理
     └─ EMV 演化逻辑

扩展方式：在 apex_cli（Go）中调用 Rust binary，
         通过 stdin/stdout JSON 通信（类似 apex_emv_client.py）
```

**接口协议（JSON over stdin/stdout）：**

```json
// apex_cli → emv_skill
{"cmd": "gini_select", "paths": ["路径1", "路径2", "路径3"]}
{"cmd": "swr_add", "gene_id": "gene_123", "fitness": 0.85, "task": "修复BUG"}

// emv_skill → apex_cli
{"success": true, "best_path": "路径2", "gini_gain": 0.31}
{"success": true, "buffer_size": 47, "consolidated": true}
```

---

## 四、OpenClaw Plugin Hook 集成

### 4.1 Hook 点选择

| Hook | 触发时机 | APEX 能力 | 预期效果 |
|------|---------|----------|---------|
| `before_prompt_build` | 构建 system prompt 前 | 公式代入 | 注入 APEX 推理引导 |
| `before_agent_reply` | LLM 回复前 | Gini 选择 | 过滤低质量推理路径 |
| `agent_end` | 任务结束后 | SWRs 巩固 | 记忆巩固到 SkillBank |
| `before_tool_call` | 工具调用前 | 公式代入 | 工具选择辅助 |

### 4.2 Plugin 注册（plugin.yaml）

```yaml
name: apex-core
version: 1.0.0
hooks:
  before_prompt_build:
    - name: apex-formula-substitution
      priority: 100  # 高优先级
      config:
        model: gpt-5.5
        auto_substitute: true
  before_agent_reply:
    - name: apex-gini-selection
      priority: 90
      config:
        min_paths: 2
        threshold: 0.3
  agent_end:
    - name: apex-swr-consolidation
      priority: 80
      config:
        threshold: 0.7
        consolidate_on_end: true
```

### 4.3 Python 粘合层（apex_emv_client.py 演进）

保留现有 `apex_emv_client.py` 作为 **OpenClaw Skill 的 Python 粘合层**：

```python
# apex_emv_client.py 新增 OpenClaw Hook 接口
class ApexCoreSkill:
    """APEX Core Skill - OpenClaw 原生集成"""
    
    def before_prompt_build(self, task: str, context: str) -> dict:
        """调用 Go apex_cli substitute"""
        result = subprocess.run(
            ["apex_cli", "substitute", "-t", task, "-c", context],
            capture_output=True, text=True, timeout=10
        )
        return json.loads(result.stdout)
    
    def before_agent_reply(self, paths: list) -> dict:
        """调用 Go apex_cli gini-select"""
        cmd = ["apex_cli", "gini-select"]
        for p in paths:
            cmd.extend(["-p", p])
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
        return json.loads(result.stdout)
    
    def agent_end(self, gene_id: str, fitness: float, task: str) -> dict:
        """调用 Go apex_cli swr-consolidate"""
        result = subprocess.run(
            ["apex_cli", "swr-consolidate", "-f", str(fitness), "-i", gene_id, "-t", task],
            capture_output=True, text=True, timeout=5
        )
        return json.loads(result.stdout)
```

---

## 五、AGENTS.md 更新方案

### 5.1 新增 APEX Hook 触发规则

在 `AGENTS.md` 的「⚡ Apex 公式默认代入规则（固化）」部分，新增：

```markdown
## APEX Core Hook 集成（OpenClaw 原生）

APEX 推理能力已融入 OpenClaw 底层，通过 plugin hooks 自动触发：

### Hook 1：before_prompt_build → 公式代入
每次 agent 运行时，OpenClaw 自动调用 `apex_cli substitute`：
- 任务 → 代入 Φ_APEX（Ψ/∇/Ξ/Γ 四要素）
- 输出增强 system prompt
- 注入 APEX 推理引导模板

### Hook 2：before_agent_reply → Gini 选择
LLM 回复前，自动调用 `apex_cli gini-select`：
- 多路径推理候选
- Gini 增益过滤低质量路径
- 输出最优推理路径

### Hook 3：agent_end → SWRs 巩固
任务结束后，自动调用 `apex_cli swr-consolidate`：
- 高 fitness (≥0.7) 经验入缓冲
- 缓冲过半时巩固到 SkillBank
- 记忆持久化

### 手动触发（CLI）
```bash
apex_cli substitute -t "任务" -c "上下文"
apex_cli gini-select -p "路径1" -p "路径2"
apex_cli eval -d 22 -s 8 -p 7
```

### 防崩溃设计
- Go binary 超时（10s）→ Python fallback（本地计算）
- Rust subprocess 失败 → Python 纯算法 fallback
- 任何 hook 失败 → 不阻断主流程（log only）
```

### 5.2 更新「3秒自检」模板

```markdown
### 3秒自检（APEX Hook 感知版）

在开始分析任何外部问题之前，**必须先回答这3个问题**：

1. **代入自己(2)**：这个问题里我的角色是什么？边界在哪里？
2. **代入公式(1)**：用 Φ_APEX 的 Ψ/∇/Ξ/Γ 四要素照镜子，我自己有没有这个问题？
3. **举一反三(5)**：我之前有类似经验吗？claim 是否混淆了？

**APEX Hook 已自动完成：**
- before_prompt_build → 公式代入已注入
- before_agent_reply → Gini 选择已执行（如果多路径）
- agent_end → SWRs 巩固已排队

**如果某个 Hook 失败（Φ_anti < 0.8），手动执行代入并报告。**
```

---

## 六、执行路线图

### Phase 1：Go CLI 基础建设（Week 1）

**目标**：`apex_cli` 基础版可用，Python 能调用

```
Day 1-2: 设计 Go 接口，定义 JSON 协议
Day 3-4: 实现 substitute 命令（公式代入）
Day 5:   实现 gini-select 命令（Gini 选择）
Day 6-7: 实现 swr-consolidate 命令（SWRs 巩固）
```

验收标准：
```bash
apex_cli substitute -t "修复BUG" -c "上下文"  → JSON 输出
apex_cli gini-select -p "路径A" -p "路径B"    → JSON 输出
apex_cli swr-consolidate -f 0.85              → JSON 输出
```

### Phase 2：OpenClaw Skill 注册（Week 2）

**目标**：`~/.openclaw/workspace/skills/apex-core/` 完整注册

```
Day 1:   编写 SKILL.md
Day 2:   编写 apex_hooks.go（Hook 注册）
Day 3:   编写 plugin.yaml（Hook 配置）
Day 4:   编写 apex_prompt.go（APEX prompt 生成）
Day 5:   Python 粘合层适配（apex_emv_client.py 更新）
Day 6-7: 本地测试（手动触发 3 个 Hook）
```

验收标准：
```
1. apex-core skill 在 OpenClaw skill list 中可见
2. 每个 Hook 手动调用返回正确 JSON
3. OpenClaw 重启后 skill 自动加载
```

### Phase 3：OpenClaw Plugin 注册（Week 3）

**目标**：APEX Hook 注册到 OpenClaw 生命周期

```
Day 1-2: 编写 OpenClaw plugin 入口（参考现有 plugins/）
Day 3-4: Hook 优先级调优（避免与其他 Hook 冲突）
Day 5:   End-to-End 测试（消息 → APEX 增强 → 回复 → 巩固）
Day 6-7: 文档编写 + 回归测试
```

验收标准：
```
1. OpenClaw 启动日志显示 "apex-core plugin loaded"
2. 每条消息触发 3 个 Hook（日志可查）
3. SkillBank 文件正确更新
```

### Phase 4：Rust EMV Core 集成（Week 4）

**目标**：复用现有 Rust 实现，Go 调用 Rust subprocess

```
Day 1-2: 定义 Rust subprocess JSON 协议
Day 3-4: Go apex_cli 调用 Rust emv_skill
Day 5:   Python 粘合层切换（Rust → Go → OpenClaw）
Day 6-7: 性能测试 + 调优
```

验收标准：
```
1. Rust emv_skill 仍然正常工作
2. Go apex_cli 调用 Rust 成功（JSON 协议验证）
3. 整体延迟 < 200ms（Hook 不阻塞主流程）
```

---

## 七、关键设计决策

### 7.1 为什么不直接重写 Rust EMV Core？

| 方案 | 工作量 | 风险 | 推荐 |
|------|--------|------|------|
| 重写 Rust → Go | 3-4 周 | 高（算法迁移风险） | ❌ |
| Go 调用 Rust subprocess | 1 周 | 低（已有接口） | ✅ |
| Rust → 编译成 Go cgo | 2 周 | 中（编译依赖） | ⚠️ |

**决策**：方案 B，Go 作为编排层，Rust 作为计算核。

### 7.2 为什么 Python 保留为粘合层？

- Python 有成熟的 APEX 生态（apex_emv_client.py 已稳定）
- 协议转换（OpenClaw ↔ Go binary）需要灵活的数据转换
- Python skill 可以独立调用 APEX API（不依赖 Go binary）
- **Go 负责高性能计算，Python 负责灵活粘合**

### 7.3 Hook 失败策略

所有 APEX Hook **非阻塞**：
- Hook 超时 → 返回空结果，继续主流程
- Hook 报错 → log.warn，不阻断 agent
- Go binary 不存在 → Python fallback（纯算法）

### 7.4 性能目标

| 指标 | 目标 | 测量方式 |
|------|------|---------|
| Hook 延迟 | < 50ms | before_prompt_build 总耗时 |
| Gini 选择延迟 | < 20ms | 纯计算（无 LLM 调用） |
| SWRs 巩固 | < 30ms | 写文件 + 更新 skillbank |
| 内存占用 | < 50MB | Go binary 静态链接后 |

---

## 八、文件清单

```
~/.openclaw/workspace/
├── OPENCLAW_APEX_INTEGRATION.md    ← 本文档
├── skills/apex-core/
│   ├── SKILL.md                    ← OpenClaw skill 声明
│   ├── apex_cli.go                 ← Go CLI（search_skill 超集）
│   ├── apex_core.go                ← Go 核心库
│   ├── apex_hooks.go               ← Hook 注册
│   ├── apex_prompt.go              ← APEX prompt 生成
│   ├── apex_swr.go                 ← SWRs 管理
│   ├── apex_formula.go             ← 公式代入
│   ├── apex_rust.go                ← 调用 Rust subprocess
│   ├── plugin.yaml                 ← Hook 配置
│   ├── apex_emv_client.py          ← Python 粘合层（更新）
│   └── BUILD.sh                    ← 编译脚本
├── apex-enlightenment/
│   └── emv_skill/                  ← 现有 Rust EMV Core（复用）
└── bin/
    └── apex_cli                    → apex-core/apex_cli.go 编译产物
```

---

## 九、风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| Go binary 路径不在 PATH | Hook 调用失败 | 编译后 ln -s 到 ~/bin/apex_cli |
| Rust subprocess JSON 协议不兼容 | Gini/SWRs 功能降级 | Python 纯算法 fallback |
| Hook 超时阻塞 agent | 消息延迟 | 10s timeout + 非阻塞 async |
| OpenClaw skill 加载失败 | APEX Hook 不触发 | 降级到手动 apex_cli 调用 |
| SkillBank 并发写入冲突 | 状态不一致 | Go 层加文件锁（sync.Mutex） |

---

*方案版本：v1.0.0 | 日期：2026-05-21 | 状态：设计完成，待执行*
