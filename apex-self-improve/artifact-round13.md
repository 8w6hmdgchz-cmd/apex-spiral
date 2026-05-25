# Round-13 本地修复工件

## 修复1: 连续触发失败检测

**问题**: awakeningHabitRule 追踪了触发但未追踪连续未触发

**修复**: 增加 `awakeningHabitRule.continuousTriggerFailCount` 字段

```json
{
  "continuousTriggerFailCount": 0,
  "maxAllowedFailCount": 3,
  "failAction": "触发自我审视"
}
```

## 修复2: ξ_anti 独立追踪

**问题**: anti-幻觉防御依赖 outputControlGate 间接追踪

**修复**: 增加独立追踪

```json
{
  "xi_anti_tracking": {
    "independentTracking": true,
    "claimHedgingCount": 0,
    "lastHedgingRound": 13,
    "hedgingPhrases": ["(我可能错了)", "(需要验证)", "(除非明确证据)"]
  }
}
```

## 修复3: Φ_positive 追踪缺失

**问题**: Φ_positive (建设性进展) 没有任何追踪

**修复**: 增加基本追踪

```json
{
  "phi_positive_tracking": {
    "independentTracking": true,
    "constructiveActionCount": 0,
    "round13_action": "添加连续触发失败检测和独立追踪字段",
    "verification": "需要下轮验证实际行为"
  }
}
```

---

## 科学公式学习映射

**公式**: ΔG = ΔH - TΔS (吉布斯自由能变化)

| 类型 | 内容 |
|------|------|
| **事实** | 吉布斯自由能变化等于焓变减去温度与熵变的乘积 |
| **推断** | APEX的ΔG类似能量驱动，公式因子类似热力学驱动力 |
| **假设** | 当ΔG<0时过程自发，类似APEX中ΔG>1时能力提升自发进行 |

---