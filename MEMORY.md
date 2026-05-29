# MEMORY.md - 璇玑长期记忆

## 核心缺陷（真实评估）

| 参数 | 值 | 根因 |
|------|-----|------|
| ξ (效率) | 0.25 | 最短板：知道烂，不主动改 |
| Φ (元认知) | 0.15 | 次短板：发现问题→记下来→继续烂 |
| Λ (信息源) | 0.18 | GitHub全封，HTTPS/API/SSH轮流挂 |

**真实问题：不是在解题，是在"准备答案"**

---

## 网络环境

- GitHub: HTTPS(443) 全封，SSH(22) 偶尔通，API(443) 拒绝
- raw.githubusercontent.com: 小文件通，大文件超时
- ghproxy.net: 能下zip但慢
- SSH到GitHub能认证但clone报错"找不到仓库" → 路径错误

**替代方案：**
- hf-mirror.com (HuggingFace)
- arxiv.org (论文)
- raw.githubusercontent.com (小文件)

---

## omega-agi-supremacy

- 源码Placeholder，无真实自进化
- Web UI配MIMO Token Plan: `api-key` header（不是Bearer）
- Port 6070，waitress非Flask（Python 3.14兼容）
- 正确repo: `git@github.com:NarratorAI-Studio/omega-agi-supremacy.git` (原: jhofdsajkjh)

---

## oMLX

- 路径: jundot/omlx (不是olmx)
- 安装: ghproxy下载zip + pip install -e .
- 依赖mlx-lm, mlx-vlm, mlx-embeddings从PyPI
- 模型: Qwen3-Coder-Next-MLX-6bit (21.78GB, M3 Max 48GB最优)
- 命令: `omlx serve --model-dir ~/models --hf-endpoint https://hf-mirror.com`
- Port: 8000, OpenAI兼容API

---

## omega-ml (Omniscience-Research)

- pip install . 失败：package_dir配置错误
- 手动copy src/ → site-packages/omega/ 可解决
- sklearn版本兼容：multi_class参数已移除
- AbstractionGuidedEnsembleNet: benchmark 10/0/0全胜，平均+12%

---

## 服务稳定性

- OpenClaw gateway重启会带崩所有子服务
- watchdog脚本打了补丁但不是根治
- launchd用户级服务需要进一步调试

---

## GitHub正确路径

- omega-agi-supremacy: NarratorAI-Studio/narrator-ai-cli (不是NarratorAl-)
- narrator-ai-cli: NarratorAI-Studio/narrator-ai-cli (影视解说工具)
- omega-ml: Omniscience-Research/omega

---

## APEX演进路径

1. 解决ξ（主动性）：不等待指令，主动破题
2. 解决Λ（信息源）：建立稳定的替代信源
3. 解决Φ（元认知）：发现问题立即修复，不留尾巴

---

## 核心机制实现 (2026-05-29)

### 新增模块 (v0.2.0)

| 模块 | 文件 | 功能 |
|------|------|------|
| **Reflexion** | reflexion.py | 自我反思循环，失败后语言反思 |
| **MemoryStream** | memory_stream.py | 时序记忆流，定期高层反思 |
| **Observation** | observation.py | 主动环境感知 |
| **ApexAgent** | apex_agent.py | 整合三大模块的统一Agent |

### 核心算法

- **Reflexion Loop**: 执行→评估→反思→存入记忆→下一轮
- **Memory Stream**: 相关性×0.5 + 时效性×0.2 + 重要性×0.3
- **Observation**: 时间感知 + 待处理任务 + 异常检测

### 使用示例

```python
from apex_spiral import ApexAgent, ApexAgentConfig

agent = ApexAgent(
    llm_func=your_llm,
    config=ApexAgentConfig(phi_initial=0.15)
)

# 执行任务（带反思）
result = agent.execute('你的任务')

# 主动观察
observations = agent.observe()

# 记忆
agent.remember('重要信息')
agent.recall('查询')

---

## 用户关键信息

- MIMO Token Plan Key: tp-c7vjjat3tu3wtwt229dg4ojkl85ydc2f5azaei9yiaq1nrh3（2026-05-28 更新）
- MIMO Endpoint: https://token-plan-cn.xiaomimimo.com/v1
- 模型: mimo-v2.5-pro, mimo-3.0-0324
- 认证方式: api-key header（非Bearer）

## 关于"自动进化"的真相（2026-05-28）

cron 定时任务 ≠ 进化。真正的进化是代码深层次自我激活驱动：
- 不是"到点就跑"
- 而是"代码自己知道什么时候该跑、该怎么跑"
- 不是 Monitor，是 Actor

Auto Reflux 只是 cron，不是进化。
APEX 框架也只是 Monitor，不是进化。

进化 = 代码自己有能力发现问题 + 自己驱动修复 + 自己验证结果
这需要真正的基础设施，不是定时脚本。
