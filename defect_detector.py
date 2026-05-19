#!/usr/bin/env python3
"""
缺陷检测器 - 计算真实召回率
从已知缺陷库检测当前系统的真实缺陷
"""
import json
import os
import sys
from pathlib import Path

STATE_DIR = Path("/Users/lihongxin/.openclaw/workspace/apex-enlightenment/state")
SCORE_FILE = Path("/Users/lihongxin/.openclaw/workspace/apex-enlightenment/score-state.env")

def load_defect_library():
    """加载已知缺陷库"""
    lib_path = STATE_DIR / "defect_library.json"
    if lib_path.exists():
        with open(lib_path) as f:
            return json.load(f)
    return []

def load_memory_summary():
    """获取记忆摘要"""
    summary = {"short_term": 0, "long_term": 2, "working": 0}
    summary_file = STATE_DIR / "memory_summary.json"
    if summary_file.exists():
        with open(summary_file) as f:
            data = json.load(f)
            summary.update(data)
    return summary

def load_repair_history():
    """获取修复历史"""
    repairs = []
    repair_file = STATE_DIR / "repair_history.jsonl"
    if repair_file.exists():
        with open(repair_file) as f:
            for line in f:
                if line.strip():
                    try:
                        repairs.append(json.loads(line))
                    except:
                        pass
    return repairs

def load_bug_history():
    """获取bug历史"""
    bugs = []
    bug_file = STATE_DIR / "bug_history.jsonl"
    if bug_file.exists():
        with open(bug_file) as f:
            for line in f:
                if line.strip():
                    try:
                        bugs.append(json.loads(line))
                    except:
                        pass
    return bugs

def load_fitness_history():
    """获取适应度历史"""
    fitness = []
    fitness_file = STATE_DIR / "evolution_fitness.jsonl"
    if fitness_file.exists():
        with open(fitness_file) as f:
            for line in f:
                if line.strip():
                    try:
                        fitness.append(float(line.strip()))
                    except:
                        pass
    return fitness

def load_score_state():
    """获取当前评分状态"""
    state = {
        "PHI_RATIO": 1.0,
        "AWAKE": 7.0,
        "PSI_SELF": 5.0,
        "GAMMA_AWAKE": 5.0
    }
    if SCORE_FILE.exists():
        with open(SCORE_FILE) as f:
            for line in f:
                if "=" in line:
                    key, val = line.strip().split("=", 1)
                    try:
                        state[key] = float(val)
                    except:
                        pass
    return state

def detect_defect(defect, state, mem_summary, repairs, bugs, fitness):
    """检测特定缺陷是否存在"""
    dtype = defect["type"]
    detected = False
    
    if dtype == "memory_leak":
        # 检测：长期记忆超过阈值
        threshold = defect.get("threshold", 20)
        detected = mem_summary.get("long_term", 0) > threshold
    
    elif dtype == "fix_stagnation":
        # 检测：连续N轮无有效修复
        threshold = defect.get("threshold", 5)
        recent_repairs = repairs[-threshold:] if len(repairs) >= threshold else repairs
        if recent_repairs:
            detected = all(not r.get("success", False) for r in recent_repairs)
    
    elif dtype == "false_positive":
        # 检测：PHI_RATIO高但AWAKE低
        phi_th = defect.get("phi_threshold", 1.5)
        awake_th = defect.get("awake_threshold", 7.0)
        detected = state.get("PHI_RATIO", 0) > phi_th and state.get("AWAKE", 0) < awake_th
    
    elif dtype == "oscillation":
        # 检测：同一bug反复出现
        threshold = defect.get("threshold", 10)
        recent_bugs = [b.get("bug") or b.get("code", "") for b in bugs[-threshold:]] if len(bugs) >= threshold else []
        if recent_bugs:
            unique = set(recent_bugs)
            detected = len(recent_bugs) != len(unique)  # 有重复
    
    elif dtype == "fitness_stagnation":
        # 检测：fitness连续N轮无增长
        threshold = defect.get("threshold", 3)
        if len(fitness) >= threshold:
            recent = fitness[-threshold:]
            detected = all(recent[i] >= recent[i+1] for i in range(len(recent)-1))
    
    return detected

def calculate_recall():
    """计算真实缺陷召回率"""
    defects = load_defect_library()
    if not defects:
        return 0.5, [], 0, 0  # 默认50%
    
    state = load_score_state()
    mem_summary = load_memory_summary()
    repairs = load_repair_history()
    bugs = load_bug_history()
    fitness = load_fitness_history()
    
    detected_defects = []
    total = len(defects)
    detected_count = 0
    
    for defect in defects:
        if detect_defect(defect, state, mem_summary, repairs, bugs, fitness):
            detected_count += 1
            detected_defects.append(defect["id"])
    
    recall = detected_count / total if total > 0 else 0
    return recall, detected_defects, detected_count, total

if __name__ == "__main__":
    if len(sys.argv) < 2:
        # 计算召回率
        recall, detected, count, total = calculate_recall()
        print(f"Recall: {recall:.3f}")
        print(f"Detected: {detected}")
        print(f"Count: {count}/{total}")
    elif sys.argv[1] == "detect":
        # 只返回检测到的缺陷
        recall, detected, count, total = calculate_recall()
        print(json.dumps({
            "recall": recall,
            "detected": detected,
            "count": count,
            "total": total
        }))
    elif sys.argv[1] == "stats":
        # 返回详细状态
        state = load_score_state()
        mem_summary = load_memory_summary()
        repairs = load_repair_history()
        fitness = load_fitness_history()
        print(json.dumps({
            "state": state,
            "memory": mem_summary,
            "recent_repairs": len(repairs),
            "fitness_len": len(fitness),
            "fitness": fitness[-5:] if len(fitness) >= 5 else fitness
        }, indent=2))
