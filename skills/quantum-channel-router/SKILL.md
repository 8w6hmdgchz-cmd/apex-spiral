---
name: quantum-channel-router
description: 量子通道路由：根据任务意图、模型分类、可用性、速度、上下文、成本/免费优先级，动态选择 OpenClaw LLM，并生成 Full Tool-Call Trajectory 执行轨迹。
metadata: { "openclaw": { "emoji": "🧬", "requires": { "bins": ["go", "openclaw"] } } }
---

# 量子通道路由 Skill

当用户要求“量子路由/超级路由器/动态选择LLM/模型分类/多Agent路由/Full Tool-Call Trajectory/APEX评分”时，先读取本 skill。

## 核心目标

把可用 LLM 组织成一个动态路由层：

- A 国内免费/计划池 LLM：优先低成本、高速、日常任务。
- B 高端推理 LLM：复杂研究、医学科研、严肃判断、长链路推理。
- C 代码深度开发 LLM：代码生成、调试、重构、测试修复。
- D 图片/视频相关 LLM：多模态理解走支持 image 的模型；图片/视频生成优先调用 OpenClaw 原生 `image_generate` / `video_generate` 工具，而不是普通 LLM。

## 当前路由 CLI

本 skill 附带 Go 原型：

```bash
go run {baseDir}/../../scripts/quantum-router/main.go --mode list

go run {baseDir}/../../scripts/quantum-router/main.go --mode route --task '任务描述'
```

输出 JSON：

- `selected`: 推荐模型
- `fallbacks`: 失败时按顺序切换
- `classes`: 模型分类标签
- `trajectory_hash`: 轨迹指纹，可用于复用缓存
- `token_budget_hint`: token 调控建议
- `full_tool_call_trajectory`: 一次性完整执行轨迹

## 使用规则

1. **先分类再执行**：复杂任务先用 `--mode route --task` 获取推荐模型和 fallback。
2. **免费/高速优先**：普通任务优先 A 类 + fast；失败或质量不足再升级 B/C。
3. **代码任务**：优先 C 类，执行后必须跑最小测试/检查门。
4. **研究/推理任务**：优先 B 类，输出前做证据门和反幻觉自检。
5. **图片/视频生成**：不要把生成任务交给普通 LLM；优先 OpenClaw media tools。
6. **多Agent协作**：可用 `sessions_spawn` 并发派发独立子任务；给每个子任务指定路由推荐模型。若模型不支持 thinking，设为 `off` 或省略 thinking。
7. **隐私安全**：不得输出 API key；配置检查只显示 Auth yes/no。

## Full Tool-Call Trajectory 新范式

替代“执行→检查→反思→重复”的长链路，先生成整条轨迹：

1. classify_intent
2. select_model_by_score
3. generate_full_tool_call_trajectory
4. execute_independent_threads
5. verify_outputs
6. cache_successful_trajectory
7. distill_to_skill_if_reusable
8. task-specific gate：代码跑 test/lint；研究跑 evidence gate；媒体跑文件存在和格式检查

## APEX 演化评分

核心公式：

```text
ΔG = (C_total · Λ_gene · Ω_entropy · τ_traj) / (H_info · t)
```

含义：

- `C_total`: 总能力覆盖，模型/工具/agent 综合能力
- `Λ_gene`: 可复用技能基因质量
- `Ω_entropy`: 多模型/多路径信息熵收益
- `τ_traj`: 完整轨迹生成效率系数
- `H_info`: 信息噪声/上下文负担
- `t`: 总耗时

成功轨迹应缓存到 `.openclaw/quantum-router/`；高 fitness 的轨迹再沉淀为 Hermes/OpenClaw skill。

## 推荐模型分类基线

运行 CLI 会动态刷新。当前基线：

- A 国内免费/计划池：`xiaomimimo/*`, `freemodel/*`, `scnet/*`, `minimax-portal-cn/*`, `zai/*`
- B 高端推理：`deepseek/deepseek-v4-pro`, `deepseek/deepseek-reasoner`, `freemodel/gpt-5.5`, `minimax-portal/MiniMax-M2.7`, `zai/glm-5.1`, `zai/glm-5-turbo`
- C 代码深度开发：`deepseek/deepseek-v4-*`, `freemodel/gpt-5.3-codex`, `scnet/qwen`, `zai/glm-4.7`, `zai/glm-5*`, `deepseek/deepseek-chat`
- D 多模态理解：`zai/glm-5v-turbo`, `zai/glm-4.6v` 等 text+image 模型；生成任务走 `image_generate` / `video_generate`

### 智谱 / Z.AI 路由策略

智谱模型已经作为 `zai/*` 纳入量子路由器，而不要求必须出现在默认 `/models` 列表：

- `zai/glm-5.1`：高端推理默认候选。
- `zai/glm-5-turbo`：高速推理/低延迟候选。
- `zai/glm-4.7`：代码开发与通用执行候选。
- `zai/glm-5v-turbo`：多模态理解候选。
- `zai/glm-4.5-air`：轻量/快速/低成本候选。
