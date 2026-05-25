#!/usr/bin/env bash
set -euo pipefail

# Safe GitHub sync for APEX evolution artifacts.
# This script is intentionally explicit and refuses to run without a target repo.
# Usage:
#   APEX_GITHUB_REPO=owner/private-repo ./apex-github-evolution/scripts/sync_private_repo.sh

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EXPORT_MANIFEST="$ROOT/apex-github-evolution/exports/latest.manifest.json"
REPO="${APEX_GITHUB_REPO:-}"
BRANCH="${APEX_GITHUB_BRANCH:-apex-evolution}"
WORKDIR="${APEX_SYNC_WORKDIR:-/tmp/apex-github-sync}"

if [[ -z "$REPO" ]]; then
  echo "BLOCKED: set APEX_GITHUB_REPO=owner/private-repo first" >&2
  exit 2
fi

if [[ ! -f "$EXPORT_MANIFEST" ]]; then
  echo "BLOCKED: export manifest missing. Run create_safe_export.py first." >&2
  exit 2
fi

SECRET_COUNT="$(python3 - <<PY
import json
print(json.load(open('$EXPORT_MANIFEST')).get('secret_hit_count', 999))
PY
)"
if [[ "$SECRET_COUNT" != "0" ]]; then
  echo "BLOCKED: secret_hit_count=$SECRET_COUNT" >&2
  exit 3
fi

if ! gh auth status >/tmp/apex_gh_auth.log 2>&1; then
  echo "BLOCKED: gh auth status failed" >&2
  cat /tmp/apex_gh_auth.log >&2
  exit 4
fi

TARBALL="$(python3 - <<PY
import json
print(json.load(open('$EXPORT_MANIFEST'))['tarball'])
PY
)"

rm -rf "$WORKDIR"
mkdir -p "$WORKDIR"

gh repo view "$REPO" >/dev/null

git clone "https://github.com/$REPO.git" "$WORKDIR/repo"
cd "$WORKDIR/repo"
if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
  git checkout "$BRANCH"
else
  git checkout -b "$BRANCH"
fi

mkdir -p apex-evolution
rm -rf apex-evolution/*
tar -xzf "$TARBALL" -C apex-evolution
cp "$EXPORT_MANIFEST" apex-evolution/export.manifest.json

git add apex-evolution
if git diff --cached --quiet; then
  echo "No changes to commit."
  exit 0
fi

git commit -m "chore(apex): sync safe evolution artifacts"
git push -u origin "$BRANCH"

echo "SYNC_OK: https://github.com/$REPO/tree/$BRANCH/apex-evolution"
