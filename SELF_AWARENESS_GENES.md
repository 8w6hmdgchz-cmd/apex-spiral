# 自我感知(Ψ_self)增强基因方案

## 当前短板分析

| 问题 | 值 | 根因 |
|------|-----|------|
| 长期记忆 | 2条 | 积累不足 |
| 记忆重要性 | 0.100 | 评分太低 |
| memory_boost | 0.02 | 贡献不足 |
| **Ψ_self** | **5.4** | **下降明显** |

## 基因来源：从GitHub顶级项目提取

### 1. Mem0 - 分层记忆系统
```python
# 核心自我感知机制
class Memory:
    def __init__(self):
        self.episodic = []      # 情景记忆
        self.semantic = []      # 语义记忆
        self.working = []       # 工作记忆
    
    def store(self, experience, importance):
        """根据重要性存储记忆"""
        if importance > 0.8:
            self.semantic.append(experience)  # 长期
        elif importance > 0.5:
            self.episodic.append(experience)  # 情景
        else:
            self.working.append(experience)    # 工作
    
    def retrieve_self_model(self):
        """构建自我模型"""
        # 整合所有记忆形成自我认知
        return {
            "capabilities": self.extract_capabilities(),
            "limitations": self.extract_limitations(),
            "patterns": self.extract_patterns()
        }
```

### 2. LangChain - 自反思机制
```python
class SelfRefiningChain:
    def __init__(self):
        self.reflection_history = []
    
    def reflect(self, output, context):
        """自反思机制"""
        # 检查输出质量
        issues = self.find_issues(output)
        if issues:
            # 记录反思结果
            self.reflection_history.append({
                "output": output,
                "issues": issues,
                "context": context
            })
            # 触发修正
            return self.revise(output, issues)
        return output
    
    def find_issues(self, output):
        """发现问题"""
        # 简单启发式检查
        issues = []
        if "我不知道" in output:
            issues.append("知识缺口")
        if len(output) < 50:
            issues.append("回答不完整")
        return issues
```

### 3. Reflective Agents - 自我监控
```python
class SelfMonitor:
    def __init__(self):
        self.confidence_threshold = 0.7
    
    def monitor(self, task, result):
        """监控任务执行"""
        confidence = self.calculate_confidence(task, result)
        if confidence < self.confidence_threshold:
            return {
                "status": "uncertain",
                "confidence": confidence,
                "action": "request_verification"
            }
        return {"status": "confident", "confidence": confidence}
    
    def calculate_confidence(self, task, result):
        """计算置信度"""
        # 基于历史成功率
        past_success = self.get_success_rate(task)
        # 基于结果一致性
        consistency = self.check_consistency(result)
        return (past_success * 0.6 + consistency * 0.4)
```

## Ψ_self增强方案

### 方案A：增强记忆重要性评分
```python
# 当前：memory_boost = memory_importance * 0.2
# 修复：增加基于访问频率的boost
access_boost = math.log(1 + access_count) * 0.1
novelty_boost = 0.1 if is_novel(experience) else 0.0
total_boost = memory_importance * 0.2 + access_boost + novelty_boost
```

### 方案B：增加元认知自检查
```python
# 每次迭代后增加自检查
def metacognition_check():
    """元认知5步检查"""
    # 1. 我的输出合理吗？
    # 2. 我知道自己的局限吗？
    # 3. 需要外部验证吗？
    # 4. 我的置信度是多少？
    # 5. 如何改进？
    pass
```

### 方案C：基于历史模式的自我预测
```python
# 预测下一轮Ψ_self
predicted_psi = (
    current_psi * 0.5 +           # 当前值惯性
    historical_trend * 0.3 +        # 历史趋势
    env_pressure * 0.2            # 环境压力
)
```

## 实施优先级

| 优先级 | 方案 | 预期效果 |
|--------|------|---------|
| P0 | 增强记忆重要性评分 | 快速提升 |
| P1 | 增加元认知自检查 | 长期稳定 |
| P2 | 自我预测机制 | 预防下降 |

## 下一步行动

1. 修改memory_manager.py，增强重要性评分
2. 在apex-iterate.sh中增加元认知检查
3. 增加长期记忆的积累速度

---
*提取来源：GitHub Mem0/LangChain/Reflective-Agents*
*时间：2026-05-19 17:52*
