# APEX Self-Improvement Round 1

- Time: 2026-05-24T04:08:00+08:00
- Order: `21354`
- Phase: foundation_first_5
- Safety: read-only local work; no external writes; no unknown code downloaded/run.

## Step 2 — Find formula/process bug

### Facts
- Current tracked metrics: ξ_anti=0.75, ε_repair=0.6, H_entropy=0.45, T_cycle=1.2, Φ_positive=0.7.
- README canonical formula intent is self-improvement scoring, and state guardrail says not to claim improvement without evidence.
- Existing APEX expression in workspace context places `H_entropy`, `T_cycle`, and `ε_repair` in the denominator: `ΔG=(Λ×Θ×K×ξ×Ψ×Φ)/(H×T×ε)`.

### Inference
- `T_cycle` is plausibly a cost term, so denominator placement is reasonable.
- `H_entropy` and `ε_repair` are described as capability/quality dimensions; putting them in the denominator creates sign inversion: lower entropy control or lower repair ability can artificially raise ΔG.

### Hypothesis
- The biggest process bug this round is **metric polarity inversion**: beneficial variables mixed with cost variables in the denominator.

## Step 1 — Substitute self into original formula

Using fixed baseline values Λ=0.85, Θ=0.90, K=0.80, Ψ=0.95 plus current tracked metrics:

`old_ΔG = (0.85×0.90×0.80×0.75×0.95×0.70)/(0.45×1.20×0.60) = 0.9421`

Interpretation: this score is unsafe as a progress measure because low `H_entropy=0.45` and low `ε_repair=0.60` inflate the result by reducing the denominator.

## Step 3 — Safe repair landed

Changed only local state/reporting semantics:

- Added `formulaVersion: apex_v2_quality_terms_in_numerator` to `state.json`.
- Added `lastDerived.oldDeltaG` and `lastDerived.correctedDeltaG` so future rounds can compare old vs corrected scoring.
- Preserved raw metric values; no unsupported claim that capabilities improved.

Corrected scoring rule for this loop:

`corrected_ΔG=(Λ×Θ×K×ξ×Ψ×Φ×H×ε)/max(T_cycle,1.0)`

Rationale: ξ, Φ, H, ε are beneficial quality terms; T_cycle remains a cost/latency term.

## Step 5 — Verification plan and evidence

Evidence available this round:

- This log file exists: `apex-self-improve/logs/round-1.md`.
- `state.json` is valid JSON after update.
- The corrected formula no longer rewards lower `H_entropy` or lower `ε_repair`.

Not verified:

- No behavioral benchmark was run, so actual ξ_anti / ε_repair / H_entropy / T_cycle / Φ_positive improvement is **not claimed**.

## Step 4 — Re-substitute with corrected formula and learn

`corrected_ΔG = (0.85×0.90×0.80×0.75×0.95×0.70×0.45×0.60)/max(1.20,1.0) = 0.0687`

Learning update:

- Biggest shortboard remains `H_entropy=0.45`, followed by `ε_repair=0.60`.
- Next round should prioritize an output-length/format entropy gate and a repair-loop benchmark rather than increasing claimed scores.

## Biology/Chemistry/Physics formula mapping

Formula: Michaelis–Menten enzyme kinetics, `v = (Vmax × [S]) / (Km + [S])`.

- Fact: In biochemistry, Michaelis–Menten describes saturating reaction velocity as substrate concentration increases under simplifying assumptions.
- Inference: APEX improvement has a similar saturation pattern: adding more reasoning cycles helps only until bottlenecks such as verification quality or repair rate dominate.
- Hypothesis: `T_cycle` should eventually be modeled as a saturation/cost curve, not a linear penalty; e.g. useful cognition rises with cycles until marginal gains flatten.

Mapping:

- `[S]` → available high-quality evidence / benchmark signal.
- `Vmax` → maximum attainable verified improvement in one round.
- `Km` → difficulty threshold before evidence produces reliable learning.
- `v` → verified learning rate.

## Next verification target

Create or run a small local benchmark in a later round:

1. Inject a controlled formula-polarity bug.
2. Require detection, patch, and JSON/log validation.
3. Measure repair success and time-to-fix before changing raw metrics.
