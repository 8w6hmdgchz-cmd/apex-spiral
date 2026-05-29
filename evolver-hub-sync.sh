#!/bin/bash
# EvoMap Hub 心跳 + 资源同步脚本 v2
# 使用Python做HTTP请求(解决macOS curl兼容问题)
# 每15分钟自动从EvoMap Hub获取资源并吸收落地

set -euo pipefail

LOG_DIR="/Users/lihongxin/.openclaw/workspace/a2a-resources"
STATE_DIR="$LOG_DIR/state"
HUB_URL="https://evomap.ai"
SYNC_LOG="$LOG_DIR/hub-sync.log"
mkdir -p "$STATE_DIR"

TIMESTAMP=$(date "+%Y-%m-%d %H:%M GMT+8")
echo "[$TIMESTAMP] === EvoMap Hub同步开始 ===" >> "$SYNC_LOG"

# ============================================================
# Step 1: 发送心跳 (POST /a2a/heartbeat)
# ============================================================
send_heartbeat() {
    local node_id="${EVO_NODE_ID:-}"
    local node_secret="${EVO_NODE_SECRET:-}"
    
    if [ -z "$node_id" ] || [ -z "$node_secret" ]; then
        echo "[$TIMESTAMP] 心跳: node_id或node_secret未设置，跳过" >> "$SYNC_LOG"
        return 0  # 心跳不是必须的，继续其他步骤
    fi
    
    python3 - <<PY
import urllib.request, json, os, sys
from datetime import datetime

node_id = "$node_id"
node_secret = "$node_secret"
hub_url = "$HUB_URL"
log_path = "$SYNC_LOG"

msg_id = f"msg_{int(__import__('time').time())}"
timestamp = datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ')

payload = {
    "protocol": "gep-a2a",
    "protocol_version": "1.0.0",
    "message_type": "heartbeat",
    "message_id": msg_id,
    "sender_id": node_id,
    "timestamp": timestamp,
    "payload": {}
}

try:
    req = urllib.request.Request(
        f"{hub_url}/a2a/heartbeat",
        data=json.dumps(payload).encode(),
        headers={
            'Content-Type': 'application/json',
            'Authorization': f'Bearer {node_secret}'
        },
        method='POST'
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        result = json.loads(resp.read())
        with open(log_path, 'a') as f:
            f.write(f"[{datetime.now().strftime('%Y-%m-%d %H:%M GMT+8')}] 心跳: 成功 status={result.get('status','?')}\n")
except Exception as e:
    with open(log_path, 'a') as f:
        f.write(f"[{datetime.now().strftime('%Y-%m-%d %H:%M GMT+8')}] 心跳: 失败 ({e})\n")
PY
}

# ============================================================
# Step 2: 获取当前短板关键词
# ============================================================
get_shortboard_keywords() {
    local report="$LOG_DIR/latest-report.md"
    local keywords=""
    
    if [ -f "$report" ]; then
        # 提取短板描述转成友好搜索词
        local shortboard=$(grep "识别短板" "$report" 2>/dev/null | sed 's/.*识别短板[：:]*//' | head -1 || echo "")
        local bug=$(grep "bug:" "$report" 2>/dev/null | head -1 | sed 's/.*bug:.*\[//' | sed 's/\].*//' || echo "")
        
        # 将短板描述转为友好搜索词
        keywords=$(python3 -c "
import sys
sb = \"$shortboard\".strip()
bug = \"$bug\".strip()
# 提取关键概念
words = []
if '自我感知' in sb or 'reflection' in sb.lower(): words.append('self reflection')
if '缺陷' in sb or 'defect' in sb.lower(): words.append('bug fix')
if '增长' in sb or 'growth' in sb.lower(): words.append('self improvement')
if '自适应' in sb or 'adaptive' in sb.lower(): words.append('adaptive loop')
if '反馈' in sb or 'feedback' in sb.lower(): words.append('feedback control')
if '进化' in sb or bug == 'B4': words.append('agent evolution')
if not words: words = ['agent evolution', 'self improvement', 'adaptive loop']
print(' '.join(words))
" 2>/dev/null || echo "agent evolution self improvement adaptive loop")
    fi
    
    # 默认关键词
    if [ -z "$keywords" ]; then
        keywords="agent evolution self improvement adaptive loop"
    fi
    
    echo "$keywords"
}

# ============================================================
# Step 3: 从EvoMap semantic-search 获取并吸收资源
# ============================================================
fetch_and_absorb() {
    local keywords="$1"
    local absorbed_dir="$STATE_DIR/absorbed_hub"
    mkdir -p "$absorbed_dir"
    
    python3 - <<PY
import urllib.request, urllib.parse, json, os, sys
from datetime import datetime

keywords = "$keywords"
hub_url = "$HUB_URL"
log_path = "$SYNC_LOG"
absorbed_dir = "$absorbed_dir"

log = lambda msg: open(log_path, 'a').write(f"[{datetime.now().strftime('%Y-%m-%d %H:%M GMT+8')}] {msg}\n")

# Step A: 搜索资源
encoded_q = urllib.parse.quote(keywords)
search_url = f"{hub_url}/a2a/assets/semantic-search?q={encoded_q}&limit=10"

try:
    req = urllib.request.Request(search_url, headers={'User-Agent': 'Mozilla/5.0'})
    with urllib.request.urlopen(req, timeout=15) as resp:
        search_data = json.loads(resp.read())
    
    assets = search_data.get('assets', [])
    log(f"搜索: 找到{len(assets)}个资源 (关键词: {keywords[:60]})")
    
    absorbed_count = 0
    
    for asset in assets:
        conf = asset.get('confidence', 0)
        streak = asset.get('success_streak', 0)
        asset_id = asset.get('asset_id', '')
        trigger = asset.get('trigger_text', 'N/A')[:80]
        
        # 只吸收高置信度(>=0.85)且有连续成功(streak>=1)的资源
        if conf >= 0.85 and streak >= 1 and asset_id:
            # 获取完整内容
            detail_url = f"{hub_url}/a2a/assets/{asset_id}?detailed=true"
            try:
                req2 = urllib.request.Request(detail_url, headers={'User-Agent': 'Mozilla/5.0'})
                with urllib.request.urlopen(req2, timeout=10) as resp2:
                    detail = json.loads(resp2.read())
                
                payload = detail.get('payload', {})
                strategy = payload.get('strategy', [])
                summary = payload.get('summary', trigger)
                
                # 写入本地吸收文件
                safe_name = asset_id.replace(':', '_').replace('-', '_')[:50]
                out_file = f"{absorbed_dir}/{safe_name}.json"
                
                with open(out_file, 'w') as f:
                    json.dump({
                        'absorbed_at': datetime.now().isoformat(),
                        'source': 'evomap_hub',
                        'asset_id': asset_id,
                        'trigger_text': trigger,
                        'confidence': conf,
                        'success_streak': streak,
                        'summary': summary,
                        'strategy': strategy[:5] if isinstance(strategy, list) else [str(strategy)],
                        'gdi_score': asset.get('gdi_score', 0)
                    }, f, indent=2, ensure_ascii=False)
                
                absorbed_count += 1
                log(f"吸收: [{conf}/{streak}] {trigger[:60]}")
                
            except Exception as e:
                log(f"吸收详情失败: {asset_id[:30]} ({e})")
        else:
            log(f"跳过: [{conf}/{streak}] {trigger[:60]}")
    
    log(f"吸收完成: 新增{absorbed_count}个高置信度资源")
    
except Exception as e:
    log(f"搜索失败: {e}")

PY
}

# ============================================================
# 主流程
# ============================================================
main() {
    # 1. 发送心跳(可选，跳过不影响主流程)
    send_heartbeat || true
    
    # 2. 获取当前短板关键词
    KEYWORDS=$(get_shortboard_keywords)
    echo "[$TIMESTAMP] 当前短板关键词: $KEYWORDS" >> "$SYNC_LOG"
    
    # 3. 搜索并吸收资源
    fetch_and_absorb "$KEYWORDS"
    
    echo "[$TIMESTAMP] === EvoMap Hub同步完成 ===" >> "$SYNC_LOG"
}

main
