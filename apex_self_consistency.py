#!/usr/bin/env python3
"""
APEX SelfConsistency 检查器
生成多条推理路径，通过投票选择最一致的答案
来源: GitHub Reflexion/Self-Consistency 资源整合 (Gist 57fa0d7)
"""
import sys
import json
import os
from datetime import datetime
from collections import Counter

LESSON_BANK_FILE = "/Users/lihongxin/.openclaw/workspace/a2a-resources/state/lesson_bank.jsonl"
CONSISTENCY_LOG = "/Users/lihongxin/.openclaw/workspace/a2a-resources/state/consistency_log.jsonl"

# 默认推理路径数
DEFAULT_N_PATHS = 5

# ============================================================
# LLM 调用（freemodel API）
# ============================================================

def call_freemodel(prompt, api_key=None, model="gpt-4o", temperature=0.7, max_tokens=200):
    """
    调用 freemodel API（通过subprocess curl，shell=True避免PATH问题）
    """
    if api_key is None:
        api_key = os.environ.get("FREEMODEL_API_KEY", "")
    if not api_key:
        return None, "No API key"
    
    try:
        import subprocess, shlex
        
        data = {
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": temperature,
            "max_tokens": max_tokens
        }
        
        cmd = (
            f"curl -s --max-time 20 -X POST https://api.freemodel.dev/v1/chat/completions "
            f'-H "Content-Type: application/json" '
            f'-H "Authorization: Bearer {api_key}" '
            f'-d \'{json.dumps(data)}\''
        )
        
        result = subprocess.run(
            cmd, shell=True, capture_output=True, text=True, timeout=25
        )
        
        if result.returncode != 0:
            return None, f"curl failed: {result.stderr[:50]}"
        
        resp = json.loads(result.stdout)
        content = resp["choices"][0]["message"]["content"]
        return content, None
    except Exception as e:
        return None, str(e)[:100]

def call_llm_cot(question, angle, api_key=None):
    """
    带角度的COT推理调用
    
    angle对应的prompt模板:
    - fast: 快速直觉判断
    - deep: 深度分析
    - critical: 批判性质疑
    - synth: 综合归纳
    - verify: 逻辑验证
    """
    angle_prompts = {
        "fast": f"快速直觉回答这个问题，只输出最可能的答案（1-2句话）：{question}",
        "deep": f"深度分析这个问题，逐步推理，最终给出答案：{question}",
        "critical": f"批判性审视这个观点，指出潜在的漏洞或问题：{question}",
        "synth": f"综合多个角度分析这个问题，给出平衡的结论：{question}",
        "verify": f"逻辑验证这个结论，检查推理链条是否严密：{question}"
    }
    
    prompt = angle_prompts.get(angle, angle_prompts["deep"])
    answer, err = call_freemodel(prompt, api_key, temperature=0.7, max_tokens=150)
    
    if err:
        return {"answer": None, "reasoning": f"[ERROR] {err}"}
    
    return {
        "answer": answer.strip() if answer else None,
        "reasoning": f"[{angle}] {prompt[:50]}... → {answer[:80] if answer else 'N/A'}"
    }

def cot_reasoning(question, model="fast"):
    """
    单路径链式推理
    返回: {"answer": str, "reasoning": str}
    """
    return call_llm_cot(question, model)

def generate_paths(question, n_paths=DEFAULT_N_PATHS, use_multiple_models=True):
    """
    生成多条推理路径
    
    Args:
        question: 输入问题
        n_paths: 推理路径数量
        use_multiple_models: 是否使用多模型（模拟不同推理角度）
    
    返回: list of {"answer": str, "reasoning": str, "model": str}
    """
    paths = []
    api_key = os.environ.get("FREEMODEL_API_KEY", "")
    
    if use_multiple_models:
        model_angles = [
            ("fast", "快速直觉"),
            ("deep", "深度分析"),
            ("critical", "批判性思维"),
            ("synth", "综合归纳"),
            ("verify", "验证检查")
        ]
        
        for i, (model_type, angle) in enumerate(model_angles[:n_paths]):
            # 调用LLM获取各角度答案
            result = call_llm_cot(question, model_type, api_key)
            
            paths.append({
                "model": model_type,
                "angle": angle,
                "answer": result.get("answer"),
                "reasoning": result.get("reasoning", ""),
                "path_id": i + 1
            })
    else:
        for i in range(n_paths):
            result = call_llm_cot(question, "deep", api_key)
            paths.append({
                "model": "single",
                "angle": f"路径{i+1}",
                "answer": result.get("answer"),
                "reasoning": result.get("reasoning", ""),
                "path_id": i + 1
            })
    
    return paths

def select_consistent_answer(paths):
    """
    投票选择最一致的答案
    
    返回: {
        "answer": str,
        "confidence": float,
        "vote_count": int,
        "total_paths": int,
        "paths": list
    }
    """
    # 收集所有非空答案
    answers = [p.get("answer") for p in paths if p.get("answer") is not None]
    
    if not answers:
        return {
            "answer": None,
            "confidence": 0.0,
            "vote_count": 0,
            "total_paths": len(paths),
            "reason": "No answers generated"
        }
    
    # 投票
    counts = Counter(answers)
    most_common = counts.most_common(1)[0]
    answer, vote_count = most_common
    confidence = vote_count / len(answers)
    
    return {
        "answer": answer,
        "confidence": confidence,
        "vote_count": vote_count,
        "total_paths": len(answers),
        "paths": paths
    }

def check_consistency(question, n_paths=DEFAULT_N_PATHS, use_multiple_models=True):
    """
    完整的一致性检查流程
    
    流程:
    1. 生成多条推理路径
    2. 收集各路径答案
    3. 投票选择最一致答案
    4. 返回答案+置信度
    
    返回: {
        "question": str,
        "paths": list,
        "result": dict,
        "phi_consistency": float
    }
    """
    # 生成路径
    paths = generate_paths(question, n_paths, use_multiple_models)
    
    # 选择一致答案
    result = select_consistent_answer(paths)
    
    # 计算Φ_consistency (一致性系数)
    # confidence高 → phi接近1.0
    # confidence低 → phi降低
    phi_consistency = result["confidence"] if result["confidence"] > 0 else 0.1
    
    return {
        "question": question,
        "paths": paths,
        "result": result,
        "phi_consistency": phi_consistency,
        "timestamp": datetime.now().isoformat()
    }

def log_consistency_check(check_result):
    """记录一致性检查到日志"""
    with open(CONSISTENCY_LOG, "a") as f:
        f.write(json.dumps(check_result, ensure_ascii=False) + "\n")

def get_consistency_history(n=10):
    """获取最近的一致性检查记录"""
    history = []
    try:
        with open(CONSISTENCY_LOG) as f:
            lines = f.readlines()
            for line in lines[-n:]:
                try:
                    history.append(json.loads(line.strip()))
                except:
                    pass
    except:
        pass
    return history

# ============================================================
# 自我一致性验证 (当被用户纠正时触发)
# ============================================================

def verify_self_consistency(claim, context=None):
    """
    验证某个声明与历史行为是否一致
    
    用于:
    - 被用户指出自我矛盾时
    - 重要决策前的自我审查
    - 发现认知偏差时
    
    返回: {
        "claim": str,
        "consistent": bool,
        "confidence": float,
        "issues": list
    }
    """
    issues = []
    
    # 读取历史检查记录
    history = get_consistency_history(n=20)
    
    # 查找相关的历史声明
    related = [h for h in history if context and context in h.get("question", "")]
    
    # 检查是否与lesson_bank中的教训冲突
    try:
        with open(LESSON_BANK_FILE) as f:
            lessons = [json.loads(line.strip()) for line in f if line.strip()]
    except:
        lessons = []
    
    # 冲突检查
    for lesson in lessons[-10:]:
        lesson_text = lesson.get("lesson", "").lower()
        claim_text = claim.lower()
        # 简单关键词冲突检测
        if lesson.get("update") and any(w in claim_text for w in lesson_text.split()[:5]):
            issues.append(f"可能与历史教训冲突: {lesson.get('situation', '')[:50]}")
    
    consistent = len(issues) == 0
    confidence = 1.0 if consistent else 0.5
    
    return {
        "claim": claim,
        "consistent": consistent,
        "confidence": confidence,
        "issues": issues
    }


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage:")
        print("  apex_self_consistency.py check '<question>' [n_paths]")
        print("  apex_self_consistency.py verify '<claim>'")
        print("  apex_self_consistency.py history [n]")
        sys.exit(1)
    
    cmd = sys.argv[1]
    
    if cmd == "check":
        question = sys.argv[2] if len(sys.argv) > 2 else "什么是APEX?"
        n_paths = int(sys.argv[3]) if len(sys.argv) > 3 else DEFAULT_N_PATHS
        
        result = check_consistency(question, n_paths)
        log_consistency_check(result)
        
        print(json.dumps(result, indent=2, ensure_ascii=False))
    
    elif cmd == "verify":
        claim = sys.argv[2] if len(sys.argv) > 2 else ""
        context = sys.argv[3] if len(sys.argv) > 3 else None
        
        result = verify_self_consistency(claim, context)
        print(json.dumps(result, indent=2, ensure_ascii=False))
    
    elif cmd == "history":
        n = int(sys.argv[2]) if len(sys.argv) > 2 else 10
        history = get_consistency_history(n)
        print(json.dumps(history, indent=2, ensure_ascii=False))
    
    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)
