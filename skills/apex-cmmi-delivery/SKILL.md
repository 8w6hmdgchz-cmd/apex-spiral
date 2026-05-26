---
name: apex-cmmi-delivery
description: APEX CMMI industrial delivery harness. Use for complex engineering work requiring formula planning, implementation slot, PR audit, automated tests, evidence gates, and GitHub release sync.
metadata: { "openclaw": { "emoji": "🧬", "requires": { "bins": ["go", "git"] } } }
---

# APEX CMMI Delivery Skill

## Formula

```text
Apex_CMMI = Apex_agent × (Plan → Code → Audit → Test → Release) × EvidenceGate
```

## Industrial Flow

1. **APEX Formula Planning**
   - Owner: GPT / phasor planner
   - Gate: phasor route + twelve-factor gate

2. **Claude-Code Implementation Slot**
   - Owner: coding agent
   - Gate: git diff + build/test
   - Note: if Claude Code is unavailable, use local coding agent/toolchain and mark the report honestly.

3. **APEX PR Audit**
   - Owner: GPT/APEX reviewer
   - Gate: fusion + evidence validator

4. **Automated Test Closure**
   - Owner: harness
   - Gate: ECC cycle + 12factor + hygiene

5. **GitHub Release Sync**
   - Owner: governance
   - Gate: safe rebase/push

## Container Rule

Docker is preferred for background isolation when available. If Docker is not installed or unavailable, the harness must report:

```text
container_mode = unavailable_local_sandbox
```

No fake container success is allowed.

## CLI

```bash
cd /Users/lihongxin/.openclaw/workspace/scripts/apex-cmmi-delivery

go build -o apex-cmmi-delivery .

./apex-cmmi-delivery --mode cycle \
  --root /Users/lihongxin/.openclaw/workspace \
  --task "deliver APEX RuntimeOS upgrade" \
  --out /Users/lihongxin/.openclaw/workspace/state/apex-cmmi-delivery-latest.json
```

## Non-Negotiables

- No fabricated metrics.
- No unverified memory admission.
- No destructive action without explicit approval.
- No GitHub push unless tests/gates pass or failure is explicitly documented.
