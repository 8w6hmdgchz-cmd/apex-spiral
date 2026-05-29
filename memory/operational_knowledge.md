# APEX 系统状态 (2026-05-29 17:32 GMT+8)

## ΔG 体系参数快照

| 参数 | 符号 | 当前值 | 状态 |
|------|------|--------|------|
| Delta G (终值) | ΔG | **0.672823** | ⚠️ < 0.7 |
| Evolution Score | ES | **0.5737** | ⚠️ < 0.6 |
| PHI 比值 | Φ% | **57.37%** | ⚠️ < 60% |
| 瓶颈 | Σ_memory | 0.378 | 🔴 最短板 |

**公式**: `ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)`
**进化公式**: `ES = ΔG / (ΔG + H)` → 反推 **H = 0.5000**

### 各维度当前值

| 维度 | 参数 | 值 | 来源 |
|------|------|-----|------|
| LLM Agent 路由效率 | Θ (theta) | 0.612 | metrics store |
| 代码掌握度 | K (k_master) | 1.107 | metrics store |
| 自修复速度 | ε (epsilon) | 1.053 | 基线 fallback |
| 反馈循环强度 | Φ (phi_cycle) | 1.284 | 基线 fallback |
| 宿主系统健康 | Ψ (psi_host) | 0.7176 | omega_dawn |
| 记忆归一化质量 | Σ (sigma_memory) | **0.378** | 🔴 最低 |
| 经验衰减率 | ξ (tau_trace) | 0.98 | full_mirror |
| Lambda Root | Λ | ~1.0 | 默认 |
| 时间阻力 | T | ~1.0 | 默认 |
| 熵/惯性 | H | **0.5000** | 反推 |

### 记忆层统计

| 类型 | 数量 | 占比 |
|------|------|------|
| Working | 72 | 16.1% |
| Semantic | 118 | 26.5% |
| Episodic | 177 | 39.7% | ← 最多
| Procedural | 79 | 17.7% |
| **总计** | **446** | |

### DAG / Skillflow

- DAG 节点: 28 | 边: 44
- 多峰路由: 11 | 轨迹路由: 14
- 进化技能数: 17 | 积分分配: 61
- 验证通过: 14/14 (100%)
- 崩溃检测: 无

### 趋势判断

- 瓶颈明确为 **Σ_memory (0.378)** — 记忆压缩/遗忘太快
- retention_threshold = 0.372，learn_rate = 0.928，decay_factor = 0.988
- τ_trace = 0.98 接近饱和，记忆追踪效率高但整合效率低
- **当前状态**: 需优化记忆压缩 + 提高 Episodic 层整合

---

# 操作知识库 (Operational Runbook)

> 每个条目：触发条件 → 正确动作 → 验证
> 执行前先查，不靠临场记忆。

---

## GitHub 资源拉取

**触发条件**：需要从 GitHub 获取 repo/资源
**错误历史**：HTTPS/API (`api.github.com`) 在中国大陆可能被阻断，之前反复踩坑

### 正确流程

1. 先用 SSH 检测 repo 是否可达：
   ```bash
   git ls-remote git@github.com:ORG/REPO.git HEAD
   ```
2. SSH 成功 → clone/fetch
3. SSH 失败 → 报"GitHub 不可达"，不要自动降级 HTTPS，先告诉用户

**验证命令**：
```bash
git ls-remote git@github.com:mem0ai/mem0.git HEAD
```

**不要做**：
- ❌ 不要用 `https://api.github.com` 或 `https://github.com`（可能被阻断）
- ❌ 不要用 `gh` CLI（依赖 API）
- ❌ 不要自动 fallback 到 HTTPS（死路）

---

## QQBot 文件发送

**触发条件**：需要给用户发送文件（非图片/非语音）

### 正确流程

1. 确认文件存在
2. 复制到媒体目录：
   ```bash
   cp <源路径> ~/.openclaw/media/qqbot/<文件名>
   ```
3. 在回复中使用 `<qqmedia>` 标签：
   ```
   <qqmedia>~/.openclaw/media/qqbot/<文件名></qqmedia>
   ```
   或使用绝对路径：
   ```
   <qqmedia>/Users/lihongxin/.openclaw/media/qqbot/<文件名></qqmedia>
   ```

**验证命令**：
```bash
ls -la ~/.openclaw/media/qqbot/
```

**限制**：
- 图片 ≤ 30MB
- 文件 ≤ 100MB
- 视频 ≤ 100MB
- 语音 ≤ 20MB

---

## 子代理调用准则

**触发条件**：需要执行任务

### 规则

1. **简单任务（< 3步，单一领域）→ 自己直接做**
   - 文件读写、搜索、单次工具调用
   - 不要为了"协调者原则"而滥用子代理
2. **复杂任务（多步、多领域、需独立审查）→ 开子代理**
   - 必须写清楚验收标准
   - 子代理输出必须经过主 agent 验证
3. **审计/客观评估 → 用不同模型**
   - 对自己开智体系的审计 → GPT-5.5
   - 第三方评审 → 独立模型

**子代理不是逃避直接执行的借口。**

---

## 失败记录

**触发条件**：任务失败 / 用户纠正 / 同错复发

**正确流程**：
1. 打开 `memory/failure_cases.jsonl`
2. 追加一条：时间、请求、输出、根因、修复、验证
3. 如果复发了，在 regression_of 字段标注原记录

---

## 新功能/新工具

**触发条件**：遇到第一次做的操作

**流程**：
1. 做完后立即写入 `operational_knowledge.md`
2. 如果是重复性操作，同时加进 `action_registry.md`
3. 下次别再查一遍

## MLX模型管理（2026-05-28补）

### 下载前 checklist
1. `df -h` 查磁盘余量
2. `python3 -c "from safetensors import safe_open"` 验证工具
3. 查 config.json 确认内存需求（model_size_bytes + overhead < 可用内存）
4. `curl -sI --max-time 5 <url>` 确认 CDN 可达（不follow重定向）
5. 确认 tokenizer.json 不是 Git LFS（tokenizer.json 空 = 需要单独下载）

### 模型验证命令
```bash
python3 -c "
from safetensors import safe_open
import os
for f in os.listdir('.'):
    if f.endswith('.safetensors'):
        try:
            with safe_open(f, framework='mlx') as sf:
                list(sf.keys())
            print(f'OK: {f}')
        except:
            print(f'CORRUPT: {f}')
"
```

### hf-mirror CDN 路径
- 小文件(≈<1MB): `raw/` 路径直接返回内容
- 大文件(≈>1MB): `resolve/` 返回302 → xethub.hf.co（常封）
- 备选: `modelers.cn API` 查其他镜像

### 常见问题
- safetensors报错"offset out of range" → 文件截断，需重新下载
- Metal内存超限 → 查 `iogpu.wired_limit_mb`，或换更小的量化版本
- tokenizer.json 空 → 是Git LFS指针，需单独 curl resolve/ 下载
