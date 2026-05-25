#!/usr/bin/env bash
# Gist SSH backup for 璇玑/APEX.
#
# Gap covered: GitHub REST/HTTPS API is blocked, so this script never calls
# api.github.com/gh gist create. It treats an existing Gist as a normal git
# remote over SSH and pushes a compact, safe backup bundle to it.
#
# Optimal plan under HTTPS block:
#   1) use git@gist.github.com:<gist-id>.git over SSH (-4 to avoid IPv6 stalls)
#   2) stage only curated non-secret evolution artifacts
#   3) commit/push idempotently; no-op when nothing changed
#
# Acceptance criteria:
#   - `bash -n scripts/gist_backup.sh` passes
#   - with SSH access, `scripts/gist_backup.sh --dry-run` builds a backup tree
#   - normal run pushes to $GIST_REPO without using HTTPS/API
#   - exits 0 on no changes, exits non-zero on SSH/push failure

set -Eeuo pipefail
IFS=$'\n\t'

ROOT="${ROOT:-/Users/lihongxin/.openclaw/workspace}"
GIST_REPO="${GIST_REPO:-git@gist.github.com:57fa0d7fc0247f91f9bb744c253c13ff.git}"
GIST_DIR="${GIST_DIR:-/tmp/xuanji_gist_backup}"
BRANCH="${GIST_BRANCH:-main}"
LOCK_FILE="${LOCK_FILE:-/tmp/xuanji_gist_backup.lock}"
DRY_RUN=0

if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=1
fi

log() { printf '[%s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$*"; }
fatal() { log "ERROR: $*" >&2; exit 1; }

require() {
  command -v "$1" >/dev/null 2>&1 || fatal "missing command: $1"
}

with_lock() {
  require mkdir
  if ! mkdir "$LOCK_FILE" 2>/dev/null; then
    log "another gist backup is running; skip"
    exit 0
  fi
  trap 'rm -rf "$LOCK_FILE"' EXIT
}

copy_if_exists() {
  local src="$1" dst="$2"
  if [[ -e "$src" ]]; then
    mkdir -p "$(dirname "$dst")"
    cp -R "$src" "$dst"
  fi
}

prepare_repo() {
  require git
  require rsync
  mkdir -p "$(dirname "$GIST_DIR")"

  if [[ ! -d "$GIST_DIR/.git" ]]; then
    rm -rf "$GIST_DIR"
    log "cloning gist over SSH: $GIST_REPO"
    GIT_SSH_COMMAND="ssh -4 -o ConnectTimeout=20" git clone "$GIST_REPO" "$GIST_DIR"
  else
    log "updating local gist checkout"
    git -C "$GIST_DIR" remote set-url origin "$GIST_REPO"
    GIT_SSH_COMMAND="ssh -4 -o ConnectTimeout=20" git -C "$GIST_DIR" fetch origin "$BRANCH" --prune || true
    git -C "$GIST_DIR" checkout "$BRANCH" 2>/dev/null || git -C "$GIST_DIR" checkout -B "$BRANCH"
    GIT_SSH_COMMAND="ssh -4 -o ConnectTimeout=20" git -C "$GIST_DIR" pull --ff-only origin "$BRANCH" || true
  fi
}

build_backup_tree() {
  local stage="$GIST_DIR/.stage"
  rm -rf "$stage"
  mkdir -p "$stage"

  cat > "$stage/README.md" <<EOF_INNER
# Xuanji APEX SSH Gist Backup

Generated: $(date -Iseconds)
Host root: $ROOT
Transport: SSH git remote (no GitHub HTTPS/API)

Contents are curated evolution state only; secrets and bulky caches are excluded.
EOF_INNER

  copy_if_exists "$ROOT/EVOLUTION_MAP.md" "$stage/EVOLUTION_MAP.md"
  copy_if_exists "$ROOT/EVOLUTION_GENES_V2.md" "$stage/EVOLUTION_GENES_V2.md"
  copy_if_exists "$ROOT/SOUL.md" "$stage/SOUL.md"
  copy_if_exists "$ROOT/IDENTITY.md" "$stage/IDENTITY.md"
  copy_if_exists "$ROOT/score-state.env" "$stage/score-state.env"
  copy_if_exists "$ROOT/state/phi_tracker_latest.json" "$stage/state__phi_tracker_latest.json"
  copy_if_exists "$ROOT/state/phi_history.jsonl" "$stage/state__phi_history.jsonl"
  copy_if_exists "$ROOT/apex-github-evolution/evomap/latest.json" "$stage/evomap__latest.json"
  copy_if_exists "$ROOT/apex-github-evolution/exports/latest.manifest.json" "$stage/exports__latest.manifest.json"
  copy_if_exists "$ROOT/scripts/gist_backup.sh" "$stage/scripts__gist_backup.sh"
  copy_if_exists "$ROOT/scripts/auto_reflux.sh" "$stage/scripts__auto_reflux.sh"
  copy_if_exists "$ROOT/scripts/phi_tracker.sh" "$stage/scripts__phi_tracker.sh"
  copy_if_exists "$ROOT/scripts/crontab_config" "$stage/scripts__crontab_config"

  # Copy recent daily memory names only if explicitly enabled; default avoids
  # leaking private journal content to Gist.
  if [[ "${INCLUDE_MEMORY:-0}" == "1" && -d "$ROOT/memory" ]]; then
    find "$ROOT/memory" -maxdepth 1 -type f -name '20*.md' -mtime -7 -print0 \
      | while IFS= read -r -d '' f; do copy_if_exists "$f" "$stage/memory__$(basename "$f")"; done
  fi

  (cd "$stage" && find . -type f -print | sort | xargs shasum -a 256 > MANIFEST.sha256)

  # Replace tracked backup payload, keeping .git intact.
  find "$GIST_DIR" -mindepth 1 -maxdepth 1 ! -name .git ! -name .stage -exec rm -rf {} +
  rsync -a "$stage/" "$GIST_DIR/"
  rm -rf "$stage"
}

commit_and_push() {
  git -C "$GIST_DIR" add -A
  if git -C "$GIST_DIR" diff --cached --quiet; then
    log "gist backup unchanged"
    return 0
  fi

  local msg="xuanji ssh gist backup $(date '+%Y-%m-%d %H:%M:%S')"
  if [[ "$DRY_RUN" == "1" ]]; then
    log "dry-run: staged changes ready in $GIST_DIR"
    git -C "$GIST_DIR" status --short
    return 0
  fi

  git -C "$GIST_DIR" commit -m "$msg"
  log "pushing gist over SSH"
  GIT_SSH_COMMAND="ssh -4 -o ConnectTimeout=20 -o ServerAliveInterval=10 -o ServerAliveCountMax=2" timeout 60 git -C "$GIST_DIR" push origin "$BRANCH"
}

main() {
  [[ -d "$ROOT" ]] || fatal "ROOT not found: $ROOT"
  with_lock
  prepare_repo
  build_backup_tree
  commit_and_push
  log "gist backup complete"
}

main "$@"
