---
name: rtk
description: Token-compression shell wrapper. Reduces stdout by 60-90% via filters, dedup, and structure-preserving summaries. Inspired by rtk-ai/rtk.
trigger: any tool returning >1KB output, verbose logs, large diffs
priority: medium
tier: do
depends-on: []
---

# rtk

> **Most of the bytes in tool output are noise. The model only needs the signal.**

## When To Use

- A shell command (npm install, cargo build, docker pull, git log, rg ...) returns >1KB.
- The output has repetitive lines (progress bars, timestamps, "Downloading...").
- The output contains a known pattern (test failures, error stacks) that can be summarized.

## The 8 Filters (rtk-inspired)

| Filter | Pattern | Replaces with |
|---|---|---|
| `--dedup` | consecutive duplicate lines | `[... N duplicates]` |
| `--progress` | `Downloading X/Y` / `Compiling foo v0.1.0` / `[==>` | `[progress: X/Y (Z%)]` |
| `--errors-only` | verbose logs | only ERROR/WARN/FATAL lines + 2 lines context |
| `--no-timestamps` | lines starting with ISO date | stripped |
| `--truncate-paths` | absolute paths to repo root | relative paths |
| `--summarize-json` | JSON > 5KB | schema + 3 sample values + size |
| `--summarize-stack` | Rust/JS stack trace | first frame + last frame + "N more" |
| `--top N` | ranked output (rg --count, ps aux) | top N + "[N more]" |

## Usage

### As a transparent hook

The `pre-tool-use.sh` hook auto-suggests rtk for known-noisy commands. The model opts in by re-running via:

```bash
./scripts/rtk.sh <original command> [--filter ...]
```

### As an explicit wrapper

```bash
# All-in-one (auto-pick filters)
./scripts/rtk.sh npm install

# Specific filter
./scripts/rtk.sh --errors-only cargo test

# Combined
./scripts/rtk.sh --dedup --progress --top 20 ps aux
```

## Implementation

`scripts/rtk.sh` is a thin shell wrapper that:
1. Forwards the command to the real binary.
2. Captures stdout.
3. Pipes through Python filters in `scripts/rtk_filters.py`.
4. Prints compressed output + a one-line summary.

```python
# scripts/rtk_filters.py (sketch)
import sys, re

def dedup(lines):
    out, prev, count = [], None, 1
    for line in lines:
        if line == prev:
            count += 1
        else:
            if count > 1:
                out.append(f"  [... {count-1} duplicates]\n")
            out.append(line)
            count = 1
        prev = line
    return out
# ... progress, errors-only, etc.
```

## Anti-Patterns

- ❌ Don't rtk the output of a command that already has structured output (JSON, YAML).
- ❌ Don't rtk error output you don't understand (you'll hide the bug).
- ❌ Don't rtk diffs that you're about to review (you need full fidelity).
- ❌ Don't rtk output of `git status` (it's already terse).

## Output Contract

```yaml
rtk:
  input_bytes: <int>
  output_bytes: <int>
  ratio: <float 0..1>
  filters_applied: [...]
  summary: <one-line>
  next_skill: <caller's choice>
```
