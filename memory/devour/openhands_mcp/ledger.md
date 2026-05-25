# First Devour: OpenHands-style Sandbox + MCP CLI Bridge

## Objective

Build the first real devour cycle for `OpenHands-style sandbox execution + MCP CLI bridge`.

## Candidate Evidence

Verified with GitHub SSH `git ls-remote`:

| Repo | Commit | Evidence |
|---|---|---|
| `All-Hands-AI/OpenHands` | `5e311f7f995008ffe4c74f8cf6f3085d4030c670` | `git ls-remote git@github.com:All-Hands-AI/OpenHands.git HEAD` |
| `modelcontextprotocol/python-sdk` | `e8e64842781c66b613872cf394de6e7d6f6925bf` | `git ls-remote git@github.com:modelcontextprotocol/python-sdk.git HEAD` |
| `modelcontextprotocol/servers` | `b1e1eb1a55847e0dcf78deb8ee821e2e46150a47` | `git ls-remote git@github.com:modelcontextprotocol/servers.git HEAD` |

Stars are intentionally omitted because web search failed in this runtime and no verified star count was retrieved. No virtual data.

## Installed Source

Sparse shallow fetch over SSH:

- `vendor/github/All-Hands-AI/OpenHands`
- `vendor/github/modelcontextprotocol/python-sdk`
- `vendor/github/modelcontextprotocol/servers`

Tracked snapshot:

- `third_party/openhands_mcp/snapshot`

## Local Reimplementation

- Skill: `skills/apex-harness-bridge/SKILL.md`
- CLI: `scripts/apex-harness-bridge/apex-harness-bridge`
- Source: `scripts/apex-harness-bridge/main.go`

## Verification

```bash
cd scripts/apex-harness-bridge
go build -o apex-harness-bridge .
./apex-harness-bridge --mode selftest
```

Selftest result: `go version go1.26.2 darwin/arm64`, exit code 0.

## Safety

The bridge is workspace-bounded, command-whitelisted, blocks dangerous tokens, enforces timeout, and returns JSON evidence.
