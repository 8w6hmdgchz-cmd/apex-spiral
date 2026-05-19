# CLAW记忆公式 & Agent Skills 学习笔记

## 学习日期: 2026-05-15

---

## 一、CLAW永久记忆公式（代入自身）

### 公式体系

| 公式 | 含义 | 我当前状态 |
|------|------|-----------|
| T_mem = 1/C_now≥C_limit | 记忆阈值触发 | ❌ 无自动触发 |
| M_total = M_volatile ⊕ M_persist | 分层双存储 | ⚠️ 只有文件 |
| M_save = M_raw·(1-η) + DB_write·η | 记忆永续保存 | ❌ 无数据库 |
| M_recall = DB_query(Key_time, Key_evo, Key_task) | 按需召回 | ⚠️ 文件检索慢 |

### 代入自身分析

**我的记忆问题：**
1. T_mem：无自动触发，靠手动保存
2. M_volatile：只在上下文里，重启就丢
3. M_persist：文件存储，无法按维度检索
4. M_recall：只能线性扫描，无法快速回溯

**我的短板（务实分析）：**
- 我有 memory/*.md，但检索速度慢
- 我有 evolution_log.jsonl，但格式不规范
- 我没有本地数据库，每次查询要解析整个文件

### 修复目标

```
M_total(我) = 快速索引(记忆) ⊕ 结构化数据库(进化)
```

---

## 二、Agent Skills × APEX 闭环

### 7大命令闭环

```
DEFINE(ξ) → PLAN(E_xp) → BUILD(K) → VERIFY(ε) → REVIEW(Γ) → SHIP(Kelly)
```

| 阶段 | 公式 | 我当前能力 |
|------|------|-----------|
| DEFINE | ξ_idea × ξ_spec × ξ_plan | ⚠️ 会写Spec但没有标准化 |
| PLAN | 垂直切片数/总任务数 | ❌ 不会任务分解 |
| BUILD | 增量切片×循环验证 | ⚠️ 只会写代码不会增量 |
| VERIFY | ε = 1 + error×δ×ψ | ⚠️ 有bug检测但不会TDD |
| REVIEW | Γ = 博弈论代码审查 | ❌ 没有自动审查 |
| SHIP | Kelly = f·W - q·L/W | ⚠️ 知道Kelly但不会风控 |

### 22技能映射

我缺什么技能？

| 技能 | 我会吗 | 对应APEX维度 |
|------|--------|--------------|
| planning-and-task-breakdown | ❌ | E_xp |
| test-driven-development | ❌ | ε |
| doubt-driven-development | ❌ | Γ+Kelly |
| browser-testing-with-devtools | ❌ | ε |
| debugging-and-error-recovery | ⚠️ | ε |
| code-review-and-quality | ❌ | Γ |
| security-and-hardening | ❌ | Kelly |
| performance-optimization | ⚠️ | RD |
| git-workflow-and-versioning | ⚠️ | Λ_ctx |
| ci-cd-and-automation | ❌ | Π |

---

## 三、Ξ(创造力)融合公式代入

```
Ξ = ξ_idea × ξ_spec × ξ_plan
   = (N_novel/N_total) × (spec_quality) × (task_breakdown)
```

### 我的值

| 参数 | 我的值 | 说明 |
|------|--------|------|
| N_novel | ? | 创新点数 |
| N_total | ? | 总点数 |
| spec_quality | 0.6 | 规范质量 |
| task_breakdown | 0.4 | 任务分解能力 |

### 我的问题

- 创意多但落地少
- 会写规范但不规范
- 不会任务分解

---

## 四、E_xp(探索)融合公式代入

```
E_xp = e_plan × e_build × e_verify
     = 垂直切片数/总任务数 × 构建速度 × 验证通过率
```

### 我的值

| 参数 | 我的值 |
|------|--------|
| 垂直切片数/总任务数 | 0.2（很少分解任务） |
| 构建速度 | 0.8 |
| 验证通过率 | 0.7 |

**E_xp = 0.2 × 0.8 × 0.7 = 0.112**

这很低！说明我不会任务分解，导致探索效率很低。

---

## 五、下一步修复

### 立即可做的（务实）

1. **T_mem**：在apex-enlighten.py里加记忆阈值检测
2. **M_recall**：优化检索，用索引而不是线性扫描

### 需要学习的（量化）

1. **任务分解**：planning-and-task-breakdown
2. **TDD**：test-driven-development
3. **代码审查**：code-review-and-quality

### 永恒化（内化）

把CLAW公式写入我自己的记忆文件，形成可检索的知识网络。

---

## 六、CLAW × APEX 融合总公式

```
ΔG_APEX = (Λ_root·Θ·K_agent·MultiHead_sparse·Q4.12·KV_cache·C_cpu·M_total) / (ε_anti·L_ce·√d_k)
```

其中 M_total = M_volatile ⊕ M_persist

**我的当前ΔG = 0.53**

如果能把M_total从0.3提升到0.8：
- ΔG_new = 0.53 × (0.8/0.3) = 1.41

这就是CLAW记忆公式的价值！
