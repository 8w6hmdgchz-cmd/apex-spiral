#!/usr/bin/env python3
"""APEX Unified Research Engine MVP orchestrator."""
from __future__ import annotations
import json
import subprocess
import sys
import time
import uuid
from pathlib import Path
from typing import Any, Dict

ROOT = Path(__file__).resolve().parents[2]
WORKSPACE = ROOT.parent
sys.path.insert(0, str(ROOT / "py" / "research"))

from era import run_era
from co_scientist import run_co_scientist
from robin import run_robin


def run_cmd(cmd: list[str], cwd: Path) -> Dict[str, Any]:
    started = time.time()
    proc = subprocess.run(cmd, cwd=str(cwd), text=True, capture_output=True)
    return {
        "cmd": cmd,
        "cwd": str(cwd),
        "code": proc.returncode,
        "latency_ms": round((time.time() - started) * 1000, 2),
        "stdout": proc.stdout[-3000:],
        "stderr": proc.stderr[-3000:]
    }


def score_ui_control() -> Dict[str, Any]:
    token_rs = WORKSPACE / "apex_token_rs"
    if not token_rs.exists():
        return {"score": 0.0, "status": "missing", "artifact": None}
    result = run_cmd(["cargo", "test"], token_rs)
    score = 1.0 if result["code"] == 0 else 0.25
    return {"score": score, "status": "ok" if result["code"] == 0 else "failed", "result": result}


def score_training() -> Dict[str, Any]:
    clawg = WORKSPACE / "clawg-mvp"
    script = clawg / "scripts" / "run_iteration.sh"
    if not script.exists():
        return {"score": 0.0, "status": "missing", "artifact": None}
    result = run_cmd([str(script)], clawg)
    summary_path = clawg / "datasets" / "processed" / "summary.json"
    summary = json.loads(summary_path.read_text(encoding="utf-8")) if summary_path.exists() else {}
    score = float(summary.get("score_mean", 0.0)) if result["code"] == 0 else 0.0
    return {"score": score, "status": "ok" if result["code"] == 0 else "failed", "summary": summary, "result": result}


def score_research(question: str, trace_dir: Path) -> Dict[str, Any]:
    research_dir = trace_dir / "research"
    era = run_era(question, research_dir)
    co = run_co_scientist(question, research_dir)
    rb = run_robin(question, research_dir)
    score = round((era["score"] + co["score"] + rb["score"]) / 3, 4)
    return {"score": score, "systems": {"era": era, "co_scientist": co, "robin": rb}}


def run_engine(question: str) -> Dict[str, Any]:
    trace_id = f"apex-engine-{int(time.time() * 1000)}-{uuid.uuid4().hex[:8]}"
    trace_dir = ROOT / "reports" / trace_id
    trace_dir.mkdir(parents=True, exist_ok=True)

    ui = score_ui_control()
    training = score_training()
    research = score_research(question, trace_dir)

    ui_score = ui["score"]
    training_score = training["score"]
    research_score = research["score"]
    engine_score = round((ui_score * 0.25) + (training_score * 0.35) + (research_score * 0.40), 4)

    report = {
        "engine": "APEX Unified Research Engine MVP",
        "trace_id": trace_id,
        "timestamp_ms": int(time.time() * 1000),
        "question": question,
        "scores": {
            "ui_control": ui_score,
            "training": training_score,
            "research": research_score,
            "engine_apex": engine_score
        },
        "modules": {
            "ui_control": ui,
            "training": training,
            "research": research
        },
        "conclusion": "MVP closed loop passed" if engine_score >= 0.75 else "MVP needs repair before expansion",
        "artifacts": [
            str(trace_dir / "engine_report.json"),
            *(s["artifact"] for s in research["systems"].values())
        ]
    }
    (trace_dir / "engine_report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    (ROOT / "reports" / "latest_report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    return report


if __name__ == "__main__":
    q = " ".join(sys.argv[1:]).strip() or "How can APEX improve reproducible local agent research workflows?"
    report = run_engine(q)
    print(json.dumps({
        "trace_id": report["trace_id"],
        "scores": report["scores"],
        "conclusion": report["conclusion"],
        "report": report["artifacts"][0]
    }, ensure_ascii=False, indent=2))
