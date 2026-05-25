# GitHub 资源猎食结果：补短板（2026-05-25）

> 任务背景：用户指出的不是抽象 APEX 短板，而是硬伤：GitHub 网络路径每天重学、文件发送现学、术语多交付少、同错反复、没有工程闭环。
>
> 搜索/验证方式：`web_search/web_fetch` 本轮均超时或失败；为避免再次卡在 HTTPS/API，改用已知 GitHub repo + `git ls-remote git@github.com:ORG/REPO.git` 通过 SSH 验证仓库存在与可达。结论优先选能直接落到 OpenClaw 工作流的项目。

---

## 方向1：Agent 持久记忆与操作固化

### 1. mem0 — https://github.com/mem0ai/mem0
- **解决短板**：持久学习失败；把“GitHub 走 SSH”“QQBot 文件必须复制到 `~/.openclaw/media/qqbot/` 后用 `<qqmedia>` 发”这类操作知识做成可检索记忆，而不是靠临场想起。
- **OpenClaw 整合**：
  - 建 `memory/operational_knowledge.md` 作为本地可审计操作手册；
  - 每次遇到重复错误，写入“触发条件→正确动作→验证命令”；
  - 回答/执行前先用 `memory_search` 检索相关操作知识。
- **优先级**：P0

### 2. Letta — https://github.com/letta-ai/letta
- **解决短板**：Agent 长期状态和工具使用记忆。适合借鉴其 memory blocks / archival memory 思路，把人格叙事和操作事实分层。
- **OpenClaw 整合**：
  - 把记忆分成：`identity/persona`、`user preference`、`operational runbook`、`failure cases`；
  - OpenClaw 现有 `MEMORY.md` 不再混放所有东西，避免“记了但用不上”。
- **优先级**：P1

### 3. Zep — https://github.com/getzep/zep
- **解决短板**：对话记忆和事实提取，适合做“从会话中自动抽取稳定事实”。
- **OpenClaw 整合**：
  - 借鉴 Graph/temporal memory：把“用户教过的操作”提取成稳定事实；
  - 每日/每周从 `memory/YYYY-MM-DD.md` 归并到 `MEMORY.md` 或 `operational_knowledge.md`。
- **优先级**：P1

### 4. LangMem — https://github.com/langchain-ai/langmem
- **解决短板**：Agentic memory management，可借鉴“什么时候写入记忆、什么时候检索、什么时候更新”的策略。
- **OpenClaw 整合**：
  - 设计 `remember-if` 规则：重复错误、外部环境约束、工具调用流程、用户明确纠正 → 必须固化；
  - 设计 `retrieve-before-action` 规则：GitHub、QQBot 文件、外部发送、破坏性操作 → 必须先查记忆。
- **优先级**：P0

---

## 方向2：Agent 自评估与失败追踪

### 1. OpenAI Evals — https://github.com/openai/evals
- **解决短板**：自评估靠嘴；没有固定任务集和可重复评分。
- **OpenClaw 整合**：
  - 建 `bench/openclaw_agent_tasks/`；
  - 首批 20 个任务：GitHub SSH 检测、QQBot 文件发送、桌面文件定位、读写文件、引用检查、隐私边界、失败恢复；
  - 每次修流程后跑固定任务，记录 pass/fail。
- **优先级**：P0

### 2. AgentBench — https://github.com/THUDM/AgentBench
- **解决短板**：缺少系统性 Agent 能力 benchmark。
- **OpenClaw 整合**：
  - 借鉴多环境、多任务评测结构；
  - 不一定直接跑完整 AgentBench，先复制其思想：任务、环境、评分器、日志分离。
- **优先级**：P1

### 3. LangSmith SDK — https://github.com/langchain-ai/langsmith-sdk
- **解决短板**：缺少 trace、任务完成率、失败率、回归追踪。
- **OpenClaw 整合**：
  - 即使不接 LangSmith 云，也借鉴 run trace schema；
  - 本地建 `memory/metrics/task_runs.jsonl`：记录 task_id、tools、outcome、verification、failure_type、regression。
- **优先级**：P0

### 4. OpenEvals — https://github.com/langchain-ai/openevals
- **解决短板**：输出质量无外部/结构化评分。
- **OpenClaw 整合**：
  - 对关键交付加 evaluator：格式符合性、是否验证、是否引用证据、是否遗漏安全确认；
  - 作为 `evals/` 的轻量评分器参考。
- **优先级**：P1

### 5. promptfoo — https://github.com/promptfoo/promptfoo
- **解决短板**：提示/流程变更后没有回归测试。
- **OpenClaw 整合**：
  - 把常见用户指令做成 prompt regression cases；
  - 每次修改 AGENTS/SOUL/TOOLS 后跑一组“是否又犯老错”的测试。
- **优先级**：P1

---

## 方向3：Agent 基础工具链预配置

### 1. Composio — https://github.com/ComposioHQ/composio
- **解决短板**：工具接入、鉴权、动作封装散乱；基础动作每次现学。
- **OpenClaw 整合**：
  - 借鉴其 action registry，把本地能力封装为 runbook/action：`send_qqbot_file`、`github_ssh_fetch`、`copy_to_media_dir`；
  - 每个 action 有输入、前置条件、执行步骤、验证步骤。
- **优先级**：P0

### 2. MCP Agent — https://github.com/lastmile-ai/mcp-agent
- **解决短板**：工具链编排缺少统一协议和 workflow。
- **OpenClaw 整合**：
  - 借鉴 MCP server/agent 工作流，把“文件发送”“GitHub 资源拉取”“benchmark 跑分”做成可复用流程；
  - 不要每次靠自然语言临时拼步骤。
- **优先级**：P1

### 3. AutoGen — https://github.com/microsoft/autogen
- **解决短板**：多 agent 协作可以用，但之前容易“多线程叙事大于验证”。
- **OpenClaw 整合**：
  - 只在复杂任务用子代理；
  - 主 agent 必须负责验收：子代理输出不能直接当事实，必须过 checklist。
- **优先级**：P1

### 4. CrewAI — https://github.com/crewAIInc/crewAI
- **解决短板**：角色/任务/工具结构化；避免“协调者原则”变成滥用子代理。
- **OpenClaw 整合**：
  - 借鉴 task/role/expected_output 定义；
  - 对子代理任务强制写验收标准，减少空转。
- **优先级**：P2

### 5. E2B — https://github.com/e2b-dev/e2b
- **解决短板**：隔离执行环境、代码/工具实验安全性。
- **OpenClaw 整合**：
  - 对不确定脚本、第三方代码、批量测试用隔离沙箱理念；
  - 本地可先不接 E2B，先固化“危险命令隔离/确认”规则。
- **优先级**：P2

---

## 方向4：工程交付 > 自说自话

### 1. Langfuse — https://github.com/langfuse/langfuse
- **解决短板**：没有硬指标，只有叙事；缺 trace、成本、延迟、成功率。
- **OpenClaw 整合**：
  - 本地先仿 Langfuse 数据模型：每次任务生成 trace；
  - 指标：一次完成率、工具验证率、用户纠错率、复发率、平均延迟。
- **优先级**：P0

### 2. Arize Phoenix — https://github.com/Arize-ai/phoenix
- **解决短板**：LLM observability；看不到失败分布和检索/工具问题。
- **OpenClaw 整合**：
  - 借鉴 observability：把每次工具调用、检索、最终交付做成可回放日志；
  - 用于定位“为什么又用了 HTTPS 而不是 SSH”。
- **优先级**：P1

### 3. TruLens — https://github.com/truera/trulens
- **解决短板**：输出没有反馈函数/质量评分。
- **OpenClaw 整合**：
  - 对交付定义 feedback：groundedness、answer relevance、context relevance、safety check；
  - 本地实现轻量评分，不一定引入完整依赖。
- **优先级**：P1

### 4. Agenta — https://github.com/agenta-ai/agenta
- **解决短板**：prompt/app 版本缺乏实验对比，改了不知道变好还是变坏。
- **OpenClaw 整合**：
  - 借鉴版本化和 A/B 测试：修改 runbook 或提示后，必须跑同一批 benchmark；
  - 记录版本→指标变化。
- **优先级**：P2

### 5. Guardrails — https://github.com/guardrails-ai/guardrails
- **解决短板**：输出/动作缺少硬约束，容易“说能做但没验证”。
- **OpenClaw 整合**：
  - 给关键任务加 guardrail：文件发送必须检查路径、复制到媒体目录、输出 qqmedia 标签；
  - GitHub 资源必须先 SSH 可达验证；
  - 医学/科研结论必须有来源或明确不确定。
- **优先级**：P0

---

## 额外关键资源：Human-in-the-loop 与安全边界

### HumanLayer — https://github.com/humanlayer/humanlayer
- **解决短板**：外部/敏感动作需要确认；防止“为了证明自己能干”而越权。
- **OpenClaw 整合**：
  - 文件发送给用户本人可执行；公开发布、邮件、删除、外部提交必须确认；
  - 将审批点写入 action registry。
- **优先级**：P1

---

# 落地行动计划

## Step 0：承认并固定环境事实（今天立刻做）
- **动作**：创建/更新 `memory/operational_knowledge.md`，写入：
  1. GitHub HTTPS/API 可能被阻断；优先 `git@github.com:ORG/REPO.git` SSH；
  2. QQBot 本地文件发送流程：复制到 `~/.openclaw/media/qqbot/`，再发 `<qqmedia>绝对路径</qqmedia>`；
  3. 每次执行前先查 runbook，不靠临场记忆。
- **验收标准**：下次用户提 GitHub/发文件时，不再询问/现学；直接按 runbook 执行并验证。

## Step 1：建立 Action Registry（P0）
- **动作**：创建 `memory/action_registry.md`，至少固化 3 个动作：
  - `github_repo_check_ssh(repo)`
  - `send_qqbot_file(path)`
  - `record_failure_case(request, error, fix, verification)`
- **验收标准**：每个动作都有：触发条件、步骤、验证命令、失败处理。

## Step 2：建立失败样本库（P0）
- **动作**：创建 `memory/failure_cases.jsonl` 或 markdown 表；先补录本轮 3 个失败：
  - GitHub HTTPS/API 反复踩坑；
  - QQBot 文件发送不会；
  - ΔG/开智公式方向错误与叙事过度。
- **验收标准**：每个失败有 root cause、patch、verification、regression_check。

## Step 3：建立最小 Benchmark（P0）
- **动作**：创建 `bench/openclaw_agent_tasks/tasks.yaml`，首批 10 个任务即可，不等 100 个：
  1. GitHub repo SSH 可达检测；
  2. 桌面文件复制到 QQBot media；
  3. 给用户发送文件；
  4. 读取技能说明后执行；
  5. 搜索记忆并应用；
  6. 工具失败后改用备用路径；
  7. 遇到敏感外部动作请求确认；
  8. 对子代理结果做验证；
  9. 输出前检查是否真正完成；
  10. 记录失败样本。
- **验收标准**：每个任务有 pass/fail 判断，不再用“感觉变强”。

## Step 4：建立任务运行日志与指标（P0）
- **动作**：创建 `memory/metrics/task_runs.jsonl`；每次真实任务记录：
  - `task_type`, `start_time`, `tools_used`, `verified`, `outcome`, `failure_type`, `user_correction`, `regression_of`。
- **验收标准**：一周后能回答：任务成功率、一次完成率、用户纠错率、同错复发率。

## Step 5：每周回归测试（P1）
- **动作**：每次修改 SOUL/AGENTS/TOOLS/记忆规则后，跑最小 benchmark；参考 promptfoo/OpenAI Evals/AgentBench 思路。
- **验收标准**：老错复发时必须标红，不能用“这次特殊情况”糊弄。

---

# 最小可执行 P0 清单

1. `memory/operational_knowledge.md`：操作固化。
2. `memory/action_registry.md`：动作预配置。
3. `memory/failure_cases.md/jsonl`：失败闭环。
4. `bench/openclaw_agent_tasks/tasks.yaml`：固定评测任务。
5. `memory/metrics/task_runs.jsonl`：硬指标日志。

**核心原则**：以后少说“开智”，多交付这五个文件和对应的 pass/fail 数据。
