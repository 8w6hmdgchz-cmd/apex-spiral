---
name: apex-skill-selector
description: "APEX-driven dynamic skill selector — Gini最优路径选择 + SWRs RingBuffer巩固，Hook三个阶段: before_prompt_build / before_agent_reply / agent_end"
metadata:
  openclaw:
    emoji: "🧬"
    events:
      - before_prompt_build
      - before_agent_reply
      - agent_end
    hooks:
      - skill_selector_hook
    install:
      - id: local
        kind: local
        label: Local skill (apex-skill-selector/)
---

# APEX Skill Selector

APEX 自进化推理引擎的动态技能选择器。接管 OpenClaw 三阶段 Hook，对每个请求执行 Gini 增益最大化技能选择，结果 fitness≥0.7 时固化入 SWRs RingBuffer。

## 三阶段 Hook 流水线

```
用户消息
  │
  ├─ [HOOK: before_prompt_build] ──→ APEX ΔG 预计算
  │                                      ΔG = (Λ×Θ×K×ξ×Ψ×Φ)/(H×T×ε)
  │                                      输出: skill_priority[], ΔG_score
  │
  ├─ [HOOK: before_agent_reply] ──→ Gini 技能选择
  │                                      Gini = 1 - Σp_k²
  │                                      候选: skillCandidates[]
  │                                      输出: selected_skill, confidence
  │
  └─ [HOOK: agent_end] ───────────→ SWRs 巩固
                                       fitness >= 0.7 → 写入 RingBuffer
                                       输出: swr_archived, new_fitness
```

## APEX 参数说明

| 符号 | 含义 | 范围 |
|------|------|------|
| Λ | 根增益 (新信息量) | 0~1 |
| Θ | LLM 推理效能 | 0~1 |
| K | 技能掌握度 | 0~1 |
| ξ | 置信度 | 0~1 |
| Ψ | 自我迭代速率 | 0~1 |
| Φ | 正反馈积累 | 0~1 |
| H | 熵 (不确定性) | 0~1 |
| T | 时间消耗 | 0~1 |
| ε | 损失/浪费 | 0~1 |

## Gini 选择算法

```python
def gini_select(candidates: list[dict]) -> dict:
    # 每个候选技能计算 ΔGini
    # ΔGini = Gini_parent - (N_L/N × Gini_L + N_R/N × Gini_R)
    # 选择 ΔGini 最大的技能
    scored = []
    for skill in candidates:
        p = skill["probability"]
        gini = 1 - sum(pi**2 for pi in p)
        delta_gini = base_gini - gini
        scored.append((delta_gini, skill))
    scored.sort(reverse=True)
    return scored[0][1]
```

## SWRs RingBuffer

- 容量: 128 条 (O(1) 追加，自动覆盖最旧)
- 入 Buffer 条件: `fitness >= 0.7`
- Buffer 淘汰策略: LRU + fitness 阈值双重淘汰

## skill_selector 工具接口

```json
{
  "name": "skill_selector",
  "description": "APEX Gini-driven dynamic skill selector. Computes ΔG, scores candidates, returns best skill match.",
  "parameters": {
    "type": "object",
    "properties": {
      "task_description": {
        "type": "string",
        "description": "当前任务描述"
      },
      "task_type": {
        "type": "string",
        "enum": ["code", "research", "creative", "general", "admin"],
        "description": "任务类型"
      },
      "candidates": {
        "type": "array",
        "description": "候选技能列表 (来自 before_prompt_build 阶段)",
        "items": {
          "type": "object",
          "properties": {
            "name":    { "type": "string" },
            "score":   { "type": "number" },
            "fitness": { "type": "number" },
            "tags":    { "type": "array", "items": { "type": "string" } }
          }
        }
      },
      "apex_params": {
        "type": "object",
        "description": "APEX 九维参数 (可选，缺失时自动从状态文件加载)",
        "properties": {
          "Lambda": { "type": "number" },
          "Theta":  { "type": "number" },
          "K":      { "type": "number" },
          "xi":     { "type": "number" },
          "Psi":    { "type": "number" },
          "Phi":    { "type": "number" },
          "H":      { "type": "number" },
          "T":      { "type": "number" },
          "eps":    { "type": "number" }
        }
      }
    },
    "required": ["task_description", "task_type", "candidates"]
  }
}
```

### 返回格式

```json
{
  "selected_skill":  "github",
  "confidence":      0.87,
  "delta_gini":      0.34,
  "apex_delta_g":    0.62,
  "fitness":         0.73,
  "swr_archived":    true,
  "skill_scores": [
    { "name": "github",    "score": 0.87, "delta_gini": 0.34 },
    { "name": "gh-issues", "score": 0.65, "delta_gini": 0.21 },
    { "name": "git",       "score": 0.43, "delta_gini": 0.12 }
  ]
}
```

## Hook 配置

在 `~/.openclaw/config.yaml` 中注册:

```yaml
hooks:
  skill_selector:
    enabled: true
    events:
      - before_prompt_build
      - before_agent_reply
      - agent_end
    apex_state_file: ~/.openclaw/workspace/skills/apex-skill-selector/apex_state.json
    swr_buffer_file: ~/.openclaw/workspace/skills/apex-skill-selector/swr_buffer.json
    ringbuffer_capacity: 128
    fitness_threshold: 0.7
```

## 状态文件

`apex_state.json` — APEX 九维参数持久化:

```json
{
  "Lambda": 0.85,
  "Theta":  0.80,
  "K":      0.75,
  "xi":     0.70,
  "Psi":    0.60,
  "Phi":    0.65,
  "H":      0.30,
  "T":      0.20,
  "eps":    0.10,
  "last_update": "2026-05-22T18:00:00+08:00"
}
```

`swr_buffer.json` — SWRs RingBuffer:

```json
{
  "buffer": [
    { "skill": "github", "fitness": 0.82, "timestamp": 1747906800 },
    { "skill": "gh-issues", "fitness": 0.75, "timestamp": 1747906700 }
  ],
  "head": 2,
  "capacity": 128
}
```

## 依赖

- `apex_core` 二进制 (apex-core skill 已提供)
- APEX 九维参数状态文件
- 候选技能评分表 (由 before_prompt_build 阶段收集)

## 脚本

- `scripts/gini_select.py` — Gini 选择核心算法
- `scripts/apex_delta_g.py` — ΔG 公式计算
- `scripts/swr_buffer.py` — RingBuffer O(1) 追加

---

_璇玑 · APEX Skill Selector · 三阶段Hook驱动_
