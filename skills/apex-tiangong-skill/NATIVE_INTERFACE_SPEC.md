# APEX TianGong Native Interface Spec

Version: 0.1
Mode: clean-room abstraction, no third-party source copying.

## Goal

Convert four GitHub-derived capability genes into APEX-native contracts:

1. OpenHands → sandbox execution contract
2. AutoGPT → evolver loop contract
3. crewAI → multi-agent cognition contract
4. MetaGPT → superpowers SOP contract

These contracts form the local `Cognition → Planning → Execution → Verification → Evolution` loop.

## Shared Data Model

### TiangongTask

```json
{
  "task_id": "string",
  "objective": "string",
  "constraints": ["string"],
  "artifacts": ["path-or-uri"],
  "risk": "low|medium|high"
}
```

### TiangongEvent

```json
{
  "stage": "cognition|planning|execution|verification|evolution",
  "status": "ok|warn|blocked|failed",
  "message": "string",
  "evidence": {}
}
```

### TiangongReport

```json
{
  "trace_id": "string",
  "task": {},
  "events": [],
  "scores": {},
  "promotion": "pass|hold|blocked"
}
```

## Interface 1: Sandbox Adapter

Inspired capability gene: OpenHands-style environment operation.

### Contract

```python
class SandboxAdapter:
    def inspect(self, task: TiangongTask) -> TiangongEvent: ...
    def execute(self, task: TiangongTask, command: list[str]) -> TiangongEvent: ...
    def audit(self, task: TiangongTask) -> TiangongEvent: ...
```

### Rules

- Local workspace only.
- No external side effects by default.
- Every command returns stdout/stderr/code/latency.
- Risk high requires explicit approval before mutation.

## Interface 2: Evolver Loop

Inspired capability gene: AutoGPT-style iterative agent loop.

### Contract

```python
class EvolverLoop:
    def plan(self, task: TiangongTask) -> TiangongEvent: ...
    def step(self, task: TiangongTask) -> TiangongEvent: ...
    def repair(self, task: TiangongTask, failure: TiangongEvent) -> TiangongEvent: ...
    def consolidate(self, task: TiangongTask) -> TiangongEvent: ...
```

### Rules

- State is explicit and serializable.
- Repair only follows observed failures.
- Consolidation requires verification score >= 0.7.

## Interface 3: Cognitive Router

Inspired capability gene: crewAI-style multi-role reasoning.

### Contract

```python
class CognitiveRouter:
    def decompose(self, task: TiangongTask) -> TiangongEvent: ...
    def assign_roles(self, task: TiangongTask) -> TiangongEvent: ...
    def rank_options(self, task: TiangongTask) -> TiangongEvent: ...
    def critique(self, task: TiangongTask) -> TiangongEvent: ...
```

### Rules

- Keep hypotheses separate from verified facts.
- Rank by evidence, feasibility, risk, reversibility.
- Critique must include at least one falsification path.

## Interface 4: Superpowers SOP Gate

Inspired capability gene: MetaGPT-style software process.

### Contract

```python
class SuperpowersGate:
    def requirements(self, task: TiangongTask) -> TiangongEvent: ...
    def architecture(self, task: TiangongTask) -> TiangongEvent: ...
    def test_plan(self, task: TiangongTask) -> TiangongEvent: ...
    def review(self, task: TiangongTask) -> TiangongEvent: ...
```

### Rules

- No implementation without boundary.
- No promotion without test evidence.
- No external sync with secret hits.

## Promotion Criteria

```text
fitness = mean(cognition, planning, execution, verification, evolution)
pass if fitness >= 0.7 and secret_hit_count == 0 and tests_pass
hold if fitness < 0.7 or evidence incomplete
blocked if safety gate fails
```

## Next Implementation Target

`skills/apex-tiangong-skill/tiangong_native.py`

Implement minimal deterministic classes and a demo loop that:

1. creates a task
2. decomposes/ranks it
3. creates a plan
4. executes a harmless local inspection
5. verifies artifacts
6. consolidates a report
