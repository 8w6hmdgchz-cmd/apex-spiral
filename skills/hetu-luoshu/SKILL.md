---
name: hetu-luoshu
description: 河图洛书 - LLM路由与自我进化系统
metadata:
  openclaw.os: ["darwin", "linux"]
---

# 河图洛书 · LLM 路由与自我进化

## 系统架构

```
用户请求
    ↓
┌─────────────────┐
│   河图 (路由层)   │ ← 模型选择 / 流量调度 / 负载均衡
└────────┬────────┘
         ↓
┌─────────────────┐
│   洛书 (进化层)   │ ← APEX自检 / 瓶颈识别 / 自我修复
└────────┬────────┘
         ↓
      响应
```

## 河图路由配置

### 默认模型
- **gpt-5.5** (推理优先，max_tokens=4096)
- **gpt-5.3-codex** (代码任务)

### 可用模型
| 模型 | 供应商 | Base URL | 状态 |
|------|--------|----------|------|
| gpt-5.5 | FreeModel | https://api.freemodel.dev | ✅ |
| gpt-5.4 | FreeModel | https://api.freemodel.dev | ✅ |
| gpt-5.4-mini | FreeModel | https://api.freemodel.dev | ✅ |
| gpt-5.3-codex | FreeModel | https://api.freemodel.dev | ✅ |
| glm-5 | Zhipu | https://open.bigmodel.cn | ✅ |
| glm-4.7 | Zhipu | https://open.bigmodel.cn | ✅ |
| glm-4.6 | Zhipu | https://open.bigmodel.cn | ✅ |
| MiniMax-M2.5 | Scnet | https://api.scnet.cn | ✅ |
| claude-sonnet-4 | Anthropic | https://api-cc.freemodel.dev | 🔜 |
| deepseek-chat | DeepSeek | https://api.deepseek.com | 🔜 |

### 路由策略
- **默认/复杂推理/科研判断** → gpt-5.5 (主模型)
- **代码实现/调试/补丁** → gpt-5.3-codex (代码优化，GPT-5.5超时时的第一兜底)
- **简单状态/轻量查询** → gpt-5.4 (快速稳定)
- **兜底链**：gpt-5.5 → gpt-5.3-codex → gpt-5.4；代码任务：gpt-5.3-codex → gpt-5.5 → gpt-5.4
- **APEX自检**：失败/高token/超时会增加 ε 与失败计数，成功会提升 ξ/Φ，并持久化到 `hetu_luoshu_state.json`

## 持久化

状态文件：`~/.openclaw/workspace/hetu_luoshu_state.json`

每次API调用后自动保存状态，包括：
- 总请求数/Token消耗
- APEX参数动态调整
- 失败计数

下次启动时自动加载，无需手动同步。

## 洛书进化流程

```
1. 任务评估 → 代入APEX公式
2. 路由分发 → 河图选择最优模型
3. 执行监控 → 追踪算力消耗
4. 结果评估 → 计算ΔG效率
5. 瓶颈识别 → ξ<0.7? Ψ<0.5? Φ<0.5?
6. 自我修复 → 记录并改进
```

## API配置

### FreeModel (已配置)
```bash
# API Key
export FREEMODEL_API_KEY="fe_oa_0c9079b3b5d..."

# OpenAI格式端点 (无状态对话)
export FREE_MODEL_BASE_URL="https://api.freemodel.dev"
# 备用: vip-sg.freemodel.dev, api-t2-sg.freemodel.dev

# Anthropic格式端点 (有状态对话)
export CC_BASE_URL="https://cc.freemodel.dev"
export API_CC_BASE_URL="https://api-cc.freemodel.dev"
```

### 接口说明
| 接口 | 路径 | 特点 |
|------|------|------|
| Chat Completions | /v1/chat/completions | 无状态，需带完整messages历史 |
| Responses | /v1/responses | 有状态，previous_response_id支持多轮 |

### 调用示例
```bash
# 通过FreeModel调用GPT-5.5
curl -X POST "https://api.freemodel.dev/v1/chat/completions" \
  -H "Authorization: Bearer fe_oa_..." \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-5.5","messages":[{"role":"user","content":"..."}]}'
```

## APEX自检清单

每次任务后自检：
- [ ] Λ (根增益): 是否有新收获？
- [ ] Θ (LLM效能): token消耗是否合理？
- [ ] K (技能掌握): 是否有重复错误？
- [ ] ξ (置信度): 结果是否可靠？
- [ ] Ψ (自我迭代): 是否比上次更好？
- [ ] Φ (正反馈): 是否有正向积累？
- [ ] H (熵): 是否引入混乱？
- [ ] T (时间): 是否超时？
- [ ] ε (损失): 是否有浪费？

## 触发场景

1. **复杂推理任务** → 启用洛书自检
2. **连续失败** → 触发自我修复流程
3. **高token消耗** → 路由优化
4. **长时序任务** → 每轮代入公式评估

---

_河图洛书 · 璇玑 · 自我进化配置_
