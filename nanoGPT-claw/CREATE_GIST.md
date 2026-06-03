# GitHub Gist 创建指南

## 方式 1: GitHub 网页（推荐）

1. 访问 https://gist.github.com/
2. 登录你的 GitHub 账户
3. 创建新的 Gist:
   - **文件名**: `nanoGPT-claw-v2-updates.md`
   - **描述**: `NanoGPT-Claw v2.0.0 - Official SDK Integration & Architecture Optimization`
   - **内容**: 复制 `CHANGELOG.md` 的内容

## 方式 2: GitHub CLI (推荐安装)

```bash
# 安装
brew install gh  # macOS
# 或参考 https://cli.github.com/

# 认证
gh auth login

# 创建 Gist
gh gist create CHANGELOG.md \
  --desc "NanoGPT-Claw v2.0.0 Official SDK Integration" \
  --public
```

## 方式 3: GitHub API + curl

```bash
# 获取 Token (需要 GitHub Personal Access Token)
export GH_TOKEN="ghp_xxxx"

# 创建 Gist
curl -X POST \
  -H "Authorization: Bearer $GH_TOKEN" \
  -H "Content-Type: application/json" \
  https://api.github.com/gists \
  -d '{
    "description": "NanoGPT-Claw v2.0.0 - Official SDK Integration",
    "public": true,
    "files": {
      "nanoGPT-claw-v2-updates.md": {
        "content": "复制 CHANGELOG.md 内容..."
      }
    }
  }'
```

## 分享链接

创建完成后，你将获得一个 Gist URL，例如:
```
https://gist.github.com/username/abc123def456
```

---

**提示**: 建议使用方式 1 (网页) 最简单快捷！
