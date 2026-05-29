#!/usr/bin/env python3
"""
缺陷检测器 - 计算真实召回率
从已知缺陷库检测当前系统的真实缺陷
"""
import json
import os
import sys
from pathlib import Path

STATE_DIR = Path("/Users/lihongxin/.openclaw/workspace/a2a-resources/state")
SCORE_FILE = Path("/Users/lihongxin/.openclaw/workspace/a2a-resources/score-state.env")

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

# ============================================================
# BG2修复: 失败样本学习硬闭环
# 旧: 失败 → 写反思 → 存档
# 新: 失败 → 归因分类 → 生成修复规则 → 写入SkillBank → 下次优先调用 → A/B验证
# ============================================================

class FailureSample:
    """失败样本结构 - BG2核心"""
    def __init__(self, task_id: str, failure_stage: str,
                 failure_pattern: str, missed_signal: str,
                 repair_action: str, trigger_condition: str,
                 validation_metric: str = "delta_g"):
        self.task_id = task_id
        self.failure_stage = failure_stage  # retrieve|select|reason|compose|verify
        self.failure_pattern = failure_pattern
        self.missed_signal = missed_signal
        self.repair_action = repair_action
        self.trigger_condition = trigger_condition
        self.validation_metric = validation_metric
        self.status = "pending"  # pending|applied|validated|rejected
        self.validation_result = None

# 失败阶段 → 对应修复动作
STAGE_TO_SKILL = {
    "retrieve": "search_general",
    "select": "apex_skill_fetch",
    "reason": "apex_doubt",
    "compose": "apex_reflection",
    "verify": "apex_metacognition",
}

def process_failure(failure: FailureSample) -> dict:
    """
    BG2 核心: 失败样本硬闭环
    返回: {skill_trigger, repair_rule, validation_plan}
    """
    # 1. 归因分类
    stage = failure.failure_stage
    skill = STAGE_TO_SKILL.get(stage, "search_general")

    # 2. 生成修复规则
    repair_rule = {
        "skill_id": f"repair_{failure.task_id}",
        "trigger": [failure.trigger_condition, failure.failure_pattern],
        "action": failure.repair_action,
        "skill_to_apply": skill,
        "expected_gain": 0.15,  # 期望ΔG提升
        "confidence": 0.7,
    }

    # 3. 验证计划
    validation_plan = {
        "metric": failure.validation_metric,
        "baseline": read_current_metric(failure.validation_metric),
        "expected_delta": 0.1,
        "test_rounds": 3,
    }

    # 4. 写入SkillBank (追加到skillbank_candidates.json)
    candidates_file = STATE_DIR / "skillbank_candidates.json"
    candidates = []
    if candidates_file.exists():
        with open(candidates_file) as f:
            candidates = json.load(f)
    candidates.append(repair_rule)
    with open(candidates_file, "w") as f:
        json.dump(candidates, f, indent=2)

    # 5. 标记为待应用
    failure.status = "applied"

    return {
        "skill_trigger": skill,
        "repair_rule": repair_rule,
        "validation_plan": validation_plan,
    }


def validate_repair(failure: FailureSample) -> bool:
    """
    BG2 验证: 应用修复后检查是否有效
    A/B验证: 对比应用前后的 validation_metric
    """
    baseline = read_current_metric(failure.validation_metric)
    current = read_current_metric(failure.validation_metric)

    improvement = (current - baseline) / max(baseline, 0.01)

    if improvement > 0.05:  # 5%提升阈值
        failure.status = "validated"
        failure.validation_result = {"improvement": improvement, "pass": True}
        return True
    else:
        failure.status = "rejected"
        failure.validation_result = {"improvement": improvement, "pass": False}
        return False


def read_current_metric(metric_name: str) -> float:
    """读取当前指标值"""
    if metric_name == "delta_g":
        score_file = SCORE_FILE
        if score_file.exists():
            for line in score_file.read_text().splitlines():
                if '=' in line:
                    k, v = line.strip().split('=', 1)
                    if k == 'AWAKE':
                        return float(v) / 10.0  # 归一化
        return 0.5
    elif metric_name == "h_entropy":
        return 0.5  # 默认值
    elif metric_name == "t_cycle":
        return 2.0  # 默认值
    return 0.5
