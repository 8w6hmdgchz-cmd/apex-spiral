#!/usr/bin/env python3
"""开智V6 - 短板扫描 + 主动验证 + 问题生成"""
import json
import sys
from datetime import datetime

LOG_DIR = "/Users/lihongxin/.openclaw/workspace/a2a-resources/state"
FEEDBACK_LOG = f"{LOG_DIR}/feedback_log.jsonl"
SHORTBOARD_LOG = f"{LOG_DIR}/shortboard_scan.jsonl"

def get_feedback_count():
    try:
        with open(FEEDBACK_LOG) as f:
            return len([l for l in f if l.strip()])
    except:
        return 0

def get_last_dg():
    try:
        with open(f"{LOG_DIR}/phi_history.jsonl") as f:
            lines = f.readlines()
            if lines:
                return json.loads(lines[-1]).get('dg', 0)
    except:
        pass
    return None

def main():
    ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    feedback_count = get_feedback_count()
    last_dg = get_last_dg()
    theta_bio = min(0.5 + feedback_count * 0.1, 0.95)
    
    # 短板列表
    shortboards = [
        {"id": "no_real_feedback", "category": "evolution", "score": max(1.0 - feedback_count * 0.2, 0.2), "desc": "无真实反馈"},
        {"id": "h_real_low", "category": "cognition", "score": 0.7, "desc": "h_real需要实测"},
        {"id": "phi_stable", "category": "evolution", "score": 0.6, "desc": "Φ值稳定无进化"},
    ]
    
    top = min(shortboards, key=lambda x: x['score'])
    
    # 生成具体问题（每次不同）
    questions = [
        "开智循环感觉停滞了吗？有什么具体改进建议？",
        "最近的自我进化是否感觉停滞了？有哪些具体的改进建议？",
        "Cron输出有什么问题？需要增加什么信息？",
        "开智的哪些步骤你觉得多余？哪些需要加强？",
        "你希望开智循环更频繁还是更稀疏？",
    ]
    import random
    question = random.choice(questions)
    
    result = {
        "ts": int(datetime.now().timestamp()),
        "xi_self": theta_bio,
        "feedback_count": feedback_count,
        "last_dg": last_dg,
        "top_shortboard": top,
        "all_shortboards": shortboards,
        "question_to_user": question
    }
    
    with open(SHORTBOARD_LOG, 'a') as f:
        f.write(json.dumps(result, ensure_ascii=False) + '\n')
    
    # 输出给用户看
    print("=" * 60)
    print(f"🔍 开智V6 短板扫描 [{ts}]")
    print("=" * 60)
    print(f"📊 Θ_bio={theta_bio:.3f} | 反馈数={feedback_count} | ΔG={last_dg}")
    print(f"🎯 主短板: {top['id']} ({top['desc']})")
    print(f"❓ 问题: {question}")
    print("=" * 60)
    
    return result

if __name__ == "__main__":
    main()
