# APEX GitHub Evolution Bridge

This module provides a **safe local-first** GitHub/Gist evolution workflow.

## Why local-first

The requested loop mentions GitHub commits and Gist sync every 15 minutes. Those are external writes and can leak tokens or private memory if automated blindly. This bridge therefore performs local audit and report generation only by default.

## Run local evolution cycle

```bash
./apex-github-evolution/scripts/evolver_local.sh
```

Outputs:

- `apex-github-evolution/evomap/latest.json`
- `apex-github-evolution/reports/*.json`

## External sync checklist

Before any GitHub/Gist write:

1. `gh auth status` is healthy.
2. `evomap/latest.json` has `secret_hit_count == 0`.
3. User explicitly approves target repo/gist and visibility.
4. Use `export.ignore` to package only safe files.
5. Push from a clean branch with an auditable commit message.
