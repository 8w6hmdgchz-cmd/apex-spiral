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

## CMMI Industrial Delivery Harness

Added:

- `scripts/apex-cmmi-delivery/main.go`
- `skills/apex-cmmi-delivery/SKILL.md`
- `state/apex-cmmi-delivery-latest.json`

Formula:

```text
Apex_CMMI = Apex_agent × (Plan → Code → Audit → Test → Release) × EvidenceGate
```

Cycle result:

```text
status: success
container_mode: unavailable_local_sandbox
docker_available: false
P1 APEX Formula Planning: pass
P2 Claude-Code Implementation Slot: pass
P3 APEX PR Audit: pass
P4 Automated Test Closure: pass
P5 GitHub Release Sync: pass
```

Docker boundary: Docker is not currently available in PATH, so the harness honestly marks local sandbox fallback instead of pretending container execution.

## Evidence-backed Memory Admission

Added:

- `scripts/apex-memory-admission/main.go`
- `state/apex-memory-admission-latest.json`
- `state/apex-memory-admission-evidence-report.json`

Flow:

```text
fusion evidence → apex-evidence-validator → sigma_memory admission → PHI full mirror
```

CMMI delivery now has six phases:

```text
Plan → Code → Audit → Test → Memory → Release
```

Latest verification:

```text
memory_admission_status: success
validated: true
sigma_memory: 0.3687
cmmi_status: success
P1..P6: pass
```

## Container Backend Detection

Added:

- `scripts/apex-container-backend/main.go`
- `state/apex-container-backend-latest.json`

Behavior:

```text
Docker present + daemon reachable → docker_isolated
Docker missing/unreachable → local_sandbox_fallback
```

Latest verification:

```text
status: success
mode: local_sandbox_fallback
docker_available: false
```

CMMI delivery now starts with container backend detection:

```text
Container → Plan → Code → Audit → Test → Memory → Release
```

## Claude Code Verified Runner

Added:

- `scripts/apex-claude-code-runner/main.go`
- `state/apex-claude-code-runner-latest.json`

Detected local coding backends:

```text
claude: available, 2.1.128 (Claude Code)
claude-code: available
codex: available, codex-cli 0.131.0
```

Safety boundary:

```text
detect/selftest never mutates source
real coding must run under CMMI gates
runner must output diff/test evidence before PR audit
no direct push from coding slot
```

CMMI formula now uses:

```text
Container → Plan → ClaudeCode → Audit → Test → Memory → Release
```

## Release Manager

Added:

- `scripts/apex-release-manager/main.go`
- `state/apex-release-manager-latest.json`
- `releases/<version>/RELEASE_NOTES.md`
- `releases/<version>/ROLLBACK.json`

Behavior:

```text
CMMI gates → version → release notes → SHA256 checksums → rollback manifest
```

Boundary:

```text
publish_mode: prepared_local_no_external_release
```

External GitHub Release publishing still requires explicit permission/tooling; local artifact preparation is automatic.

## Memory Admission v2

Upgraded `apex-memory-admission` to format `apex-memory-admission-2.0`.

New gates:

```text
quality_floor: 0.72
content dedupe: enabled
capacity: 500
failure/rollback/repair claim → Procedural memory
rank = importance + bounded access_count bonus
```

Latest verification:

```text
status: success
validated: true
added: 0
skipped/deduped: duplicate fusion evidence was not re-admitted
memory_count: 426
sigma_memory: 0.3716
```

This prevents Σ_memory inflation from repeated identical evidence.
