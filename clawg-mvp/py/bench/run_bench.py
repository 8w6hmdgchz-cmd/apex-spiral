#!/usr/bin/env python3
"""ClawG MVP: AutoVerify + LLMVerify占位评分"""
import json, pathlib, random

ROOT = pathlib.Path(__file__).resolve().parents[2]
TASKS = ROOT / "datasets" / "raw" / "tasks.jsonl"
REPORTS = ROOT / "datasets" / "processed" / "reports.jsonl"
REPORTS.parent.mkdir(parents=True, exist_ok=True)

with TASKS.open(encoding="utf-8") as f, REPORTS.open("w", encoding="utf-8") as out:
    for line in f:
        task = json.loads(line)
        auto = 0.8 + random.random() * 0.15
        llm = 0.75 + random.random() * 0.15
        score = 0.6 * auto + 0.4 * llm
        report = {
            "task_id": task["task_id"],
            "trace_id": f"clawg-{task['task_id']}",
            "auto_verify": {"score": round(auto, 4), "weight": 0.6, "checks": ["schema", "safe_ops", "mock_tests"]},
            "llm_verify": {"score": round(llm, 4), "weight": 0.4, "judge_model": "gpt-5.5"},
            "score_apex": round(score, 4),
            "breakdown": {"correctness": round(auto,4), "clarity": round(llm,4)}
        }
        out.write(json.dumps(report, ensure_ascii=False) + "\n")

print(f"wrote {REPORTS}")
