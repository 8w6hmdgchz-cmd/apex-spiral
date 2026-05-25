# PraisonAI GitHub Installation Ledger

## Installed Source

- Repo: `git@github.com:MervinPraison/PraisonAI.git`
- Commit: `8acf77c531e624c46d3d61dcae37e9942e90972c`
- Local path: `/Users/lihongxin/.openclaw/workspace/vendor/github/MervinPraison/PraisonAI`
- Tracked snapshot: `/Users/lihongxin/.openclaw/workspace/third_party/praisonai/snapshot`
- Method: SSH sparse shallow fetch

## Command Used

```bash
mkdir -p vendor/github/MervinPraison/PraisonAI
cd vendor/github/MervinPraison/PraisonAI
git init
git remote add origin git@github.com:MervinPraison/PraisonAI.git
git config core.sparseCheckout true
printf 'README*\npyproject.toml\nsetup.py\npraisonaiagents/**\npraisonai/**\nsrc/**\n' > .git/info/sparse-checkout
GIT_SSH_COMMAND="ssh -o ConnectTimeout=20 -o ServerAliveInterval=10 -o ServerAliveCountMax=6" \
  perl -e 'alarm 300; exec @ARGV' \
  git fetch --depth=1 --no-tags --filter=blob:none origin HEAD
git checkout FETCH_HEAD
git rev-parse HEAD > .openclaw-source
```

## Why Sparse Fetch

Full `git clone --depth 1 --filter=blob:none` stalled in this runtime. GitHub SSH `git archive --remote` is not supported. Sparse shallow fetch successfully installed the GitHub source snapshot while avoiding the clone hang.

## Upstream Features Observed

README/source snapshot contains:

- `Agent` primitive
- `Agents` multi-agent primitive
- MCP/tool integration
- background tasks
- workflows / AgentFlow
- handoffs
- guardrails, memory, knowledge, cron/dashboard concepts

## Local Skill Activated

- Skill: `/Users/lihongxin/.openclaw/workspace/skills/apex-praison-chain/SKILL.md`
- Go helper: `/Users/lihongxin/.openclaw/workspace/scripts/apex-praison-chain/apex-praison-chain`
- Formula: `ApexPraisonChain = RoleAgents × TaskGraph × ProcessMode × ToolGate × VerifyLoop × MemLedger`
