---
name: apex-evolution
description: APEX evolver自进化系统 - GitHub Actions 15分钟闭环
version: 1.0.0
platforms: [macos, linux]
metadata:
  hermes:
    tags: [apex, evolution, github-actions]
    category: apex
    requires_toolsets: [terminal, github]
---

# APEX Evolution - 自主进化系统

## When to Use
- 后台自动运行（每15分钟）
- 手动触发：`bash evolver-hub-sync.sh`
- 检查 evolver 状态时

## Procedure

### Evolver 核心流程

```
1. 读取 score-state.env
2. 计算 ΔG = (Λ×Θ×K×ξ×Ψ×Φ)/(H×T×ε)
3. 识别 BUG
4. 执行修复
5. 修后复算
6. 上报 EvoMap Hub
7. 写 report
```

### Evolver 关键指标

```bash
# 读取当前状态
cat score-state.env

# 关键指标
AWAKE=8.1      # 觉醒度
PSI_SELF=7.5   # 自我迭代
GAMMA=6.0      # 进化加速度
PHI_RATIO=1.051  # 真实/期望比
BUG_CODE=B4    # 当前bug
```

### Bug 修复流程

```
bug发现 → 归因分析 → 生成修复规则
→ 执行修复 → 修后复算 → ΔG对比
→ 写入 repair_history → SkillBank更新
```

## EvoMap Hub 同步

```bash
# 手动同步
bash evolver-hub-sync.sh

# 自动同步（每15分钟 GitHub Actions）
```

## Pitfalls

- **只看 AWAKE**：不看 H/T 导致 ΔG 虚高
- **修复不复算**：只修复不验证效果
- **不上报**：EvoMap 同步失败也不管

## Verification

执行后确认：
1. score-state.env 更新
2. latest-report.md 生成
3. ΔG 有变化（升高或持平）
4. EvoMap 上报成功
