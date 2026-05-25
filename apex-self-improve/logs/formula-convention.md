# APEX Formula Convention Note

Bug repaired: `H_entropy` was used ambiguously.

- In the original ΔG denominator, `H` behaves like an entropy/cost term: lower is better.
- In `state.json.metrics.h_entropy`, the value is tracked like a capability score: higher is better.

Local convention from this round onward:

- Keep `metrics.h_entropy` as **h_output_control_score** semantics: higher = better output control / lower uncontrolled entropy.
- For ΔG calculation, use `H_cost = 1 / h_entropy_score` when substituting into the denominator, or equivalently multiply by `h_entropy_score` in the numerator.
- Do not compare old and new ΔG values as identical formulas unless this convention is stated.

This prevents falsely treating a better output-control score as worse formula performance.
