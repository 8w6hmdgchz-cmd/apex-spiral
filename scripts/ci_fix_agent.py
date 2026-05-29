#!/usr/bin/env python3
"""
CI 自动修复 Agent
当检测到 GitHub CI 失败邮件时，自动分析日志并尝试修复
"""
import os
import imaplib
import email
from email.header import decode_header
import json
import time
import sys
import re
from pathlib import Path

# ============ 配置 =============
EMAIL = os.environ.get("CI_FIX_EMAIL", "284287005@qq.com")
PASSWORD = os.environ.get("CI_FIX_EMAIL_PASSWORD", "")  # QQ邮箱授权码：必须通过环境变量提供，禁止写入仓库
IMAP_SERVER = os.environ.get("CI_FIX_IMAP_SERVER", "imap.qq.com")
IMAP_PORT = int(os.environ.get("CI_FIX_IMAP_PORT", "993"))

REPO_DIR = Path("/Users/lihongxin/.openclaw/workspace")
STATE_FILE = REPO_DIR / "memory" / "ci_fix_state.json"

MAX_RETRIES = 3  # 最多自动修复尝试次数
# =================================

def load_state():
    if STATE_FILE.exists():
        return json.loads(STATE_FILE.read_text())
    return {"last_checked_uid": None, "retry_counts": {}}

def save_state(state):
    STATE_FILE.parent.mkdir(parents=True, exist_ok=True)
    STATE_FILE.write_text(json.dumps(state, indent=2))

def connect_mail():
    if not PASSWORD:
        raise RuntimeError("CI_FIX_EMAIL_PASSWORD is not set")
    mail = imaplib.IMAP4_SSL(IMAP_SERVER, IMAP_PORT)
    mail.login(EMAIL, PASSWORD)
    mail.select('INBOX')
    return mail

def parse_failure_email(msg):
    """从邮件中提取 CI 失败的关键信息"""
    subject = msg['Subject']
    date = msg['Date']
    
    # 提取 commit hash
    commit = None
    for part in msg.walk():
        if part.get_content_type() == 'text/plain':
            payload = part.get_payload(decode=True)
            if payload:
                body = payload.decode('utf-8', errors='replace')
                # 找 commit hash
                m = re.search(r'\(([0-9a-f]{7})\)', subject)
                if m:
                    commit = m.group(1)
                
                # 找 workflow 名称（case-insensitive，防止 "Gist State Sync" vs "gist-sync" 差异）
                workflow = None
                subject_lower = subject.lower()
                if 'gist state sync' in subject_lower or 'gist-sync' in subject_lower:
                    workflow = 'gist-sync'
                elif 'rust ci' in subject_lower:
                    workflow = 'Rust CI'
                
                # 找 duration 和 annotation 数
                duration = None
                annotations = None
                for line in body.split('\n'):
                    if 'Duration:' in line:
                        m = re.search(r'Duration:\s*([\d.]+)', line)
                        if m:
                            duration = float(m.group(1))
                    if 'annotation' in line.lower():
                        m = re.search(r'(\d+)\s*annotation', line)
                        if m:
                            annotations = int(m.group(1))
                
                return {
                    'subject': subject,
                    'date': date,
                    'commit': commit,
                    'workflow': workflow,
                    'duration': duration,
                    'annotations': annotations,
                    'body': body[:2000]  # 限制长度
                }
    return None

def get_failed_workflows(mail, last_uid):
    """获取所有未处理的失败邮件"""
    status, messages = mail.search(None, 'ALL')
    mail_ids = messages[0].split()
    
    failures = []
    seen = set()
    
    for mid in mail_ids:
        uid = int(mid)
        if last_uid and uid <= last_uid:
            break
        
        try:
            status, msg_data = mail.fetch(mid, '(RFC822)')
            msg = email.message_from_bytes(msg_data[0][1])
            subject = msg['Subject']
            
            # 只处理 CI 失败通知
            if '[8w6hmdgchz-cmd/apex-spiral]' in subject and 'Run failed' in subject:
                # 提取 commit
                m = re.search(r'\(([0-9a-f]{7})\)', subject)
                commit = m.group(1) if m else None
                
                if commit and commit not in seen:
                    info = parse_failure_email(msg)
                    if info:
                        failures.append({'uid': uid, 'commit': commit, 'info': info})
                        seen.add(commit)
        except Exception as e:
            print(f"Error fetching mail {mid}: {e}", file=sys.stderr)
            continue
    
    return failures

def main():
    print(f"[{time.strftime('%H:%M:%S')}] CI fix agent checking...")
    
    state = load_state()
    last_uid = state.get('last_checked_uid')
    retry_counts = state.get('retry_counts', {})
    
    try:
        mail = connect_mail()
        failures = get_failed_workflows(mail, last_uid or 0)
        mail.logout()
    except Exception as e:
        print(f"Email check failed: {e}", file=sys.stderr)
        return
    
    if not failures:
        print(f"  No new CI failures")
        return
    
    print(f"  Found {len(failures)} new CI failure(s)")
    
    for f in failures:
        commit = f['commit']
        uid = f['uid']
        info = f['info']
        
        print(f"  Processing {info['workflow']} failure at {commit}")
        print(f"    Duration: {info['duration']}s, Annotations: {info['annotations']}")
        
        # 更新 last_uid
        if not last_uid or uid > last_uid:
            last_uid = uid
            state['last_checked_uid'] = last_uid
        
        # gist-sync 的 0.0s 失败无法自动修复（runner问题），跳过
        if info['workflow'] == 'gist-sync' and info['duration'] == 0.0:
            print(f"    Skipping gist-sync 0.0s failure (runner startup issue)")
            continue

        # gist-sync 非零时长失败——通常是 GIST_ID 未配置或 HTTP 错误，无需自动修复
        if info['workflow'] == 'gist-sync':
            print(f"    Skipping gist-sync failure (GIST_ID/config issue, not auto-fixable)")
            continue

        # Rust CI 失败，触发修复
        if info['workflow'] == 'Rust CI':
            retry_key = f"rust_ci_{commit}"
            retries = retry_counts.get(retry_key, 0)
            
            if retries >= MAX_RETRIES:
                print(f"    Max retries ({MAX_RETRIES}) reached, skipping")
                continue
            
            print(f"    Triggering fix (retry {retries + 1}/{MAX_RETRIES})")
            retry_counts[retry_key] = retries + 1
            state['retry_counts'] = retry_counts
            
            # 写入待处理队列，供 fix-agent 读取
            fix_queue = REPO_DIR / "memory" / "ci_fix_queue.json"
            queue = []
            if fix_queue.exists():
                queue = json.loads(fix_queue.read_text())
            
            queue.append({
                'workflow': info['workflow'],
                'commit': commit,
                'duration': info['duration'],
                'annotations': info['annotations'],
                'body': info['body'][:1500],
                'timestamp': time.time(),
                'retry': retries + 1
            })
            
            fix_queue.write_text(json.dumps(queue, indent=2))
            print(f"    Added to fix queue: {fix_queue}")

    # 保存处理进度，避免重复处理同一封邮件
    save_state(state)

if __name__ == "__main__":
    main()
