# APEX Verification Gate (local)

Purpose: reduce false improvement claims in each APEX self-improvement round.

Minimum evidence before metric increase:

1. **File evidence**: target log exists at `apex-self-improve/logs/round-<n>.md`.
2. **JSON evidence**: `apex-self-improve/state.json` parses as valid JSON after update.
3. **Content evidence**: log contains these labels:
   - `Order`
   - `Largest shortboard`
   - `Repair action`
   - `Verification evidence`
   - `Fact / Inference / Hypothesis`
4. **Conservative scoring**: do not increase a metric unless a concrete local artifact or successful validation directly supports that metric.

This gate is intentionally local-only and read-only/verifiable after creation.
