#!/usr/bin/env python3
"""开智V6 - 主动修复 + 真实验证"""
import json
import os
import subprocess
from datetime import datetime

LOG_DIR = "/Users/lihongxin/.openclaw/workspace/apex-enlightenment/state"
FIX_LOG = f"{LOG_DIR}/fix_log.jsonl"

def run_real_test():
    """真实运行测试验证"""
    tests = [
        ("GeneNexus", "cd ~/Desktop/开智/GeneNexus-main && python3 -m pytest tests/ -q"),
        ("apex_self_check", "cd ~/Desktop/开智 && python3 -c 'import apex_self_check_v6'"),
    ]
    results = []
    for name, cmd in tests:
        try:
            r = subprocess.run(cmd, shell=True, capture_output=True, timeout=30)
            results.append({"test": name, "ok": r.returncode == 0, "output": r.stdout.decode()[:200] if r.stdout else ""})
        except Exception as e:
            results.append({"test": name, "ok": False, "error": str(e)})
    return results

def main():
    ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    
    # 读取短板
    try:
        with open(f"{LOG_DIR}/shortboard_scan.jsonl") as f:
            lines = f.readlines()
            if lines:
                scan = json.loads(lines[-1])
                top = scan.get('top_shortboard', {})
                question = scan.get('question_to_user', '')
    except:
        top = {"id": "unknown", "score": 1.0}
        question = "有什么改进建议？"
    
    # 真实验证
    test_results = run_real_test()
    all_passed = all(t['ok'] for t in test_results)
    
    # 根据短板生成修复
    fix_actions = {
        "no_real_feedback": {
            "action": "需要用户反馈",
            "msg": f"❓ {question}",
            "delta_theta": 0.15
        },
        "h_real_low": {
            "action": "运行真实测试",
            "msg": f"✅ 测试结果: {'通过' if all_passed else '失败'}",
            "delta_h": 0.2
        },
        "phi_stable": {
            "action": "触发变异",
            "msg": "🔄 Φ值稳定，注入随机扰动",
            "delta_phi": 0.1
        }
    }
    
    fix = fix_actions.get(top['id'], fix_actions["no_real_feedback"])
    fix['test_results'] = test_results
    fix['ts'] = int(datetime.now().timestamp())
    fix['top_shortboard'] = top
    
    with open(FIX_LOG, 'a') as f:
        f.write(json.dumps(fix, ensure_ascii=False) + '\n')
    
    # 输出
    print("=" * 60)
    print(f"🔧 开智V6 自我修复 [{ts}]")
    print("=" * 60)
    print(f"🎯 修复目标: {top.get('id', 'unknown')}")
    print(f"📋 动作: {fix['action']}")
    print(f"💬 {fix['msg']}")
    if test_results:
        print("📊 测试结果:")
        for t in test_results:
            status = "✅" if t['ok'] else "❌"
            print(f"   {status} {t['test']}")
    print("=" * 60)
    
    return fix

if __name__ == "__main__":
    main()
