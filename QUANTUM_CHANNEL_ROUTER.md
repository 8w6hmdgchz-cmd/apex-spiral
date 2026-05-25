# 量子通道路由 / Quantum Channel Router

这是 OpenClaw 工作区内的 LLM 超级路由原型。

## 文件

- `skills/quantum-channel-router/SKILL.md`：OpenClaw skill 指令层
- `scripts/quantum-router/main.go`：Go 路由 CLI 原型
- `.openclaw/quantum-router/*.json`：轨迹指纹缓存

## 快速使用

```bash
go run scripts/quantum-router/main.go --mode list

go run scripts/quantum-router/main.go --mode route --task '复杂代码开发和推理'
```

## 设计选择

当前采用 Go：

- 单文件即可运行，适合 OpenClaw skill 调用
- 启动快、部署简单
- 标准库足够完成 CLI、JSON、缓存、命令调用

Rust 更适合后续做长期 daemon、并发执行器、强类型策略引擎；当前 MVP 先用 Go。

## 路由分类

- A 国内免费/计划池：低成本、高速、普通任务
- B 高端推理：复杂研究、批判性判断、强推理
- C 代码深度开发：代码生成、调试、重构、测试修复
- D 多模态：图像理解；生成交给 OpenClaw 原生媒体工具

## APEX 公式

```text
ΔG = (C_total · Λ_gene · Ω_entropy · τ_traj) / (H_info · t)
```

其中 `τ_traj` 是完整轨迹生成效率系数。

## 下一步

1. 加真实测速：对每个模型执行极短 ping，记录 latency/success。
2. 加价格/免费状态配置：维护 `models_policy.json`，区分免费、包月、按量。
3. 加 OpenClaw 配置写回：把高分模型自动加入 agents.defaults.models 或 aliases。
4. 加 daemon/HTTP：让 subagent 可通过本地服务获取路线。
5. 成功轨迹自动沉淀：根据缓存生成新 skill 草稿。
