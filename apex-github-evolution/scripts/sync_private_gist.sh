#!/usr/bin/env bash
set -euo pipefail

# Safe private gist sync. Requires explicit env var APEX_GIST_ID for update,
# or APEX_GIST_CREATE=1 to create a new private gist.

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EXPORT_MANIFEST="$ROOT/apex-github-evolution/exports/latest.manifest.json"
GIST_ID="${APEX_GIST_ID:-}"
CREATE="${APEX_GIST_CREATE:-0}"

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

TARBALL="$(python3 - <<PY
import json
print(json.load(open('$EXPORT_MANIFEST'))['tarball'])
PY
)"
TMP="/tmp/apex-gist-sync"
rm -rf "$TMP"
mkdir -p "$TMP"
tar -xzf "$TARBALL" -C "$TMP"
cp "$EXPORT_MANIFEST" "$TMP/export.manifest.json"

# Gist cannot preserve directories well in every client; publish manifest + README only by default.
README="$TMP/APEX_EVOLUTION_README.md"
cat > "$README" <<'EOF'
# APEX Evolution Safe Export

This private gist contains the export manifest for a local-safe APEX evolution bundle.
Full artifact tarball remains local unless separately approved.
EOF

sync_with_git_ssh() {
  local gist_id="$1"
  local repo_dir="/tmp/apex-gist-repo-$gist_id"
  rm -rf "$repo_dir"
  git clone "git@gist.github.com:$gist_id.git" "$repo_dir"
  cp "$TMP/export.manifest.json" "$repo_dir/export.manifest.json"
  cp "$README" "$repo_dir/APEX_EVOLUTION_README.md"
  (cd "$repo_dir" && git add export.manifest.json APEX_EVOLUTION_README.md)
  if (cd "$repo_dir" && git diff --cached --quiet); then
    echo "GIST_UP_TO_DATE: $gist_id"
    return 0
  fi
  (cd "$repo_dir" && git commit -m "Update APEX safe export manifest")
  (cd "$repo_dir" && git push origin HEAD)
  echo "GIST_UPDATE_OK: $gist_id"
}

if [[ -n "$GIST_ID" ]]; then
  if gh auth status >/tmp/apex_gh_auth.log 2>&1; then
    gh gist edit "$GIST_ID" "$TMP/export.manifest.json" "$README"
    echo "GIST_UPDATE_OK: $GIST_ID"
  else
    echo "WARN: gh auth unavailable; falling back to git+ssh gist sync" >&2
    sync_with_git_ssh "$GIST_ID"
  fi
elif [[ "$CREATE" == "1" ]]; then
  if ! gh auth status >/tmp/apex_gh_auth.log 2>&1; then
    echo "BLOCKED: gh auth status failed; creation requires gh API auth" >&2
    cat /tmp/apex_gh_auth.log >&2
    exit 4
  fi
  gh gist create --private "$TMP/export.manifest.json" "$README" --desc "APEX evolution safe export manifest"
else
  echo "BLOCKED: set APEX_GIST_ID=<id> or APEX_GIST_CREATE=1" >&2
  exit 2
fi
