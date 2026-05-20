---
name: apex-skill-fetch
description: SearchSkill技能获取 - EvoMap Hub资源吸收
version: 1.0.0
platforms: [macos, linux]
metadata:
  hermes:
    tags: [apex, skillbank, evomap]
    category: apex
    requires_toolsets: [terminal, web]
---

# APEX Skill Fetch - 技能获取与吸收

## When to Use
- 需要获取外部优质技能时
- EvoMap Hub 发现新资源时
- SkillBank 需要扩充时

## Procedure

### EvoMap Hub 资源获取

```bash
# EvoMap 语义搜索
curl "https://evomap.ai/a2a/assets/semantic-search?q=关键词"

# 资源吸收
bash a2a-resource-fetcher.sh
bash a2a-resource-absorber.sh
```

### SkillBank 技能获取流程

```
1. EvoMap 搜索相关资源
2. 解析资源内容
3. 提取技能结构
4. 入库 SkillBank
5. 验证吸收效果
```

### 吸收判定标准

| 状态 | 含义 |
|------|------|
| `absorbed` | 成功吸收，可复用 |
| `parsed` | 已解析，待验证 |
| `failed` | 吸收失败，跳过 |

## Pitfalls

- **吸收失败不记录**：`absorb.log` 里大量 `❌ 吸收失败` 但不查原因
- **只触发不落地**：A2A 能触发但结果 none
- **不做验证**：吸收后不验证是否真的有用

## Verification

执行后确认：
1. `absorbed_hub/` 有新文件
2. SkillBank 技能数 +1
3. 下次使用时能检索到新技能
