---
name: browser
description: Natural-language browser automation. Inspired by hyperbrowserai/HyperAgent: `page.ai("click the blue button")` style. Multi-driver (Playwright, headless Chrome, MCP browser server).
trigger: "open the browser", "click on", "navigate to", "scrape", "fill the form"
priority: high
tier: do
depends-on: []
---

# browser

> **You should never say "open a browser" — you should say what you want done in the browser.**

## When To Use

- The user wants a web page opened, scraped, filled, clicked, or tested.
- A web app needs end-to-end verification.
- A workflow involves a UI that the LLM cannot see directly.

## The HyperAgent Pattern

From `hyperbrowserai/HyperAgent`:

```python
page = await browser.new_page()
await page.goto("https://example.com")
await page.ai("find the 'Sign up' button and click it")
await page.ai("fill the email field with 'foo@bar.com'")
await page.ai("wait for the success message and copy its text")
```

The model never writes CSS selectors — it says what it wants, the agent translates to selectors.

## APEX-SKILL Wrapper

The `browser` skill in APEX-SKILL is a thin shell that:

1. Spawns a Playwright/Chrome driver (configurable, defaults to Playwright).
2. Exposes a small CLI:

```bash
python3 scripts/browser.py "open https://example.com and copy the H1"
python3 scripts/browser.py "click the 'Sign up' button"
python3 scripts/browser.py "fill the form with name=foo, email=foo@bar.com"
python3 scripts/browser.py "screenshot to /tmp/page.png"
```

3. Returns a structured result:

```yaml
browser_op:
  op: ai
  prompt: "..."
  result: {text: "...", screenshot: "/tmp/page.png"}
  steps:
    - {kind: goto, url: "...", ok: true}
    - {kind: click, selector: "button.signup", ok: true}
    - {kind: extract, selector: "h1", text: "..."}
```

## Safety Rails

- **Read-only by default.** The driver refuses to fill forms / click submit unless `--allow-write` is set.
- **Domain allowlist.** Editable in `~/.apex-skill/browser-allowlist.txt`. Default: `localhost`, `127.0.0.1`, `*.example.com`.
- **Timeout.** 30s default; 5min for `--allow-write` flows.
- **Audit.** Every browser op is logged to `~/.apex-skill/logs/telemetry.jsonl` with the prompt.

## MCP-Native

If the host platform supports MCP, `browser` also exposes an MCP server (`apex-browser`) that hosts can call directly.

## Anti-Patterns

- ❌ Don't paste CSS selectors into the prompt (defeats the abstraction).
- ❌ Don't scrape sites that prohibit it (check `robots.txt`).
- ❌ Don't fill forms without `--allow-write` (safety).
- ❌ Don't loop click+retry — debug with the `debug` skill instead.

## Output Contract

```yaml
browser_op:
  op: ai | screenshot | goto | click | fill | extract
  prompt: <string>
  result: {text?, screenshot?, html?, ...}
  steps: [...]
  next_skill: <caller's choice>
```
