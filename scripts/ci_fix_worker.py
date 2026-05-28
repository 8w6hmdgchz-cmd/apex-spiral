#!/usr/bin/env python3
"""
CI Fix Worker - 读取 ci_fix_queue.json，处理失败任务
每次只处理一个，失败后保留在队列供下次重试
"""
import json
import subprocess
import time
import sys
from pathlib import Path

REPO_DIR = Path("/Users/lihongxin/.openclaw/workspace")
FIX_QUEUE = REPO_DIR / "memory" / "ci_fix_queue.json"
STATE_FILE = REPO_DIR / "memory" / "ci_fix_state.json"
LOG_FILE = REPO_DIR / "memory" / "ci_fix_log.jsonl"

def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}")

def load_queue():
    if FIX_QUEUE.exists():
        return json.loads(FIX_QUEUE.read_text())
    return []

def save_queue(queue):
    FIX_QUEUE.write_text(json.dumps(queue, indent=2))

def load_state():
    if STATE_FILE.exists():
        return json.loads(STATE_FILE.read_text())
    return {"last_checked_uid": None, "retry_counts": {}}

def git_pull():
    """确保本地是最新"""
    result = subprocess.run(
        ["git", "fetch", "origin", "main"],
        cwd=REPO_DIR, capture_output=True, text=True, timeout=30
    )
    return result.returncode == 0

def git_push(fix_description):
    """Commit 并 push 修复"""
    subprocess.run(["git", "add", "-A"], cwd=REPO_DIR, capture_output=True)
    result = subprocess.run(
        ["git", "commit", "-m", fix_description],
        cwd=REPO_DIR, capture_output=True, text=True
    )
    if result.returncode != 0:
        return False, result.stderr
    
    result = subprocess.run(
        ["git", "push", "origin", "main"],
        cwd=REPO_DIR, capture_output=True, text=True, timeout=30
    )
    return result.returncode == 0, result.stderr

def main():
    queue = load_queue()
    if not queue:
        log("Queue empty, nothing to fix")
        return
    
    item = queue[0]  # 取第一个
    workflow = item['workflow']
    commit = item['commit']
    retry = item.get('retry', 1)
    
    log(f"Processing {workflow} failure (commit {commit}, retry {retry})")
    
    # Git pull 确保最新
    git_pull()
    
    # 检查是否是 Rust CI
    if workflow == 'Rust CI':
        # 读取 CI 日志信息
        body = item.get('body', '')
        annotations = item.get('annotations', 0)
        
        # 本地跑一下 Rust CI，看具体哪里错
        log("  Running cargo clippy locally...")
        result = subprocess.run(
            ["cargo", "clippy", "--workspace", "--", "-D", "warnings"],
            cwd=REPO_DIR, capture_output=True, text=True, timeout=120
        )
        
        if result.returncode == 0:
            log("  Local clippy passes - issue may be in CI environment")
            # 如果本地过但 CI 失败，可能是 GitHub runner 缺少组件
            # 检查 CI workflow 是否显式安装了 toolchain components
            
            ci_yml = REPO_DIR / ".github/workflows/ci.yml"
            content = ci_yml.read_text()
            
            if "rust-toolchain" not in content or "components" not in content:
                log("  Adding explicit rust-toolchain with components...")
                
                # 添加 components 安装 step
                new_step = """      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy"""
                
                content = content.replace(
                    """      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable""",
                    new_step
                )
                ci_yml.write_text(content)
                
                success, err = git_push(f"fix: explicit rustfmt+clippy components for CI (auto-fix)")
                if success:
                    log("  Pushed rust-toolchain fix!")
                    queue.pop(0)
                    save_queue(queue)
                    log("  Removed from queue, waiting for CI to verify...")
                else:
                    log(f"  Push failed: {err}")
            else:
                log("  CI config looks correct - may need manual investigation")
                # 移出队列（不重试）
                queue.pop(0)
                save_queue(queue)
        else:
            # 本地也失败 - 解析错误
            errors = result.stderr
            log(f"  Local clippy failed: {errors[:300]}")
            
            # 简单错误修复
            fix_applied = False
            
            # 1. 格式化问题
            if "format" in errors.lower():
                log("  Applying: cargo fmt...")
                subprocess.run(["cargo", "fmt"], cwd=REPO_DIR, capture_output=True)
                success, err = git_push("style: cargo fmt (auto-fix from CI failure)")
                if success:
                    queue.pop(0)
                    save_queue(queue)
                    log("  Fixed formatting issue, pushed!")
                    fix_applied = True
            
            if not fix_applied:
                # 无法自动修复，保留在队列
                log("  Cannot auto-fix, will retry on next cycle")
                queue[0]['last_retry'] = time.time()
                save_queue(queue)

if __name__ == "__main__":
    main()
