# GitHub Resource Queue for APEX Improvement

Fetched/read-only checked with `git ls-remote` and README snapshots on 2026-05-24.

## 1. SWE-bench — princeton-nlp/SWE-bench

- Repo: https://github.com/princeton-nlp/SWE-bench
- HEAD observed: f7bbbb2ccdf479001d6467c9e34af59e44a840f9
- What it offers: benchmark for evaluating LLMs/agents on real-world GitHub issues; patch generation is verified in reproducible harnesses.
- APEX mapping:
  - Improves `ε_repair`: detect → patch → test.
  - Improves `ξ_anti`: claim success only when tests pass.
  - Improves `Φ_positive`: shifts from narrative improvement to benchmarked improvement.
- Local adaptation: create tiny SWE-style local tasks in workspace instead of pulling Docker images immediately.

## 2. Reflexion — noahshinn/reflexion

- Repo: https://github.com/noahshinn/reflexion
- HEAD observed: 218cf0ef1df84b05ce379dd4a8e47f17766733a0
- What it offers: verbal reinforcement learning via self-reflection over failed attempts.
- APEX mapping:
  - Improves `ε_repair`: stores failure reflection and retry strategy.
  - Improves `H_entropy/h_output_control`: reflection templates reduce drift.
- Local adaptation: every failed cron round must create: failure → cause → next-attempt constraint.

## 3. OpenAI Evals — openai/evals

- Repo: https://github.com/openai/evals
- HEAD observed: 8eac7a7de5215c907fbddc30efdaf316913eccdd
- What it offers: framework for custom evals and evaluation registries.
- APEX mapping:
  - Improves `H_entropy`: output criteria can be turned into pass/fail evals.
  - Improves `ξ_anti`: factuality and instruction-following evals.
- Local adaptation: define YAML/JSON-like local eval cases for fact/inference/hypothesis separation.

## 4. Inspect AI — UKGovernmentBEIS/inspect_ai

- Repo: https://github.com/UKGovernmentBEIS/inspect_ai
- HEAD observed: 02531a4ee59d49d1c747a339ed23feec9e8677ea
- What it offers: LLM evaluation framework with tool usage, multi-turn dialog, model-graded evaluations, 200+ evals.
- APEX mapping:
  - Improves `T_cycle`: structured task/eval/scorer separation.
  - Improves `ξ_anti`: independent scorer concept.
- Local adaptation: mirror its task/solver/scorer pattern in APEX logs.

## 5. DeepEval — confident-ai/deepeval

- Repo: https://github.com/confident-ai/deepeval
- HEAD observed: ec8ea666f6d8e1b4f689aeee3a1898631ac5d220
- What it offers: pytest-like LLM evaluation framework with metrics such as G-Eval, task completion, answer relevancy, hallucination.
- APEX mapping:
  - Improves `H_entropy`: add answer relevancy / hallucination checks.
  - Improves `Φ_positive`: task completion metric.
- Local adaptation: create simple local JSON evals before installing any dependency.

## Recommended immediate integration

Do not clone/install yet. First implement a lightweight local APEX eval harness:

1. `apex-self-improve/evals/output_control_cases.json`
2. `apex-self-improve/evals/repair_cases.json`
3. `apex-self-improve/run_local_eval.py`
4. Metrics gates:
   - raise `H_entropy` only if output-control cases pass
   - raise `ε_repair` only if repair cases pass
   - lower `T_cycle` only if same quality passes with fewer steps/tokens

## Safety

- Read-only GitHub use unless explicitly authorized.
- No unknown code execution.
- No external writes.
