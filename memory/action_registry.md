# 动作注册表 (Action Registry)

> 预封装常用动作，不靠临场拼步骤。

---

## action: github_repo_check

**用途**：检测 GitHub repo 是否可达

**触发条件**：需要访问 GitHub repo

**步骤**：
1. 构造 SSH URL：`git@github.com:${ORG}/${REPO}.git`
2. 执行：`git ls-remote ${SSH_URL} HEAD`
3. 如果成功 → repo 可达，继续 clone/fetch
4. 如果失败 → 报错，不自动降级

**验证**：exit code == 0

**失败处理**：告诉用户 "GitHub SSH 不可达，可能需要检查代理或网络"

---

## action: send_file_to_user

**用途**：通过 QQBot 给用户发送文件

**触发条件**：用户请求接收文件 / 需要交付文件

**步骤**：
1. 确认源文件存在：`ls -la <源路径>`
2. 复制到媒体目录：`cp <源路径> ~/.openclaw/media/qqbot/<文件名>`
3. 发送：在回复中用 `<qqmedia>/Users/lihongxin/.openclaw/media/qqbot/<文件名></qqmedia>`

**验证**：
- 文件存在媒体目录
- 大小 ≤ 100MB

**失败处理**：
- 文件不存在 → 告诉用户找不到
- 超过大小限制 → 告诉用户文件太大

---

## action: record_failure

**用途**：记录失败案例，防止复发

**触发条件**：任务失败 / 用户纠正 / 同类错误复发

**步骤**：
1. 打开 `memory/failure_cases.jsonl`
2. 追加 JSON 行：
   ```json
   {
     "timestamp": "ISO-8601",
     "task": "用户请求简述",
     "what_happened": "发生了什么错误",
     "root_cause": "根因分析",
     "fix_applied": "修复了什么",
     "verification": "如何验证修复有效",
     "regression_of": "如果是复发，标注原始记录 ID",
     "related_runbook": "关联的操作知识条目"
   }
   ```

**验证**：文件可读、格式正确

---

## action: run_benchmark

**用途**：跑固定评测任务，检查回归

**触发条件**：修改了 SOUL/AGENTS/TOOLS/runbook 之后

**步骤**：
1. 读取 `bench/openclaw_agent_tasks/tasks.yaml`
2. 逐个执行任务
3. 记录结果到 `memory/metrics/task_runs.jsonl`

**验证**：所有 P0 任务通过；P1 任务允许失败但需记录原因

---

## action: self_audit

**用途**：对自己进行客观审计

**触发条件**：定期（每周）/ 重大更新后 / 用户要求

**步骤**：
1. 用独立模型（如 GPT-5.5）审计
2. 审计内容包括：
   - 近期任务完成率
   - 用户纠错率
   - 同类错误复发率
   - 是否有新盲点
3. 写入审计报告

**验证**：审计报告必须有具体数据和 actionable 建议
