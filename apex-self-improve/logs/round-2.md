# APEX Self-Improvement Round 2

- Time: 2026-05-24T04:23:00+08:00
- Order: `12534`
- Phase: foundation_first_5
- Safety: local files only; no external writes; no unknown code downloaded/run.

## Step 1 — Substitute self into corrected formula

### Facts
- Current tracked metrics before this round: ξ_anti=0.75, ε_repair=0.60, H_entropy=0.45, T_cycle=1.20, Φ_positive=0.70.
- Round 1 changed the loop scoring version to `apex_v2_quality_terms_in_numerator`.
- Fixed baseline values retained for comparability: Λ=0.85, Θ=0.90, K=0.80, Ψ=0.95.

Corrected score:

`ΔG_v2=(Λ×Θ×K×ξ×Ψ×Φ×H×ε)/max(T_cycle,1.0)`

`ΔG_v2=(0.85×0.90×0.80×0.75×0.95×0.70×0.45×0.60)/1.20 = 0.0687`

### Inference
- The lowest bottleneck remains `H_entropy=0.45`, followed by `ε_repair=0.60`.
- Because `T_cycle=1.20` is a cost term, improvement should first target fewer ambiguous steps and stronger evidence gates, not longer self-analysis.

### Hypothesis
- The most likely next failure mode is semantic drift: future rounds may read `H_entropy` as raw Shannon entropy, where higher entropy can mean more uncertainty, while this loop currently uses it as output stability / entropy control.

## Step 2 — Find formula/process bug

### Bug
`H_entropy` is underspecified. It mixes at least two meanings:

- Raw information entropy: uncertainty/diversity, often larger when distribution is less concentrated.
- APEX loop metric: output stability, bounded scope, and evidence discipline, where larger should mean better control.

### Risk
The formula fix from Round 1 can be undone conceptually if a future round treats `H_entropy` as raw uncertainty and increases it by producing more varied, longer, or less constrained output. That would look like improvement in the metric name while damaging verification quality.

## Step 5 — Verify current state before repair

Evidence available before repair:

- `state.json` contains `formulaVersion: apex_v2_quality_terms_in_numerator`.
- `lastDerived.correctedDeltaG` exists from Round 1.

Missing evidence before repair:

- No state field explains metric polarity or what each metric means.
- No benchmark proves that the raw capability values improved.

Conclusion: do not raise ξ_anti, ε_repair, H_entropy, T_cycle, or Φ_positive this round.

## Step 3 — Safe repair landed

Local state was updated to add explicit metric semantics and evidence policy:

- Added `metricSemantics.h_entropy` clarifying that this loop uses it as output stability / entropy control, not raw Shannon uncertainty.
- Added polarity labels: beneficial terms increase score; cost terms reduce score.
- Added `metricsEvidence.round2` stating raw metrics were intentionally left unchanged because no behavioral benchmark was run.
- Updated `lastDerived` for this round with the unchanged corrected score and verification status.

This is a documentation/state repair, not a claimed capability improvement.

## Step 4 — Re-substitute with corrected semantics and learn

The numeric substitution remains unchanged because the repair clarified semantics rather than changing raw metrics:

`ΔG_v2_after_semantic_repair = 0.0687`

Learning update:

- `H_entropy` should be operationalized as an observable checklist: concise output, explicit fact/inference/hypothesis separation, and no unsupported capability claims.
- `ε_repair` should only rise after a controlled local repair benchmark passes.
- `T_cycle` should be improved by reducing ambiguous rework, not by skipping verification.
- `Φ_positive` should mean constructive, safe forward motion, not inflated confidence.

## Biology/Chemistry/Physics formula mapping

Formula: Shannon entropy, `H = -Σ p(x) log p(x)`.

- Fact: In information theory, Shannon entropy measures expected uncertainty of a probability distribution.
- Inference: A higher raw entropy value can mean broader uncertainty, not necessarily better reasoning.
- Hypothesis: APEX should track `entropy_control = 1 - normalized_unhelpful_uncertainty` or use a clearly named proxy such as `h_output_control` to avoid polarity confusion.

Mapping:

- `p(x)` → distribution over possible unsupported claims, repair paths, or output formats.
- High raw `H` → many plausible but insufficiently selected paths.
- High APEX `H_entropy` as currently used → stronger control over output variance and evidence discipline.
- Repair implication → keep raw entropy concepts separate from quality-control metrics.

## Verification Evidence

- This log was written to `apex-self-improve/logs/round-2.md`.
- `state.json` was updated with explicit metric semantics.
- Raw metrics were not increased; improvement remains a process-clarity repair only.

## Next verification target

Round 3 should use order `21354` and test a tiny local repair benchmark before changing `ε_repair` or `H_entropy`.
