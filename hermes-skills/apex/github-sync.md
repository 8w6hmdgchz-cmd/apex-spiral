---
name: apex-github-sync
description: APEX GitHub同步 - evolver状态上报与数据同步
version: 1.0.0
platforms: [macos, linux]
metadata:
  hermes:
    tags: [apex, github, sync]
    category: apex
    requires_toolsets: [terminal, github]
---

# APEX GitHub Sync - GitHub 同步

## When to Use
- evolver 运行后自动同步
- 手动触发：`git push`
- 检查 evolver 状态时

## Procedure

### GitHub 同步流程

```bash
# 1. 添加更改
git add -A

# 2. 提交
git commit -m "描述"

# 3. 推送（需要 SSH key）
git push origin main

# 4. 验证
git log --oneline -3
```

### SSH Key 配置

```bash
# SSH key 位置
~/.ssh/id_ed25519

# 测试连接
ssh -T git@github.com

# 确认 key
ssh-add -l
```

### Evolver 同步指标

```bash
# evolver 状态
cat score-state.env

# 关键字段
AWAKE=8.1
PSI_SELF=7.5
GAMMA=6.0
PHI_RATIO=1.051
BUG_CODE=B4
```

## GitHub Actions

```yaml
# .github/workflows/evolver.yml
on:
  schedule:
    - cron: '*/15 * * * *'  # 每15分钟
```

## Pitfalls

- **SSH key 问题**：公钥当私钥用，无法 git push
- **不验证推送**：git push 成功但不检查
- **冲突不处理**：多进程同时推送导致冲突

## Verification

执行后确认：
1. `git log` 有新提交
2. GitHub repo 页面能看到更新
3. evolver 状态同步
