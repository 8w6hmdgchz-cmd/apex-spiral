# Emv trajectories

## 2026-05-23 initial fusion
- Source: 超级进化4-上下文学习新框架 attachment.
- Pattern: Challenger/Reasoner/Judge self-play + cross-time replay + entropy/Gini mutation selection.
- Integration: OpenClaw skill metadata loads trigger; references store SkillBank and trajectories; Rust crate implements deterministic scoring core.

## Integrated: APEX Enlightenment Failure Replay

Source: `apex-enlightenment/state/consistency_log.jsonl`  
Converted dataset: `skills/emv-entropy-skill/references/enlightenment-failure-replay.jsonl`

Purpose:
- train/evaluate EMV against single-path high-confidence mistakes
- down-weight `[ERROR] curl failed` paths instead of pretending full consensus
- flag `confidence=1.0` with `total_paths=1` as a hallucination-risk pattern

Records converted: 47

Key rule extracted:
> self-consistency requires successful independent paths; one surviving path is availability, not consensus.

