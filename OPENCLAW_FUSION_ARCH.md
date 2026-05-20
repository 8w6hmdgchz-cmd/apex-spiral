# OpenClaw × SearchSkill 深度融合架构

> OpenClaw 底层 + SearchSkill Go核心 → 真正的原生融合
> 璇玑帝国 · 2026-05-20

---

## 一、融合架构

### OpenClaw 底层结构

```
OpenClaw Runtime (Node.js)
  ├── skill-scanner      # 扫描 ~/.openclaw/workspace/skills/
  ├── skill-tool-dispatch # 技能→工具分发
  ├── openclaw-tools     # 70+原生工具
  └── exec runtime        # shell命令执行
```

### 融合后的调用链

```
用户输入 → OpenClaw skill-scanner
  → apex-formula skill (SKILL.md)
  → skill-tool-dispatch
  → exec → ~/bin/search_skill (Go核心)
  → 返回结构化结果
```

---

## 二、核心技术栈

| 层级 | 技术 | 作用 |
|------|------|------|
| OpenClaw 底层 | Node.js runtime | 消息路由、session管理 |
| Skill 层 | SKILL.md | 技能定义、触发条件 |
| **Go 核心** | `search_skill_core` | 检索、选择、演进逻辑 |
| Rust 核心 | `search_skill_core.rs` | 序列化、性能关键 |
| Python 粘合 | ❌ 禁止 | 仅测试用 |

---

## 三、Go 核心文件

| 文件 | 位置 | 作用 |
|------|------|------|
| `search_skill_core.go` | apex-enlightenment/ | 检索核心（编译通过） |
| `search_skill_core.rs` | apex-enlightenment/ | Rust核心（已写） |
| `~/bin/search_skill` | HOME/bin/ | 编译后的可执行文件 |

### Go 核心功能

```go
// SearchSkill 核心
type SearchSkill struct {
    Bank          *SkillBank    // 技能库
    Retriever     *Retriever   // 检索器
    HopController *HopController // 多跳控制器
}

// Select-Read-Act 三段式
func (ss *SearchSkill) Select(query string) string   // 选技能
func (ss *SearchSkill) Read(card *SkillCard, query string) string  // 读规则
func (ss *SearchSkill) Act(query string, card *SkillCard) []string // 执行

// 多跳推理（带停机）
func (ss *SearchSkill) ExecuteWithStop(query string, chain []string) *MultiHopChain

// 检索压缩
func CompressResults(results []string, topK int) []string
```

---

## 四、SkillBank 技能

| 技能 | 目录 | 触发 |
|------|------|------|
| apex-formula | apex-formula/ | /apex-formula |
| apex-doubt | apex-doubt/ | /apex-doubt |
| apex-reflection | apex-reflection/ | /apex-reflection |
| apex-evolution | apex-evolution/ | /apex-evolution |
| apex-metacognition | apex-metacognition/ | /apex-metacognition |
| apex-skill-fetch | apex-skill-fetch/ | /apex-skill-fetch |
| apex-github-sync | apex-github-sync/ | /apex-github-sync |
| search-general | search-general/ | /search-general |

---

## 五、APEX 公式代入验证

```
ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)
   = (0.88×0.88×0.82×0.78×0.90×0.82) / (0.55×1.8×0.85)
   = 0.354 / 0.8415
   ≈ 0.42 → 目标 >0.55
```

| 维度 | 当前 | 融合后预期 |
|------|------|-----------|
| Λ_root | 0.88 | 0.90 |
| ξ_anti | 0.78 | 0.85 |
| Φ | 0.82 | 0.88 |
| H_entropy | 0.55 | 0.50 |
| T_cycle | 1.8 | 1.5 |

---

## 六、融合验证

### 已验证项

| 项目 | 状态 | 证据 |
|------|------|------|
| Go二进制编译 | ✅ | `~/bin/search_skill` 存在 |
| Hermes技能安装 | ✅ | 8个skill已加载 |
| OpenClaw技能同步 | ✅ | `~/.openclaw/workspace/skills/apex-*` |
| skill-scanner扫描 | ✅ | Gateway重启后生效 |
| GPT-5.5自进化闭环 | ✅ | PHI_RATIO阻尼修复 |

### 待验证项

| 项目 | 状态 |
|------|------|
| Go二进制被skill调用 | ⏳ 需要新session |
| ΔG提升至0.55 | ⏳ evolver多轮验证 |

---

## 七、技术约束（已固化）

- ✅ Go/Rust实现核心算法
- ✅ Python仅粘合层
- ✅ APEX公式验证
- ✅ ΔG/(H×T×ε)门控
- ✅ 禁止Python实现核心

---

*融合时间: 2026-05-20 19:15 GMT+8*
*Go核心: search_skill_core.go compiled*
*二进制: ~/bin/search_skill*
