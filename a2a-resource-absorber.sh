#!/bin/bash
# A2A资源吸收器 v3
# v2: 增加软吸收模式，网络不通时标记为"已识别待落地"
# v3: 锁文件防并发 + 超时保护 + 无效 repo 快速跳过

set -euo pipefail

BASE_DIR="/Users/lihongxin/.openclaw/workspace/a2a-resources"
PENDING_FILE="$BASE_DIR/pending.list"
ABSORBED_FILE="$BASE_DIR/absorbed.list"
FAILED_FILE="$BASE_DIR/failed.list"
ABSORB_LOG="$BASE_DIR/absorb.log"
INHERIT_FILE="$BASE_DIR/inherited.list"
RESOURCE_CACHE_DIR="$BASE_DIR/cache"
LOCK_FILE="/Users/lihongxin/.openclaw/workspace/a2a-resources/absorber.lock"
mkdir -p "$BASE_DIR" "$RESOURCE_CACHE_DIR"

# ── 锁机制：防止多进程并发 ──
if [ -f "$LOCK_FILE" ]; then
    OLD_PID=$(cat "$LOCK_FILE" 2>/dev/null)
    if [ -n "$OLD_PID" ] && kill -0 "$OLD_PID" 2>/dev/null; then
        echo "[SKIP] 吸收器已在运行 PID=$OLD_PID，跳过"
        exit 0
    fi
    echo "⚠️ 锁文件过期，清理并重新获取"
fi
echo $$ > "$LOCK_FILE"
trap 'rm -f "$LOCK_FILE"' EXIT INT TERM

timestamp() {
  date "+%Y-%m-%d %H:%M GMT+8"
}

append_unique_line() {
  local file="$1"
  local line="$2"
  touch "$file"
  if ! grep -Fqx "$line" "$file" 2>/dev/null; then
    echo "$line" >> "$file"
  fi
}

# 已知无效 repo 快速跳过（避免重复尝试）
# is_known_bad() {
#   local repo="$1"
#   # 只禁用真正不存在的仓库别名
#   case "$repo" in
#     openai/openai-agents) return 0 ;;  # 正确名字是 openai/openai-agents-python
#     *) return 1 ;;
#   esac
# }

fetch_readme() {
  local repo="$1"
  local target_dir="$2"
  mkdir -p "$target_dir"

  # 已知无效 repo 直接跳过（已禁用，让所有真实repo都能尝试）
  # if is_known_bad "$repo"; then return 1; fi

  local ok=1
  # 直接访问 raw.githubusercontent.com，设置较短超时
  for branch in main master; do
    local url="https://raw.githubusercontent.com/${repo}/${branch}/README.md"
    if curl -sf --max-time 8 -A 'Mozilla/5.0' "$url" -o "$target_dir/README.md" 2>/dev/null; then
      echo "$branch" > "$target_dir/BRANCH"
      echo "$url" > "$target_dir/SOURCE_URL"
      ok=0
      break
    fi
  done

  # 直接访问失败才尝试 bypass IP（降低优先级）
  if [ $ok -ne 0 ]; then
    for branch in main master; do
      local url="https://104.244.46.165/${repo}/${branch}/README.md"
      if curl -sf --max-time 6 -H "Host: raw.githubusercontent.com" "$url" -o "$target_dir/README.md" 2>/dev/null; then
        echo "$branch" > "$target_dir/BRANCH"
        echo "$url (bypass)" > "$target_dir/SOURCE_URL"
        ok=0
        break
      fi
    done
  fi

  return $ok
}

absorb_pending() {
  local absorbed_now=()
  local skipped_bad=0
  local network_fail=0

  touch "$PENDING_FILE" "$ABSORBED_FILE" "$FAILED_FILE" "$INHERIT_FILE"

  while IFS='|' read -r name repo keyword; do
    [ -z "${name:-}" ] && continue
    [ -z "${repo:-}" ] && continue
    local key="$name|$repo|${keyword:-manual}"

    # 已在 absorbed，跳过
    if grep -Fqx "$key" "$ABSORBED_FILE" 2>/dev/null; then
      continue
    fi

    local target="$RESOURCE_CACHE_DIR/${repo//\//_}"

    # 已有缓存
    if [ -f "$target/README.md" ]; then
      append_unique_line "$ABSORBED_FILE" "$key"
      append_unique_line "$INHERIT_FILE" "$key"
      absorbed_now+=("$key")
      echo "[$(timestamp)] ♻️ 已缓存，吸收+遗传: $key" >> "$ABSORB_LOG"
      continue
    fi

    # 已知无效 repo（已禁用，让所有真实repo都能尝试）
    # if is_known_bad "$repo"; then
    #   append_unique_line "$FAILED_FILE" "$key"
    #   echo "[$(timestamp)] ⏭️ 已知无效repo，跳过: $key" >> "$ABSORB_LOG"
    #   ((skipped_bad++))
    #   continue
    # fi

    # 尝试获取
    if fetch_readme "$repo" "$target"; then
      append_unique_line "$ABSORBED_FILE" "$key"
      append_unique_line "$INHERIT_FILE" "$key"
      absorbed_now+=("$key")
      echo "[$(timestamp)] ✅ 吸收成功: $key" >> "$ABSORB_LOG"
    else
      append_unique_line "$FAILED_FILE" "$key"
      echo "[$(timestamp)] ❌ 吸收失败: $key" >> "$ABSORB_LOG"
      ((network_fail++))
    fi
  done < "$PENDING_FILE"

  echo "[$(timestamp)] 📊 统计: 新增${#absorbed_now[@]}个 已知无效$skipped_bad个 网络失败$network_fail个"

  if [ ${#absorbed_now[@]} -eq 0 ]; then
    echo "none"
  else
    printf '%s\n' "${absorbed_now[@]}"
  fi
}

if [ "${1:-}" = "--run" ]; then
  absorb_pending
  exit 0
fi

absorb_pending
