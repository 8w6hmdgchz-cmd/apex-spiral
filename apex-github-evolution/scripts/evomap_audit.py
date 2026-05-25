#!/usr/bin/env python3
"""APEX EvoMap: local audit manifest for GitHub-safe evolution artifacts."""
from __future__ import annotations
import hashlib
import json
import os
import re
import subprocess
import time
from pathlib import Path
from typing import Dict, List

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "apex-github-evolution" / "reports"
EVOMAP = ROOT / "apex-github-evolution" / "evomap"
OUT.mkdir(parents=True, exist_ok=True)
EVOMAP.mkdir(parents=True, exist_ok=True)

SAFE_DIRS = [
    "apex_token_rs",
    "clawg-mvp",
    "apex-unified-engine",
    "skills/hetu-luoshu",
    "skills/apex-token-optimizer",
]

SECRET_PATTERNS = [
    re.compile(r"fe_oa_[A-Za-z0-9]{16,}"),
    re.compile(r"sk-[A-Za-z0-9_\-]{16,}"),
    re.compile(r"ghp_[A-Za-z0-9]{20,}"),
    re.compile(r"github_pat_[A-Za-z0-9_]{20,}"),
]


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def scan_file(path: Path) -> List[str]:
    try:
        text = path.read_text(encoding="utf-8", errors="ignore")
    except Exception:
        return []
    hits = []
    for pat in SECRET_PATTERNS:
        if pat.search(text):
            hits.append(pat.pattern)
    return hits


def git_status() -> str:
    proc = subprocess.run(["git", "status", "--short"], cwd=str(ROOT), text=True, capture_output=True)
    return proc.stdout


def main() -> None:
    files: List[Dict] = []
    secret_hits: List[Dict] = []
    for rel_dir in SAFE_DIRS:
        base = ROOT / rel_dir
        if not base.exists():
            continue
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            rel = path.relative_to(ROOT).as_posix()
            if any(part in rel for part in ["/target/", "/__pycache__/"]) or rel.endswith(".pyc"):
                continue
            item = {"path": rel, "bytes": path.stat().st_size, "sha256": sha256(path)}
            files.append(item)
            hits = scan_file(path)
            if hits:
                secret_hits.append({"path": rel, "patterns": hits})

    report = {
        "trace_id": f"evomap-{int(time.time()*1000)}",
        "timestamp_ms": int(time.time() * 1000),
        "safe_dirs": SAFE_DIRS,
        "file_count": len(files),
        "files": files,
        "secret_hits": secret_hits,
        "git_status_short": git_status(),
        "external_sync_allowed": False,
        "next_step": "Review secret_hits and git_status. Ask user before commit/push/gist."
    }
    latest = EVOMAP / "latest.json"
    latest.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    (OUT / f"{report['trace_id']}.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps({
        "trace_id": report["trace_id"],
        "file_count": report["file_count"],
        "secret_hit_count": len(secret_hits),
        "latest": str(latest),
        "external_sync_allowed": False
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
