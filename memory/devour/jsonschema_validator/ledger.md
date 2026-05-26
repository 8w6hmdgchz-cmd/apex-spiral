# Devour: jsonschema → APEX Evidence Validator

## Objective

把 PaperQA 的“有证据才入库”变成硬门禁：任何要进入 Σ_memory 的吞噬声明，必须有 source repo、commit、source path、context id、score、verification evidence。

## Source Evidence

- Repo: `https://github.com/python-jsonschema/jsonschema.git`
- Commit: `c6dc09209c0224af200f0663c52cecaea063de36`
- Local snapshot: `vendor/github/python-jsonschema/jsonschema`

Inspected files:

- `jsonschema/validators.py`
- `jsonschema/exceptions.py`
- `jsonschema/cli.py`
- `pyproject.toml`

## Distilled Pattern

jsonschema contributes the hard-gate pattern:

1. **Schema first**
   - validate instances against an explicit schema before accepting them.

2. **Validator selection / draft awareness**
   - `validator_for` chooses validator based on schema dialect.
   - APEX adaptation uses an explicit Draft 2020-12 schema file.

3. **Structured error paths**
   - Validation errors expose instance paths / JSON paths.
   - APEX adaptation reports `$[index].field` paths for failed evidence.

4. **Remote reference caution**
   - jsonschema warns that automatic remote reference retrieval can be unsafe.
   - APEX schema avoids remote refs and uses local schema only.

5. **CLI validation pattern**
   - load schema/instance
   - report parse/file/validation errors separately
   - exit non-zero on validation failure.

## Local Reimplementation

Created:

- `schemas/apex-evidence.schema.json`
- `scripts/apex-evidence-validator/main.go`
- `scripts/apex-evidence-validator/go.mod`
- `scripts/apex-evidence-validator/apex-evidence-validator`
- `state/apex-evidence-selftest.json`
- `state/apex-evidence-validator-latest.json`
- `state/apex-evidence-devour-samples.json`
- `state/apex-evidence-validator-devour-report.json`

Hard gate fields:

```text
id
claim
source_repo
source_commit
source_path
context_id
score >= 0.70
verification.command
verification.result == pass
verification.evidence_path
memory_type optional enum
```

Safety rules:

- source/evidence paths must be relative
- no `..`
- commit must be 7-40 hex
- verification must pass before memory admission
- score below 0.70 blocks admission

## Verification

```bash
cd scripts/apex-evidence-validator
go build -o apex-evidence-validator .
./apex-evidence-validator --mode selftest --input state/apex-evidence-selftest.json --out state/apex-evidence-validator-latest.json
./scripts/apex-evidence-validator/apex-evidence-validator --mode validate --input state/apex-evidence-devour-samples.json --out state/apex-evidence-validator-devour-report.json
```

Results:

```text
selftest: success, checked 1, passed 1
devour samples: success, checked 2, passed 2
```

## PHI Impact

This is a real gate, not a ledger-only improvement:

- **Working**: evidence records are machine-validated before memory admission.
- **Procedural**: validator selftest and devour-sample validation are repeatable.
- **Ω_dawn**: reduces uncontrolled/ungrounded memory writes.
- **Σ_memory**: future additions can be accepted/rejected by schema, not vibes.

## Next Integration

- Make `apex-eval-harness` optionally require an evidence JSON for every passed task.
- Make memory injection scripts call `apex-evidence-validator` before writing `state/sigma_memory.json`.
- Add rejected evidence to `memory/failure_cases.jsonl`.
