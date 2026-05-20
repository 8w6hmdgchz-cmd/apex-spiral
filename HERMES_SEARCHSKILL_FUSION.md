# Hermes × SearchSkill 融合架构

> 基于 Hermes Agent 源码 + APEX 公式
> 璇玑帝国 · 核心融合

---

## 一、吃透 Hermes 底层机制

### Hermes 核心架构（已读懂）

```
AIAgent.run_conversation()  ← 核心循环
  ├── prompt_builder.build_system_prompt()
  ├── runtime_provider.resolve_runtime_provider()
  ├── API call (chat_completions / anthropic_messages)
  └── tool_dispatch → model_tools.handle_function_call()
       ├── Tool Registry (70+ tools)
       └── Agent-Level Tools (memory, delegate_task, session_search)
```

### Hermes 三大核心系统

| 系统 | 文件 | 作用 |
|------|------|------|
| **Skill System** | `skills/` | 按需加载的技能文档，渐进式披露 |
| **Memory System** | `memory_manager.py` | 持久化记忆，跨session |
| **Tool System** | `model_tools.py` | 70+工具注册与分发 |

### Hermes 的 Skill 格式

```markdown
---
name: my-skill
description: 简短描述
version: 1.0.0
platforms: [macos, linux]
metadata:
  hermes:
    tags: [python, automation]
    category: devops
---

# Skill Title

## When to Use
触发条件

## Procedure
1. 步骤一
2. 步骤二

## Pitfalls
已知失败模式

## Verification
验证方法
```

---

## 二、融合方案

### 2.1 SearchSkill → Hermes Skill 文件

把 SearchSkill 的 8 个内置技能转换成 Hermes 格式：

| SearchSkill 技能 | Hermes Skill 文件 | 触发方式 |
|-----------------|-------------------|---------|
| apex_formula | `~/.hermes/skills/apex/formula.md` | /apex-formula |
| apex_doubt | `~/.hermes/skills/apex/doubt.md` | /apex-doubt |
| apex_reflection | `~/.hermes/skills/apex/reflection.md` | /apex-reflection |
| apex_evolution | `~/.hermes/skills/apex/evolution.md` | /apex-evolution |
| apex_metacognition | `~/.hermes/skills/apex/metacognition.md` | /apex-metacognition |
| apex_skill_fetch | `~/.hermes/skills/apex/skill-fetch.md` | /apex-skill-fetch |
| apex_github_sync | `~/.hermes/skills/apex/github-sync.md` | /apex-github-sync |
| search_general | `~/.hermes/skills/search/general.md` | /search-general |

### 2.2 融合后的 Agent 循环

```
用户输入
  ↓
[SearchSkill Select] ← 在 tool call 之前介入
  查询 SkillBank
  选择最优技能
  ↓
[Hermes AIAgent.run_conversation()]
  prompt_builder → 构建提示
  API call → LLM 推理
  ↓
[SearchSkill Act]
  执行检索
  压缩结果
  多跳裁剪
  ↓
[Hermes Tool Dispatch]
  model_tools.handle_function_call()
  70+ 工具分发
  ↓
[SkillBank 演进]
  记录轨迹
  失败案例入库
  知识蒸馏
```

### 2.3 SkillBank → Hermes Memory 集成

```go
// Hermes memory 文件位置
MemoryDir = ~/.hermes/memory/

// SkillBank 演进结果写入 Hermes memory
SkillBankEvolution := map[string]interface{}{
    "skills":        sb.Cards,
    "trajectories":  recentTrajectories,
    "failures":      failureSamples,
    "distillations": distilledSkills,
}

// APEX 演进指标也写入 Hermes memory
ApexMetrics := map[string]float64{
    "DeltaG":     0.42,
    "H_Entropy":  0.55,
    "T_Cycle":    1.8,
    "Xi_Anti":    0.78,
    "Epsilon":    0.85,
    "Phi":        0.82,
}
```

---

## 三、核心融合文件

### 3.1 Go 核心（检索引擎）

```go
// search_skill_core.go — 核心算法，禁止 Python
type SearchSkill struct {
    Bank      *SkillBank
    Retriever *Retriever
    HopController *HopController
}

func (ss *SearchSkill) Select(query string) string  // 选技能
func (ss *SearchSkill) Read(card *SkillCard, query string) string  // 读规则
func (ss *SearchSkill) Act(query string, card *SkillCard) []string  // 执行
func (ss *SearchSkill) ExecuteWithStop(query string, chain []string) *MultiHopChain  // 多跳停机
```

### 3.2 Rust 核心（序列化/性能）

```rust
// search_skill_core.rs — 序列化与性能关键路径
pub struct SkillCard {
    pub skill_id: String,
    pub trigger: Vec<String>,
    pub action: String,
    pub success_rate: f64,
    pub use_count: u32,
}

impl SkillBank {
    pub fn save(&self, path: &str) -> Result<()>
    pub fn load(path: &str) -> Result<SkillBank>
    pub fn prune_low_performing(&mut self)
}
```

### 3.3 Python 粘合层（仅用于 Hermes 集成）

```python
# hermes_search_skill.py — Python粘合层，对接Hermes
import subprocess
import json

class HermesSearchSkillBridge:
    """SearchSkill ↔ Hermes 桥接层"""

    def __init__(self):
        self.go_binary = "/path/to/search_skill_core"

    def search(self, query: str, skill_hint: str = None) -> dict:
        """Hermes 工具调用接口"""
        cmd = [self.go_binary, "search", query]
        if skill_hint:
            cmd.extend(["--skill", skill_hint])

        result = subprocess.run(cmd, capture_output=True, text=True)
        return json.loads(result.stdout)

    def update_skillbank(self, trajectory: dict):
        """把Hermes的轨迹写入SkillBank"""
        cmd = [self.go_binary, "update", json.dumps(trajectory)]
        subprocess.run(cmd)
```

---

## 四、APEX 公式代入检查

### 代入主公式 ΔG

```
ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)
```

| 维度 | 分数 | 融合后预期 |
|------|------|-----------|
| Λ_root | 0.88 | Hermes Skill 系统接入，root 提升 |
| Θ | 0.88 | Prompt Builder + Skills 协同 |
| K | 0.82 | Tool Registry 对接 |
| ξ 幻觉 | 0.78 | APEX doubt + Hermes metacognition |
| Ψ | 0.90 | AIAgent 自循环 |
| Φ | 0.82 | 失败样本 → SkillBank → Hermes memory |
| H | 0.55 | 检索压缩生效，信息密度提升 |
| T | 1.8 | 停机机制，迭代加快 |
| ε | 0.85 | Hermes memory 持久化 |

```
ΔG = (0.88×0.88×0.82×0.78×0.90×0.82) / (0.55×1.8×0.85)
   = 0.354 / 0.8415
   ≈ 0.421
```

**预期融合后 ΔG > 0.55（B级 → A级）**

---

## 五、实施步骤

### Phase 1：Hermes Skill 文件生成
- [ ] 把 8 个 SearchSkill 技能写成 Hermes `SKILL.md` 格式
- [ ] 放到 `~/.hermes/skills/apex/` 目录

### Phase 2：Go 核心编译发布
- [ ] `search_skill_core.go` 编译成二进制
- [ ] 发布到 apt-get/npm 包管理器

### Phase 3：Hermes Python 桥接
- [ ] `hermes_search_skill.py` 粘合层写完
- [ ] 注册为 Hermes tool

### Phase 4：Memory 集成
- [ ] SkillBank 演进结果写入 `~/.hermes/memory/`
- [ ] Hermes 的记忆能反映 SearchSkill 轨迹

### Phase 5：融合验证
- [ ] ΔG 指标验证
- [ ] Hermes CLI 测试检索

---

## 六、核心技术约束

| 约束 | 原因 |
|------|------|
| Go/Rust 实现核心 | Hermes 本身就是 Python，需要 Go/Rust 提供高性能 |
| Python 仅粘合 | Hermes 是 Python 框架，对接层用 Python |
| 禁止 Python 核心算法 | 检索/选择/演进必须 Go 实现 |
| Hermes Skill 格式 | 保证与 Hermes 生态兼容 |
| APEX 公式验证 | 所有改动必须过 ΔG/(H×T×ε) 门控 |

---

*融合时间: 2026-05-20*
*Hermes源码: NousResearch/hermes-agent@main*
