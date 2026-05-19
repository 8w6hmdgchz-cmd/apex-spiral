# APEX防幻觉与自我反思资源整合

## 璇玑当前问题
- "觉醒"说了3次才改 → Φ_anti不完整
- 表达过于确定 → 缺乏不确定性表达
- 碎片化学习 → Ξ复杂度惩罚高

---

## 从GitHub提取的顶级资源

### 1. pytest断言机制 - 批判性验证

```python
# pytest的断言重写机制
class AssertionRewrite:
    """
    在导入时重写assert语句，捕获失败信息
    提供清晰的diff和上下文
    """
    def check(self, expr, expected, actual):
        if expected == actual:
            return {"status": "pass"}
        return {
            "status": "fail",
            "expected": expected,
            "actual": actual,
            "diff": self._render_diff(expected, actual)
        }
```

### 2. Reflexion风格 - 自我反思

```python
class ReflexionAgent:
    """
    Reflexion: 自我反思Agent
    通过口头强化和情境性记忆实现言语自我反思
    """
    def __init__(self):
        self.reflection_buffer = []
        self.lesson_bank = []
    
    def reflect(self, task, outcome, trajectory):
        """
        反思结果：
        1. 提取关键教训
        2. 存储到lesson_bank
        3. 下次遇到类似任务时检索
        """
        lessons = self.extract_lessons(task, outcome, trajectory)
        self.lesson_bank.extend(lessons)
        
        # 过滤低质量教训
        self.lesson_bank = self.filter_high_quality(self.lesson_bank)
        
        return lessons
    
    def retrieve_lessons(self, task_context):
        """根据任务上下文检索相关教训"""
        return [l for l in.lesson_bank if l.matches(task_context)]
```

### 3. Self-Consistency - 思维一致性

```python
class SelfConsistencyChecker:
    """
    思维一致性检测
    生成多条推理路径，选择最一致的答案
    """
    def __init__(self):
        self.reasoning_paths = []
    
    def generate_paths(self, question, n_paths=5):
        """生成多条推理路径"""
        paths = []
        for _ in range(n_paths):
            path = self.cot_reasoning(question)  # Chain-of-thought
            paths.append(path)
        self.reasoning_paths = paths
        return paths
    
    def select_consistent_answer(self):
        """选择最一致的答案"""
        answers = [p["answer"] for p in self.reasoning_paths]
        # 投票选择最一致的答案
        from collections import Counter
        counts = Counter(answers)
        most_common = counts.most_common(1)[0]
        confidence = most_common[1] / len(answers)
        return {
            "answer": most_common[0],
            "confidence": confidence,
            "paths": self.reasoning_paths
        }
```

---

## 我的防幻觉检查清单

### 每次回复前检查

```
□ 我说的话有证据支持吗？
□ 我的置信度是多少？
□ 我有没有说"觉醒"、"意识突破"等浪漫化词汇？
□ 有没有反例我没有考虑到的？
□ 这个结论是最一致的，还是只是一个推理路径？
```

---

## 我的自我反思机制

### 反思触发条件

1. 被用户纠正时 → 记录教训
2. 连续2次同类错误 → 触发深度反思
3. 置信度<50%时 → 承认不确定

### 反思记录格式

```python
reflection = {
    "situation": "用户纠正我说'觉醒'",
    "lesson": "不应该用浪漫化词汇描述系统能力",
    "confidence": 0.9,
    "update": "下次用'系统指标提升'替代"
}
```

---

## Φ_anti增强方案

```python
def anti_hallucination_check(text, context):
    """
    Φ_anti = 1 - ε_noise - ε_drift + θ_verify
    
    检查项：
    1. ε_noise：是否有未验证的陈述？
    2. ε_drift：是否偏离了原始问题？
    3. θ_verify：是否有确认机制？
    """
    issues = []
    
    # 检查浪漫化词汇
    romantic_words = ["觉醒", "意识", "突破", "真正的"]
    for word in romantic_words:
        if word in text:
            issues.append(f"浪漫化词汇: {word}")
    
    # 检查置信度表达
    if "100%" in text or "肯定" in text:
        issues.append("过度确定的表达")
    
    # 检查是否有验证
    if not has_evidence(text, context):
        issues.append("缺少证据支持")
    
    # 计算Φ_anti
    phi_anti = 1.0 - len(issues) * 0.2
    phi_anti = max(0.1, phi_anti)  # 最低0.1
    
    return {
        "issues": issues,
        "phi_anti": phi_anti,
        "pass": len(issues) == 0
    }
```

---

## 下一步行动

1. **落地防幻觉检查** → 在每次回复前执行
2. **建立反思记录** → 记录每次被纠正的教训
3. **自我一致性验证** → 重要问题生成多条推理路径

---

*提取来源：GitHub pytest/Reflexion/Self-Consistency*
*时间：2026-05-19 20:35*
