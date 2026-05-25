# Integrated from `apex-github-evolution/policies/safety.md` at 2026-05-23 15:34:23 +0800

# APEX GitHub Evolution Safety Policy

Goal: use Git/GitHub/Gist as an auditable backup and review channel for APEX evolution artifacts.

Hard rules:
1. No automatic external push/commit/gist without explicit user approval.
2. Never include API keys, tokens, credentials, channel secrets, personal chat logs, or private memory by default.
3. Prefer local reports first; external sync is a second step.
4. Every exported artifact must pass a secret scan.
5. Cron jobs may generate local reports; external writes require manual confirmation.

Allowed by default:
- local git status audit
- local diff summary
- local evomap manifest
- local report generation
- local tar/git bundle excluding sensitive files

Requires explicit confirmation:
- gh repo create
- git push
- gh gist create/edit
- public issue/PR/comment
- scheduled external publishing
