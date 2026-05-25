# Web-Failure Fallback Gate

Created: 2026-05-24T16:08:00+08:00
Round: 46

## Process bug

A read-only web/GitHub lookup is allowed once per round, but if it times out it can waste cycle time and tempt unsupported claims.

## Local repair

When the one permitted external read fails or times out:

1. Mark external benchmark as `skipped_with_error`.
2. Do not retry in the same round.
3. Use only local state/log evidence for metric changes.
4. Science mapping may use a standard formula from existing model knowledge, but must label `Fact / Inference / Hypothesis` and avoid claiming a fresh external source.
5. Do not improve metrics merely because a lookup was attempted.

## Claim budget

- External lookup failure can justify a process repair artifact.
- It cannot justify improved `xi_anti`, `t_cycle`, or `h_entropy` unless the log shows direct behavioral evidence.
- A small `epsilon_repair` increase is allowed only if the repair artifact is created and validated.
