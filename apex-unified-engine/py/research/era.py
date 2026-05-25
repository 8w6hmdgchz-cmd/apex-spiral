#!/usr/bin/env python3
"""ERA MVP: LLM x TreeSearch x CodeSandbox skeleton."""
from __future__ import annotations
import json
from pathlib import Path
from typing import Any, Dict, List


def run_era(research_question: str, out_dir: Path) -> Dict[str, Any]:
    out_dir.mkdir(parents=True, exist_ok=True)
    nodes: List[Dict[str, Any]] = [
        {"node_id": "root", "claim": research_question, "score": 0.72},
        {"node_id": "n1", "claim": "Map the question into variables, datasets, and falsifiable tests", "score": 0.82},
        {"node_id": "n2", "claim": "Prefer mechanisms that can be checked by code or local evidence", "score": 0.79},
        {"node_id": "n3", "claim": "Generate a minimal reproducible experiment before writing conclusions", "score": 0.86}
    ]
    best = max(nodes, key=lambda n: n["score"])
    artifact = out_dir / "era_tree.json"
    artifact.write_text(json.dumps({"question": research_question, "nodes": nodes, "best": best}, ensure_ascii=False, indent=2), encoding="utf-8")
    return {"system": "ERA", "score": best["score"], "best_node": best, "artifact": str(artifact)}
