# GitHub Secrets 设置指南

## 需要在GitHub网页设置的内容

### 1. 设置 Secrets（Settings → Secrets and variables → Actions）

| Secret名称 | 值 | 说明 |
|-----------|---|------|
| `GH_SSH_KEY` | （见下方） | GitHub部署密钥 |
| `OPENAI_API_KEY` | 你的GPT-5.5 API key | GPT修复者用 |

**生成GH_SSH_KEY步骤：**
```bash
# 1. 生成新SSH密钥（用于Actions）
ssh-keygen -t ed25519 -C "actions@apex-spiral" -f ~/.ssh/actions_key

# 2. 添加到GitHub (Settings → SSH and GPG keys → New SSH key)
cat ~/.ssh/actions_key.pub
# 粘贴到GitHub

# 3. 添加到Secrets
cat ~/.ssh/actions_key
# Settings → Secrets → New secret → GH_SSH_KEY
```

### 2. 启用GitHub Pages
- Settings → Pages → Source: Deploy from branch main

### 3. 启用Actions权限
- Settings → Actions → General → Workflow permissions: Read and write

### 4. 添加Telegram告警（可选）
- 创建Bot: https://t.me/BotFather
- 获取Chat ID: https://t.me/useridbot
- 设置Secrets: `TELEGRAM_BOT_TOKEN` 和 `TELEGRAM_CHAT_ID`
