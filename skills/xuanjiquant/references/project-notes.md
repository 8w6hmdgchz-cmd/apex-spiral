# XuanjiQuant Project Notes

Original project files live at:

`/Users/lihongxin/.openclaw/workspace/XuanjiQuant-main`

## Project Claim / Positioning

- Localized quantitative database using Tencent Finance API.
- Local execution and private data storage.
- APEX formula system for autonomous review, parameter adjustment, and evolution.
- Daily pre-market sync, closing review, strategy evolution.

## Configuration Notes

- Requires Tencent Finance API permission if using original intended data source.
- Data should be stored locally.
- Validation standards mentioned by project:
  - Market correlation ≥ 0.8
  - Qualified strong/weak stock return spread
  - No algorithm error before execution

## Practical Caution

The repository currently contains formula/architecture markdown, not a full runnable engine. Treat it as a framework/spec unless executable scripts and API credentials are added.
