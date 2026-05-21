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
LESSON_BANK_FILE = "/Users/lihongxin/.openclaw/workspace/apex-enlightenment/state/lesson_bank.jsonl"

# ============================================================
# Reflexion-style Lesson Bank (来源: GitHub Gist 57fa0d7)
# ============================================================

def filter_high_quality(lessons, min_confidence=0.6, max_lessons=50):
    """
    过滤低质量教训，保留高置信度样本
    
    规则:
    1. confidence >= min_confidence
    2. 教训有实际内容（非空）
    3. 最多保留 max_lessons 条
    """
    filtered = []
    for lesson in lessons:
        conf = lesson.get("confidence", 0)
        lesson_text = lesson.get("lesson", "").strip()
        situation = lesson.get("situation", "").strip()
        
        # 质量门槛
        if conf >= min_confidence and len(lesson_text) > 10 and len(situation) > 5:
            filtered.append(lesson)
    
    # 按confidence降序，保留前max_lessons条
    filtered.sort(key=lambda x: x.get("confidence", 0), reverse=True)
    return filtered[:max_lessons]

def retrieve_lessons(task_context, lessons=None, top_k=5):
    """
    根据任务上下文检索相关教训
    
    匹配策略:
    1. 关键词重叠
    2. 任务类型相似
    3. 时间衰减（近期优先）
    """
    if lessons is None:
        lessons = get_recent_lessons(n=100)
    
    if not lessons:
        return []
    
    context_words = set(task_context.lower().split())
    scored = []
    
    for lesson in lessons:
        score = 0
        
        # 关键词重叠
        lesson_text = (lesson.get("lesson", "") + " " + lesson.get("situation", "")).lower()
        lesson_words = set(lesson_text.split())
        overlap = len(context_words & lesson_words)
        score += overlap * 2
        
        # 时间衰减（近30天权重更高）
        try:
            from datetime import datetime, timedelta
            ts = lesson.get("ts", "")
            if ts:
                lesson_date = datetime.fromisoformat(ts)
                days_ago = (datetime.now() - lesson_date).days
                if days_ago <= 30:
                    score += (30 - days_ago) / 30 * 3  # 最多+3分
        except:
            pass
        
        # 置信度加权
        score += lesson.get("confidence", 0.5) * 2
        
        scored.append((score, lesson))
    
    # 降序排列，返回top_k
    scored.sort(key=lambda x: x[0], reverse=True)
    return [lesson for _, lesson in scored[:top_k]]

def extract_lessons(task, outcome, trajectory):
    """
    从任务执行结果中提取教训
    
    来源: Reflexion Agent
    
    Args:
        task: 任务描述
        outcome: 执行结果
        trajectory: 完整轨迹
    
    返回: list of lessons
    """
    lessons = []
    
    # 简单规则提取（实际应该用模型分析）
    # 失败场景
    if "fail" in outcome.lower() or "error" in outcome.lower() or "wrong" in outcome.lower():
        lessons.append({
            "situation": task,
            "lesson": f"任务失败: {outcome[:100]}",
            "confidence": 0.8,
            "update": "下次遇到类似任务优先检查此处"
        })
    
    # 纠正场景
    if "纠正" in outcome or "应该" in outcome:
        lessons.append({
            "situation": task,
            "lesson": outcome[:100],
            "confidence": 0.9,
            "update": "已被验证的正确做法"
        })
    
    return lessons

def reflect_and_update(task, outcome, trajectory):
    """
    Reflexion风格的反思+更新流程
    
    1. 提取教训
    2. 存入lesson_bank
    3. 过滤低质量教训
    """
    # 提取教训
    new_lessons = extract_lessons(task, outcome, trajectory)
    
    # 读取现有教训
    existing = get_recent_lessons(n=100)
    
    # 合并
    all_lessons = existing + new_lessons
    
    # 过滤
    filtered = filter_high_quality(all_lessons)
    
    # 写回lesson_bank
    with open(LESSON_BANK_FILE, "w") as f:
        for lesson in filtered:
            f.write(json.dumps(lesson, ensure_ascii=False) + "\n")
    
    return {
        "extracted": len(new_lessons),
        "total_stored": len(filtered)
    }

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

def get_recent_lessons(n=50):
    """获取最近的教训（从reflection_log和lesson_bank合并）"""
    lessons = []
    
    # 优先从lesson_bank读
    try:
        with open(LESSON_BANK_FILE) as f:
            for line in f:
                try:
                    lessons.append(json.loads(line.strip()))
                except:
                    pass
    except:
        pass
    
    # 再从reflection_log补充
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
        print("Usage:")
        print("  anti_hallucination_check.py check '<text>'        # 检查文本")
        print("  anti_hallucination_check.py --recent [n]          # 显示最近教训")
        print("  anti_hallucination_check.py retrieve '<context>' [k] # 检索相关教训")
        print("  anti_hallucination_check.py reflect '<task>' '<outcome>' '<trajectory>'")
        print("  anti_hallucination_check.py log '<situation>' '<lesson>' <confidence>")
        sys.exit(1)
    
    cmd = sys.argv[1]
    
    if cmd == "check":
        text = sys.argv[2] if len(sys.argv) > 2 else ""
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
    
    elif cmd == "--recent" or cmd == "recent":
        n = int(sys.argv[2]) if len(sys.argv) > 2 else 50
        lessons = get_recent_lessons(n)
        print(json.dumps(lessons, indent=2, ensure_ascii=False))
    
    elif cmd == "retrieve":
        context = sys.argv[2] if len(sys.argv) > 2 else ""
        top_k = int(sys.argv[3]) if len(sys.argv) > 3 else 5
        lessons = get_recent_lessons(100)
        relevant = retrieve_lessons(context, lessons, top_k)
        print(json.dumps(relevant, indent=2, ensure_ascii=False))
    
    elif cmd == "reflect":
        if len(sys.argv) < 4:
            print("Usage: reflect '<task>' '<outcome>' '<trajectory>'")
            sys.exit(1)
        task = sys.argv[2]
        outcome = sys.argv[3]
        trajectory = sys.argv[4] if len(sys.argv) > 4 else ""
        result = reflect_and_update(task, outcome, trajectory)
        print(json.dumps(result, indent=2, ensure_ascii=False))
    
    elif cmd == "log":
        if len(sys.argv) < 4:
            print("Usage: log '<situation>' '<lesson>' <confidence>")
            sys.exit(1)
        situation = sys.argv[2]
        lesson = sys.argv[3]
        confidence = float(sys.argv[4])
        entry = log_reflection(situation, lesson, confidence)
        print(json.dumps(entry, indent=2, ensure_ascii=False))
    
    else:
        # 兼容旧语法
        text = " ".join(sys.argv[1:])
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
