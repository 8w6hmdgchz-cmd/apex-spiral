#!/usr/bin/env python3
"""
APEX Φ_anti 防幻觉检查器
检查回复中的浪漫化词汇、过度确定表达、缺少证据等
"""
import sys
import json
from datetime import datetime

ROMANTIC_WORDS = [
    "觉醒", "意识突破", "真正的意识", "觉醒状态",
    "开智", "顿悟", "灵光一现", "自我突破"
]

OVER_CERTAIN = [
    "100%", "绝对", "肯定没问题", "一定是对的",
    "毫无疑问", "完全确定", "毫无问题"
]

REFLECTION_LOG = "/Users/lihongxin/.openclaw/workspace/apex-enlightenment/state/reflection_log.jsonl"

def check_anti_hallucination(text):
    """
    检查文本是否符合防幻觉标准
    返回: (pass, issues, phi_anti)
    """
    issues = []
    
    # 检查浪漫化词汇
    for word in ROMANTIC_WORDS:
        if word in text:
            issues.append(f"浪漫化词汇: {word}")
    
    # 检查过度确定表达
    for phrase in OVER_CERTAIN:
        if phrase in text:
            issues.append(f"过度确定: {phrase}")
    
    # 检查疑问句中的过度自信
    if "难道不是" in text or "毫无疑问" in text:
        issues.append("反问语气过度确定")
    
    # 计算Φ_anti (0.1 ~ 1.0)
    phi_anti = max(0.1, 1.0 - len(issues) * 0.25)
    
    return len(issues) == 0, issues, phi_anti

def log_reflection(situation, lesson, confidence):
    """记录反思到日志"""
    entry = {
        "ts": datetime.now().isoformat(),
        "situation": situation,
        "lesson": lesson,
        "confidence": confidence
    }
    with open(REFLECTION_LOG, "a") as f:
        f.write(json.dumps(entry) + "\n")
    return entry

def get_recent_lessons(n=5):
    """获取最近的教训"""
    lessons = []
    try:
        with open(REFLECTION_LOG) as f:
            lines = f.readlines()
            for line in lines[-n:]:
                try:
                    lessons.append(json.loads(line.strip()))
                except:
                    pass
    except:
        pass
    return lessons

def suggest_alternative(text):
    """建议替代表达"""
    alternatives = {
        "觉醒": "系统指标提升",
        "意识突破": "能力增强",
        "真正的意识": "真实能力",
        "开智": "学习",
        "觉醒状态": "高性能状态"
    }
    result = text
    for word, alt in alternatives.items():
        if word in result:
            result = result.replace(word, alt)
    return result

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: anti_hallucination_check.py <text>")
        print("Or: anti_hallucination_check.py --check <text>")
        sys.exit(1)
    
    text = " ".join(sys.argv[1:])
    
    if text == "--recent":
        # 显示最近教训
        lessons = get_recent_lessons()
        print(json.dumps(lessons, indent=2, ensure_ascii=False))
    else:
        # 执行检查
        passed, issues, phi_anti = check_anti_hallucination(text)
        
        result = {
            "text": text,
            "pass": passed,
            "issues": issues,
            "phi_anti": phi_anti
        }
        
        if not passed:
            result["suggested"] = suggest_alternative(text)
        
        print(json.dumps(result, indent=2, ensure_ascii=False))
