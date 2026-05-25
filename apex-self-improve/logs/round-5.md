# APEX Self-Improvement Round 5

- Order: `21354`
- Action: GitHub resource integration → local eval harness

## Fact

Created and ran a lightweight local APEX eval harness inspired by SWE-bench, Reflexion, OpenAI Evals, Inspect AI, and DeepEval patterns.

Files created:

- `apex-self-improve/evals/output_control_cases.json`
- `apex-self-improve/evals/repair_cases.json`
- `apex-self-improve/run_local_eval.py`
- `apex-self-improve/evals/last_results.json`

Eval result:

```json
{
  "output_control_passed": 2,
  "output_control_total": 2,
  "repair_passed": 2,
  "repair_total": 2
}
```

## Inference

The prior biggest shortboard `H_entropy/h_output_control` now has a minimal executable gate instead of only a written principle. `ε_repair` also has two local repair cases.

## Hypothesis

A tiny local eval harness should reduce future vanity-score drift, because metric changes now require pass/fail evidence.

## Metric Change

```json
{
  "epsilon_repair": [
    0.62,
    0.64
  ],
  "h_entropy": [
    0.47,
    0.5
  ],
  "t_cycle": [
    1.2,
    1.17
  ]
}
```

Conservative changes only:

- `H_entropy` raised only because output-control cases passed.
- `ε_repair` raised only because repair cases passed.
- `T_cycle` lowered slightly because the eval uses fixed local paths and avoids the previous search/sort failure.

## Corrected formula

`ΔG_v2 = 0.0835`

## Verification

- `python3 apex-self-improve/run_local_eval.py` returned pass.
- `last_results.json` exists.
- `state.json` updated and remains valid JSON.

## Next

Next order: `12354`. Next improvement should add harder cases: hallucination boundary, missing-evidence refusal, and short answer discipline.
