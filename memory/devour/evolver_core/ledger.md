# Second Devour: Evolver Core

## Objective

Break Evol_code bottleneck by adding a local observe→diagnose→patch-plan→verify core.

## Candidate Evidence

- `yoheinakajima/babyagi`: `fa8930ebe72a82e5ad57b356e7cbec96290e5bb2`, installed by SSH sparse fetch.
- `significant-gravitas/AutoGPT`: `127a0fa96a31076dbb297d585de2faacc0c7a890`, SSH HEAD verified; install blocked by fetch/checkout stall.
- `microsoft/autogen`: `027ecf0a379bcc1d09956d46d12d44a3ad9cee14`, SSH HEAD verified; install blocked by batch fetch stall.

No virtual stars or unverified benchmark numbers.

## Local Implementation

- `scripts/apex-evolver-core/main.go`
- `scripts/apex-evolver-core/apex-evolver-core`
- `skills/apex-evolver-core/SKILL.md`

## Verification

```bash
cd scripts/apex-evolver-core
go build -o apex-evolver-core .
./apex-evolver-core --mode cycle
```

The first cycle produced real findings from local files, including TODO markers and portable timeout risks.

## Third Devour: Findings → Patch → Clean Cycle

Changes:

- Implemented `SelfModEngine::save_state` persistence to `state/selfmod_history.json`.
- Hardened `apex-evolver-core` scanner:
  - skips binaries/executables/vendor/third_party/target/.git
  - ignores scanner self-definition lines
  - narrows timeout risk to shell-command patterns
  - avoids treating "no virtual data" policy docs as fake-data findings
- Replaced legacy GNU `timeout` in `scripts/crontab_config` with macOS-compatible `perl -e 'alarm ...'`.

Verification:

```bash
cd scripts/apex-evolver-core && go build -o apex-evolver-core .
./scripts/apex-evolver-core/apex-evolver-core --mode cycle
cd apex-ene/engine && cargo check
```

Result:

- Evolver findings: `null`
- Patch plan: `no_patch_needed`
- Rust selfmod cargo check: passed
