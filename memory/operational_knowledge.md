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
