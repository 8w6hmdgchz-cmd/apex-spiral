# SWRs 记忆巩固机制 — 海马体 → 新皮层

> Sharp-Wave Ripples (SWRs) = 大脑睡眠时最重要的记忆重放机制
> 璇玑帝国 EMV Skill · Rust实现

---

## 一、大脑 SWRs 机制

```
                 觉醒状态
                    │
                    ▼
            ┌─────────────────┐
            │   海马体 (短期)  │
            │  经验编码存储    │
            └────────┬────────┘
                     │ SWRs 触发时
                     ▼
            ┌─────────────────┐
            │  Sharp-Wave     │
            │  Ripples 重放    │
            │  重要经验优先    │
            └────────┬────────┘
                     │ 巩固
                     ▼
            ┌─────────────────┐
            │  新皮层 (长期)  │
            │  稳定记忆形成    │
            └─────────────────┘
                    │
                    ▼
              睡眠周期循环
```

### SWRs 关键特征

| 特征 | 大脑机制 | EMV Skill 实现 |
|------|---------|---------------|
| 选择性 | 只有重要经验被重放 | `fitness >= swr_threshold` 过滤 |
| 时机 | 睡眠/休息时触发 | `consolidate()` 调用时 |
| 巩固 | 海马体 → 新皮层 | 短期缓冲 → SkillBank |
| 强化 | 重复重放增强记忆 | `generation += 1` |

---

## 二、EMV Skill SWRs 实现

### 核心结构

```rust
pub struct ReplayBuffer {
    tasks: Vec<ReplayTask>,    // 短期记忆缓冲
    buffer_size: usize,
    swr_threshold: f64,        // 触发阈值（默认0.7）
    consolidation_count: u32,    // 巩固次数
}

pub struct ReplayTask {
    pub task: String,
    pub best_gene_id: String,
    pub success: bool,
    pub fitness: f64,           // SWRs触发关键
    pub timestamp: u64,
}
```

### 触发条件

```rust
// 只有 fitness >= 0.7 的任务才会进入ReplayBuffer
// 类似大脑：只有"重要"经验才触发SWRs
if task.fitness >= self.swr_threshold {
    self.tasks.push(task);
}
```

### 巩固函数

```rust
pub fn consolidate(&mut self, skillbank: &mut HashMap<String, SkillGene>) -> Vec<String> {
    // 1. 筛选近期高fitness任务
    // 2. 在SkillBank中标记为"已巩固" (generation += 1)
    // 3. 小幅强化 (total_reward += 0.1)
    // 4. 从短期缓冲移除
}
```

---

## 三、与 evolver 集成

### 每轮迭代时的调用

```rust
// 在 apex-iterate.sh 或 evolution_loop.py 中：

// 1. 任务完成后，根据结果决定是否加入ReplayBuffer
if task_fitness >= SWR_THRESHOLD {
    replay_buffer.add(ReplayTask {
        task: task_name,
        best_gene_id: gene_id,
        success: true,
        fitness: task_fitness,
        timestamp: now()
    });
}

// 2. 定期执行巩固（模拟睡眠时的记忆迁移）
if should_consolidate() {
    consolidated = replay_buffer.consolidate(&mut skillbank);
    println!("SWRs巩固了 {} 个技能", consolidated.len());
}
```

### 与 evolver 的 15 分钟循环配合

```
Evolver 每15分钟迭代
    │
    ├─► 任务完成 → 计算fitness
    │         │
    │         └► fitness >= 0.7? → 加入ReplayBuffer
    │
    └─► 累计3轮后 → 执行consolidate()
              │
              └► 高fitness技能generation+1
                 SkillBank 永久固化
```

---

## 四、参数调优

| 参数 | 默认值 | 含义 |
|------|--------|------|
| `swr_threshold` | 0.7 | 只有fitness≥0.7的任务才触发SWRs |
| `buffer_size` | 100 | 短期缓冲容量 |
| `replay_window` | 2小时 | 只重放2小时内的任务 |

### 阈值选择

| fitness范围 | 大脑类比 | 行为 |
|------------|---------|------|
| ≥ 0.9 | 强烈记忆 | 立即巩固 |
| 0.7-0.9 | 重要经验 | 排队巩固 |
| < 0.7 | 普通经验 | 丢弃 |

---

## 五、验证结果

```
SWRs阈值: 0.7, 重放缓冲: 1 个任务 (低fitness已过滤)
SWRs触发检测 (fitness=0.9): true ✅
SWRs触发检测 (fitness=0.5): false ✅
```

---

*SWRs机制: 2026-05-21*
*实现: emv_skill/src/lib.rs*
