# APEX Token Root Fix × OpenClaw Fusion

## Problem classes

1. Screenshot scaling causes physical click coordinate drift.
2. Single screenshot frames cost ~1000-1800 tokens and can overflow long tasks.
3. Unproductive reasoning/text loops waste compute effort.

## Core formulas

Coordinate correction:

`X_real = X_out * (W_screen / W_img)`

`Y_real = Y_out * (H_screen / H_img)`

Context budget:

`Token_reserve = Token_text + Σ Token_img(n), n=N-2..N`

Only the latest 3 image frames should be retained for vision-heavy interaction unless the task explicitly needs older frames.

Effort validity:

`Effort_valid = Total_effort - Waste_effort`

Efficiency:

`Effort_efficiency = Effort_valid / Total_effort`

## Implementation

Existing Go core:

- `skills/apex-core/apex_token_optimizer.go`
- binary: `skills/apex-core/apex_token_optimizer`

New Rust core:

- `crates/apex_token_optimizer/src/main.rs`

Rust CLI examples:

```bash
cargo run --manifest-path crates/apex_token_optimizer/Cargo.toml -- correct --x 100 --y 50 --sw 1920 --sh 1080 --iw 1000 --ih 500
cargo run --manifest-path crates/apex_token_optimizer/Cargo.toml -- reserve --text 100 --imgs 1000,1200,900,800 --keep 3
cargo run --manifest-path crates/apex_token_optimizer/Cargo.toml -- effort --total 100 --waste 25
cargo run --manifest-path crates/apex_token_optimizer/Cargo.toml -- purify --dir /tmp/screens --keep 3
```

## 25-step purification policy

Maintain a rotating cleanup cycle:

- steps 0,5,10,15,20: old screenshots
- steps 1,6,11,16,21: temp cache
- steps 2,7,12,17,22: stale conversation cache
- steps 3,8,13,18,23: duplicate screenshots
- steps 4,9,14,19,24: compressed morning/old frames

Current cron uses a practical daily smoke/purify entrypoint. Deeper OpenClaw internal cache mutation requires gateway/source-level hooks and should not be claimed unless implemented there.

## Runtime use policy

- Before clicking on a scaled screenshot, run coordinate correction.
- For image-heavy sessions, keep latest 3 frames and summarize older frames as text if still needed.
- Record avoidable waste patterns into APEX/SWRs memory only when they are reusable.
- Prefer Rust/Go/C for deterministic core; Python only as glue.

## Validation

`cargo test --manifest-path crates/apex_token_optimizer/Cargo.toml`

Expected: 3 tests pass.

## OpenClaw browser CLI hook

Patched files under `/opt/homebrew/lib/node_modules/openclaw/dist`:

- `browser-cli-actions-input-BQzZ_pkR.js`
  - `openclaw browser click-coords` now supports optional APEX correction.
  - Enable with `OPENCLAW_APEX_COORD_CORRECT=1`.
  - Required dimensions:
    - `OPENCLAW_APEX_SCREEN_W`
    - `OPENCLAW_APEX_SCREEN_H`
    - `OPENCLAW_APEX_IMAGE_W`
    - `OPENCLAW_APEX_IMAGE_H`
- `browser-cli-inspect-MK7-CJBe.js`
  - `openclaw browser screenshot` records screenshot paths to `memory/apex-screenshot-frames.jsonl` unless `OPENCLAW_APEX_SCREENSHOT_RECORD=0`.

Example:

```bash
OPENCLAW_APEX_COORD_CORRECT=1 \
OPENCLAW_APEX_SCREEN_W=1920 OPENCLAW_APEX_SCREEN_H=1080 \
OPENCLAW_APEX_IMAGE_W=1000 OPENCLAW_APEX_IMAGE_H=500 \
openclaw browser click-coords 100 50
```

This maps to real click `(192,108)` before sending the browser action.

Backups were created beside the patched dist files with `.bak-apex-<timestamp>` suffix.

## Verification note

`node --check` passed for both patched dist files.

Runtime command `openclaw browser click-coords --help` is currently blocked by plugin policy:

`plugins.allow excludes "browser"`

So the browser hook is installed but cannot be exercised until the browser plugin is allowed. The standalone Rust formula test still verifies the correction `(100,50) -> (192,108)` for `1920x1080 / 1000x500`.

## QQBot approval lesson

QQBot native approvals are accepted with `/bot-approve`, not the generic `/approve` command. For local CLI device scope upgrades, `openclaw devices list` reveals pending requests and `openclaw devices approve <requestId>` can approve after explicit user authorization.

## Latest-3 screenshot ring

The browser screenshot hook now maintains:

- append-only audit log: `memory/apex-screenshot-frames.jsonl`
- bounded latest-three index: `memory/apex-screenshot-latest3.json`

After four screenshot calls, `apex-screenshot-latest3.json` was verified to contain exactly 3 entries. This is the safe context-control layer: it limits what future agents should load while preserving original media files unless a separate explicit cleanup policy removes them.
