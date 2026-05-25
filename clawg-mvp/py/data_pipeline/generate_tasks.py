#!/usr/bin/env python3
"""ClawG MVP: 生成模拟任务 Task_APEX"""
import json, uuid, pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "datasets" / "raw" / "tasks.jsonl"
OUT.parent.mkdir(parents=True, exist_ok=True)

def make_task(i):
    return {
        "task_id": f"task_{i:04d}_{uuid.uuid4().hex[:6]}",
        "persona_intent": {
            "persona_id": "local_file_operator",
            "goals": ["edit local file", "run verification", "produce concise report"],
            "style": {"lang": "zh", "tone": "concise", "risk": "low"},
            "preferences": {"stack": ["rust", "go", "python-glue"]}
        },
        "skill_grounding": {
            "skill_id": "file_patch_verify",
            "tools": ["read", "write", "edit", "exec"],
            "knowledge_refs": ["kb://openclaw/file-ops"],
            "action_templates": ["inspect", "patch", "run tests", "report"],
            "success_criteria": ["file updated", "tests pass", "no destructive command"]
        },
        "mock_workspace": {
            "workspace_id": f"mock_ws_{i:04d}",
            "base_image": "local-darwin",
            "visible_paths": ["/workspace/mock"],
            "limits": {"cpu": 2, "mem_mb": 1024, "timeout_sec": 60, "net": "deny"},
            "tests_entry": "python3 -m py_compile solution.py"
        }
    }

with OUT.open("w", encoding="utf-8") as f:
    for i in range(1, 11):
        f.write(json.dumps(make_task(i), ensure_ascii=False) + "\n")

print(f"generated {OUT}")
