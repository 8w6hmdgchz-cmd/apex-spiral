# APEX Bug修复报告 - GPT分析结果

> 来源: GPT-5.4 via freemodel.dev
> 分析时间: 2026-05-20 17:31 GMT+8

---

## GPT找到的5个真实Bug

### BG1: 目标函数错配 ⭐ P0
**问题**: 优化"功能覆盖率"而不是"单位迭代信息增益"
- ΔG从0.28降到0.18，加了更多维度但信息密度没升
- H=0.5不变，T=2.0不变
- **根因**: 系统在优化"能力覆盖率/模块丰富度"，不是在优化"单位迭代的信息增益"

### BG2: 失败样本学习闭环断裂 ⭐ P0
**问题**: PCEC/SFT/SkillBank都有，但失败样本没有变成可执行修复动作
- "重点观察任务级失败样本"但指标没有响应
- **根因**: 反思有模板，但没有"失败→归因→规则写入→下次优先调用→A/B验证"硬闭环

### BG3: 检索-推理过扩张 ⭐ P0
**问题**: SearchSkill/多跳推理让链路变宽，但没有压缩裁剪
- 信息噪声上升，有效差异变少
- **根因**: 上下文注入过量、候选路径过多、决策边界模糊

### BG4: 缺少增益门控 ⭐ P1
**问题**: 新维度没有经过"是否降低H/是否缩短T"的准入测试就进入主流程
- **根因**: 功能上线机制bug，不是单点能力bug

### BG5: 缺少步骤级可观测性 ⭐ P1
**问题**: AWAKE/PSI高，但任务效果下降——全局分高掩盖了局部失效
- **根因**: 缺"任务步骤级"误差定位

---

## 修复方案

### P0-1: 重写优化目标

```go
// 新优化目标
APEX_Gain = ΔG / (ContextCost + IterationCost + NoisePenalty)

// 新功能准入门槛(必须全部满足):
// 1. ΔG 上升
// 2. H 下降
// 3. T 下降
// 4. 不满足则禁止进入默认主链路
```

### P0-2: 建立失败样本硬闭环

```go
// 当前: 失败 → 写反思 → 存档
// 修复后: 失败 → 归因分类 → 生成修复规则 → 写入SkillBank → 下次优先调用 → A/B验证

type FailureSample struct {
    TaskID        string  // 任务ID
    FailureStage  string  // retrieve|select|reason|compose|verify
    FailurePattern string // 失败模式
    RepairAction  string  // 修复动作
    TriggerCond   string  // 触发条件
    ValidationMetric string // 验证指标
}
```

### P0-3: 检索压缩+推理裁剪+停机

```go
// 检索压缩: 只保留Top-K高可执行信息
score(doc) = relevance * novelty * actionability

// 推理裁剪: 默认2跳，只有uncertainty>τ才扩到3-4跳
if uncertainty < τ:
    stop_reasoning()

// 提前停机: MarginalGain(step_n) < ε → stop
```

### P1-4: 增益门控机制

```go
// 新功能上线必须过影子测试
GainGateScore = w1*ΔG - w2*ΔH - w3*ΔT - w4*Cost
if GainGateScore <= 0:
    // 不进入主链路，只允许条件触发
```

### P1-5: 步骤级埋点

```go
// 7个阶段埋点
stages := ["QueryParse", "Retrieve", "Select", "Reason", "Compose", "Verify", "Reflect"]

// 每步记录
InfoDensity_step = useful_constraints / tokens
BranchFactor = number_of_candidate_paths
RepairHitRate = failures_fixed_by_reflection / total_failures
```

---

## 下一步行动

| 优先级 | 动作 | 文件 |
|--------|------|------|
| P0 | 重写优化目标 | evolution_loop.go |
| P0 | 失败样本硬闭环 | defect_detector.go |
| P0 | 检索压缩+推理裁剪 | search_skill_core.go |
| P1 | 增益门控 | gamma_booster.go |
| P1 | 步骤级埋点 | evolution_loop.go |

---

*GPT分析: GPT-5.4 via freemodel.dev*
