# Devour: Dawn Gate → Measured Ω_dawn Readiness

## Objective

继续打 PHI 瓶颈：`Ω_dawn` 不能只靠把 git 噪音剥离，还要把 dawn readiness 从静态常数变成真实门禁通过率。

## Local Reimplementation

Created:

- `scripts/apex-dawn-gate/main.go`
- `scripts/apex-dawn-gate/go.mod`
- `scripts/apex-dawn-gate/apex-dawn-gate`
- `state/apex-dawn-gate-latest.json`

`apex-dawn-gate` runs four local gates:

1. `apex-mini-executor --mode selftest`
2. `apex-eval-harness --mode selftest`
3. `apex-evidence-validator --mode selftest`
4. `apex-hygiene`

It computes:

```text
ready_score = passed / checked
auto_learn = ready_score * 0.92
```

This preserves V10.1's learning readiness concept while grounding it in executable evidence.

## Integration

Updated `scripts/auto_reflux.sh`:

- Keeps conservative formula fallback.
- If `scripts/apex-dawn-gate/apex-dawn-gate` exists and passes, uses its `auto_learn` value.
- If any dawn gate fails, keeps the conservative fallback and logs the failure.

## Verification

```bash
cd scripts/apex-dawn-gate
go build -o apex-dawn-gate .
./apex-dawn-gate --root /Users/lihongxin/.openclaw/workspace --out /Users/lihongxin/.openclaw/workspace/state/apex-dawn-gate-latest.json
```

Result:

```text
status: success
ready_score: 1.0
auto_learn: 0.92
passed: 4
checked: 4
```

## PHI Impact

Before this change, `auto_learn` was a static conservative value around `0.34`; after real gates passed it became `0.92`.

Latest mirror:

```text
Ω_dawn: 0.7176
PHI: 57.28%
bottleneck: Σ_memory
```

This is a real bottleneck shift: `Ω_dawn` is no longer the shortest pole after executor/eval/evidence/hygiene are all passing.
