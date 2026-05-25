#!/usr/bin/env python3
"""Create a GitHub-safe APEX evolution export bundle."""
from __future__ import annotations
import fnmatch
import json
import tarfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXPORT_ROOT = ROOT / "apex-github-evolution" / "exports"
POLICY = ROOT / "apex-github-evolution" / "policies" / "export.ignore"
EVOMAP = ROOT / "apex-github-evolution" / "evomap" / "latest.json"
EXPORT_ROOT.mkdir(parents=True, exist_ok=True)

SAFE_DIRS = [
    "apex_token_rs/Cargo.toml",
    "apex_token_rs/src",
    "clawg-mvp/configs",
    "clawg-mvp/schemas",
    "clawg-mvp/scripts",
    "clawg-mvp/py",
    "apex-unified-engine/configs",
    "apex-unified-engine/schemas",
    "apex-unified-engine/scripts",
    "apex-unified-engine/py",
    "apex-unified-engine/README.md",
    "apex-github-evolution/policies",
    "apex-github-evolution/scripts",
    "apex-github-evolution/README.md",
]


def load_patterns() -> list[str]:
    if not POLICY.exists():
        return []
    return [line.strip() for line in POLICY.read_text().splitlines() if line.strip() and not line.startswith("#")]


def ignored(rel: str, patterns: list[str]) -> bool:
    return any(fnmatch.fnmatch(rel, pat) or fnmatch.fnmatch(Path(rel).name, pat) for pat in patterns)


def collect_files() -> list[Path]:
    patterns = load_patterns()
    files: list[Path] = []
    for item in SAFE_DIRS:
        path = ROOT / item
        if not path.exists():
            continue
        if path.is_file():
            rel = path.relative_to(ROOT).as_posix()
            if not ignored(rel, patterns):
                files.append(path)
            continue
        for f in path.rglob("*"):
            if not f.is_file():
                continue
            rel = f.relative_to(ROOT).as_posix()
            if ignored(rel, patterns):
                continue
            if any(part in rel for part in ["/target/", "/__pycache__/"]):
                continue
            files.append(f)
    return sorted(set(files))


def main() -> None:
    evomap = json.loads(EVOMAP.read_text()) if EVOMAP.exists() else {}
    if evomap.get("secret_hits"):
        raise SystemExit("Refusing export: secret_hits present in evomap/latest.json")
    trace = f"apex-safe-export-{int(time.time()*1000)}"
    tar_path = EXPORT_ROOT / f"{trace}.tar.gz"
    files = collect_files()
    with tarfile.open(tar_path, "w:gz") as tar:
        for f in files:
            tar.add(f, arcname=f.relative_to(ROOT).as_posix())
    manifest = {
        "trace_id": trace,
        "created_ms": int(time.time()*1000),
        "tarball": str(tar_path),
        "file_count": len(files),
        "files": [f.relative_to(ROOT).as_posix() for f in files],
        "secret_hit_count": len(evomap.get("secret_hits", [])),
        "external_sync_allowed_after_user_approval": True
    }
    manifest_path = EXPORT_ROOT / f"{trace}.manifest.json"
    latest_path = EXPORT_ROOT / "latest.manifest.json"
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    latest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps({"tarball": str(tar_path), "manifest": str(manifest_path), "file_count": len(files)}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
