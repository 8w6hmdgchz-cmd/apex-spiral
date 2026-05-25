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
