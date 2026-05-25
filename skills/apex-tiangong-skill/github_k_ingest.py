#!/usr/bin/env python3
"""Safe GitHub K-ingestion for APEX TianGong.

Collects public repository metadata using lightweight git commands and raw README
fetches. It does not copy third-party source into APEX skills. It records HEAD,
README excerpts, and a clean-room abstraction plan for later reimplementation.
"""
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import time
import uuid
from pathlib import Path
from typing import Any, Dict, List

ROOT = Path(__file__).resolve().parents[2]
SKILL_DIR = ROOT / "skills" / "apex-tiangong-skill"
SEEDS = SKILL_DIR / "github_k_seeds.json"
STATE_DIR = ROOT / "state" / "tiangong" / "github_k"
CACHE_DIR = ROOT / ".openclaw" / "tiangong-github-cache"
STATE_DIR.mkdir(parents=True, exist_ok=True)
CACHE_DIR.mkdir(parents=True, exist_ok=True)

IGNORE_DIRS = {".git", "node_modules", "target", ".venv", "venv", "dist", "build", "__pycache__"}
README_NAMES = ["README.md", "README.rst", "README.txt", "readme.md"]


def run(cmd: List[str], cwd: Path | None = None, timeout: int = 60) -> Dict[str, Any]:
    started = time.time()
    try:
        p = subprocess.run(cmd, cwd=str(cwd) if cwd else None, text=True, capture_output=True, timeout=timeout)
        return {
            "cmd": cmd,
            "code": p.returncode,
            "latency_ms": round((time.time() - started) * 1000, 2),
            "stdout": p.stdout[-4000:],
            "stderr": p.stderr[-4000:],
        }
    except subprocess.TimeoutExpired as exc:
        return {"cmd": cmd, "code": 124, "latency_ms": round((time.time() - started) * 1000, 2), "stdout": "", "stderr": "timeout"}


def safe_name(name: str) -> str:
    return "".join(c.lower() if c.isalnum() else "-" for c in name).strip("-")


def read_excerpt(path: Path, max_chars: int = 2500) -> str:
    if not path.exists():
        return ""
    text = path.read_text(encoding="utf-8", errors="ignore")
    return text[:max_chars]


def top_level_tree(repo_dir: Path) -> List[str]:
    items = []
    for p in sorted(repo_dir.iterdir(), key=lambda x: x.name.lower()):
        if p.name in IGNORE_DIRS:
            continue
        suffix = "/" if p.is_dir() else ""
        items.append(p.name + suffix)
    return items[:120]


def parse_owner_repo(url: str) -> tuple[str, str]:
    cleaned = url.removesuffix(".git")
    parts = cleaned.rstrip("/").split("/")
    return parts[-2], parts[-1]


def fetch_url(url: str, timeout: int = 25) -> Dict[str, Any]:
    return run(["curl", "-L", "--max-time", str(timeout), "-sS", url], timeout=timeout + 5)


def parse_owner_repo(url: str) -> tuple[str, str]:
    cleaned = url.removesuffix(".git")
    parts = cleaned.rstrip("/").split("/")
    return parts[-2], parts[-1]


def clone_or_update(seed: Dict[str, Any]) -> Dict[str, Any]:
    # Minimal robust mode: use only git ls-remote. This avoids copying source and
    # survives slow GitHub raw/clone paths.
    url = seed["url"]
    owner, repo = parse_owner_repo(url)
    head_result = run(["git", "ls-remote", url, "HEAD"], timeout=40)
    if head_result["code"] != 0 or not head_result["stdout"].strip():
        return {"seed": seed, "status": "failed", "action": "ls-remote", "logs": {"ls_remote": head_result}}
    head = head_result["stdout"].split()[0]
    return {
        "seed": seed,
        "status": "ok",
        "action": "ls_remote_only",
        "head": head,
        "branch": "unknown",
        "repo_slug": f"{owner}/{repo}",
        "readme_file": None,
        "readme_excerpt": "README fetch intentionally deferred; this pass records stable remote identity and clean-room plan only.",
        "logs": {"ls_remote": head_result},
    }

def abstraction_plan(item: Dict[str, Any]) -> Dict[str, Any]:
    seed = item["seed"]
    pillar = seed["pillar"]
    base = {
        "pillar": pillar,
        "repo": seed["name"],
        "license_caution": "Do not copy source directly into APEX; derive interfaces and reimplement clean-room after license review.",
        "clean_room_steps": [
            "Identify externally visible behavior and architecture patterns from docs/tests.",
            "Write APEX-native interface spec.",
            "Implement minimal Rust/Go/C core plus Python glue if needed.",
            "Add local tests and negative controls.",
            "Run TianGong orchestrator before promotion."
        ],
    }
    recommendations = {
        "openhands": ["sandbox adapter", "terminal/file operation contract", "browser/action audit trail"],
        "evolver": ["agent loop state machine", "task memory ledger", "repair/evaluation cycle"],
        "autoresearch": ["multi-agent role router", "evidence ledger", "rank-reflect-distill loop"],
        "superpowers": ["SOP gate model", "TDD checklist", "requirement-to-implementation trace"],
    }
    base["candidate_genes"] = recommendations.get(pillar, ["capability abstraction"])
    return base


def main() -> None:
    seeds = json.loads(SEEDS.read_text(encoding="utf-8"))["repos"]
    trace_id = f"github-k-{int(time.time()*1000)}-{uuid.uuid4().hex[:8]}"
    repos = [clone_or_update(seed) for seed in seeds]
    plans = [abstraction_plan(r) for r in repos if r.get("status") == "ok"]
    ok_count = sum(1 for r in repos if r.get("status") == "ok")
    report = {
        "engine": "APEX TianGong GitHub K Ingest",
        "trace_id": trace_id,
        "timestamp_ms": int(time.time() * 1000),
        "source_count": len(repos),
        "ok_count": ok_count,
        "score_k_ingest": round(ok_count / len(repos), 4) if repos else 0,
        "policy": {
            "external_sync_allowed": False,
            "source_copy_allowed": False,
            "mode": "metadata/readme/top-level structure only; clean-room reimplementation required",
        },
        "repos": repos,
        "abstraction_plans": plans,
    }
    out = STATE_DIR / f"{trace_id}.json"
    latest = STATE_DIR / "latest.json"
    out.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    latest.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps({
        "trace_id": trace_id,
        "source_count": len(repos),
        "ok_count": ok_count,
        "score_k_ingest": report["score_k_ingest"],
        "report": str(latest),
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
