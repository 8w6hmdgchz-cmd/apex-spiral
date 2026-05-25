# APEX Unified Research Engine MVP

三合一科研统一引擎：

```text
Engine_APEX = (Coord_Fix × Token_Control)
            × (Task_Syn + Train_SFT/RL + Bench_Verify)
            × (ERA + CoScientist + Robin)
```

## Modules

- UI Control: reuses `../apex_token_rs` Rust tests for coordinate/token control health.
- Local Training: reuses `../clawg-mvp/scripts/run_iteration.sh` for Task_APEX → Bench → Score_APEX.
- Autonomous Research: MVP implementations of ERA, Co-Scientist, and Robin write local artifacts.

## Run

```bash
./scripts/run_engine.sh "your research question"
```

Report:

```text
reports/latest_report.json
reports/<trace_id>/engine_report.json
```

## Current boundary

This is an MVP orchestration scaffold. LLM judging, literature retrieval, real code sandboxing, and database persistence are next-stage integrations.
