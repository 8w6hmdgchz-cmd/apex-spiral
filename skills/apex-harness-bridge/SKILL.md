---
name: apex-harness-bridge
description: APEX harness bridge for OpenHands-style sandbox execution and MCP-like CLI JSON calls. Use when a task needs safe local command execution with JSON request/response, workspace boundary checks, timeouts, and evidence logs.
metadata: { "openclaw": { "emoji": "🛠️", "requires": { "bins": ["go", "git"] } } }
---

# APEX Harness Bridge Skill

## Source Devoured

Installed via GitHub SSH sparse fetch:

| Repo | Commit | Role |
|---|---|---|
| `All-Hands-AI/OpenHands` | `5e311f7f995008ffe4c74f8cf6f3085d4030c670` | sandbox execution / agent loop reference |
| `modelcontextprotocol/python-sdk` | `e8e64842781c66b613872cf394de6e7d6f6925bf` | MCP protocol reference |
| `modelcontextprotocol/servers` | `b1e1eb1a55847e0dcf78deb8ee821e2e46150a47` | MCP server/tool reference |

Tracked snapshot: `/Users/lihongxin/.openclaw/workspace/third_party/openhands_mcp/snapshot`

## Local Reimplementation

Go CLI:

```bash
/Users/lihongxin/.openclaw/workspace/scripts/apex-harness-bridge/apex-harness-bridge
```

This is a local bounded bridge, not a blind copy of upstream code.

## Protocol

```json
{
  "protocol": "apex-cli-mcp/v1",
  "action": "sandbox.exec",
  "tool": "go",
  "args": ["version"],
  "cwd": "/Users/lihongxin/.openclaw/workspace",
  "timeout_seconds": 10
}
```

Actions:

- `sandbox.exec`: safe local command execution
- `mcp.exec`: same JSON transport shape for MCP-style CLI tool calls

Safety:

- `cwd` must stay under `/Users/lihongxin/.openclaw/workspace`
- allowed tool whitelist only
- blocks dangerous tokens such as `rm`, `sudo`, `dd`, `shutdown`
- timeout enforced per request
- JSON response includes evidence, stdout, stderr, exit code, duration

## Verification

```bash
cd scripts/apex-harness-bridge
go build -o apex-harness-bridge .
./apex-harness-bridge --mode selftest
./apex-harness-bridge --mode schema
```
