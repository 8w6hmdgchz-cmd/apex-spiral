# APEX ECC RuntimeOS

## Purpose

ECC is the controlled Agent Harness layer for long-running engineering tasks.

It upgrades APEX toward AI Agent Infrastructure through:

- Runtime
- Plugin Architecture
- Skills
- Memory
- Hooks
- Rules
- Multi-agent
- Session State
- Security
- Observability
- Governance
- Learning

## First Activation

Created and verified:

- `scripts/apex-ecc-runtimeos/main.go`
- `skills/apex-ecc-runtimeos/SKILL.md`
- `scripts/apex_ecc_nightly.sh`
- `state/apex-ecc-runtimeos-latest.json`

Gate result:

```text
status: success
fusion_ok: true
evidence_ok: true
security_ok: true
domains: 10/10 active
```

## Governance

- No destructive operations without explicit approval.
- No fabricated metrics; PHI must read full_mirror artifacts.
- Every upgrade must pass fusion + evidence + hygiene gates.
- Nightly work must be incremental, committed, and reversible.

## Nightly Command

```bash
/Users/lihongxin/.openclaw/workspace/scripts/apex_ecc_nightly.sh
```

## Boundary

This is not a claim of autonomous unrestricted AGI. It is a controlled local runtime harness that can evolve code only through audited gates and repository commits.

## Phasor LLM Layer

Added local phasor router:

- `scripts/apex-phasor-llm/main.go`
- `state/apex-phasor-llm-latest.json`

Purpose:

```text
task vector × model vector × quantum-router × twelve-factor gate → selected LLM + fallbacks
```

Selftest result:

```text
status: success
selected: zai/glm-5-turbo
fallbacks: deepseek/deepseek-v4-pro, zai/glm-5v-turbo, zai/glm-5.1, zhipuai/glm-5-flash
alignment: 0.911
twelve_factor_gate: present
```

Boundary: this is local route optimization, not provider config mutation. It does not expose or edit API keys.
