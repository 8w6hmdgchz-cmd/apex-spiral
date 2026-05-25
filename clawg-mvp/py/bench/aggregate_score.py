#!/usr/bin/env python3
"""ClawG MVP: 聚合Score_APEX"""
import json, pathlib, statistics

ROOT = pathlib.Path(__file__).resolve().parents[2]
REPORTS = ROOT / "datasets" / "processed" / "reports.jsonl"
SUMMARY = ROOT / "datasets" / "processed" / "summary.json"

scores = []
with REPORTS.open(encoding="utf-8") as f:
    for line in f:
        scores.append(json.loads(line)["score_apex"])

summary = {
    "count": len(scores),
    "score_mean": round(statistics.mean(scores), 4) if scores else 0,
    "score_min": min(scores) if scores else 0,
    "score_max": max(scores) if scores else 0,
    "pass_rate_0_75": round(sum(s >= 0.75 for s in scores) / len(scores), 4) if scores else 0
}
SUMMARY.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps(summary, ensure_ascii=False, indent=2))
