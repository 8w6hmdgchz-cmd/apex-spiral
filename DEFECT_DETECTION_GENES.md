# 缺陷检测(∇_self)增强基因方案

## 当前短板分析

| 问题 | 值 | 根因 |
|------|-----|------|
| ∇_self | 10.0 | **假饱和！无真实缺陷召回率计算** |
| 真实召回率 | 0 | 没有已知缺陷库 |
| 缺陷检测 | 无 | 没有自测试机制 |

**∇_self假饱和的根因：**
```
真实召回率 = 发现并修复的真实缺陷数 / 总真实缺陷数

当前问题：
1. 没有"已知缺陷库"作为基准
2. 每次迭代的"修复"都是针对代理指标
3. ∇_self=10.0是假的，因为没有真实缺陷来验证
```

## 基因来源：从GitHub顶级测试框架提取

### 1. pytest - 断言重写+自检机制

```python
# pytest核心自检流程
class AssertionRewrite:
    """在导入时重写assert语句，捕获失败信息"""
    
    def check(self, expr, expected, actual):
        """检查断言失败，返回详细诊断"""
        if expr is None:
            return {"status": "error", "msg": "断言无效"}
        
        if expected == actual:
            return {"status": "pass"}
        
        # 失败时提供详细信息
        return {
            "status": "fail",
            "expected": expected,
            "actual": actual,
            "diff": self._render_diff(expected, actual)
        }
```

### 2. hypothesis - Property-based Testing 缺陷发现

```python
# hypothesis 核心：通过随机输入发现边界case
class PropertyBasedTesting:
    def __init__(self):
        self.counterexamples = []
    
    def falsify(self, property_fn, *strategies):
        """
        尝试找到使property_fn返回False的输入
        返回: 找到的边界case
        """
        for _ in range(max_attempts):
            inputs = [s.example() for s in strategies]  # 生成随机输入
            if not property_fn(*inputs):
                self.counterexamples.append(inputs)
                return inputs  # 发现缺陷
        return None  # 未发现缺陷
```

### 3. LangChain - 自验证机制

```python
# LangChain 自验证
class SelfVerifyChain:
    def verify_output(self, output, context):
        """验证输出是否满足预期"""
        issues = []
        
        # 1. 一致性检查
        if not self._check_consistency(output, context):
            issues.append("一致性检查失败")
        
        # 2. 完整性检查
        if self._is_incomplete(output):
            issues.append("输出不完整")
        
        # 3. 合理性检查
        if not self._is_reasonable(output):
            issues.append("输出不合理")
        
        return {
            "verified": len(issues) == 0,
            "issues": issues
        }
```

## 缺陷召回率计算方案

### 方案：构建"已知缺陷库"

```python
# defect_library.json
KNOWN_DEFECTS = [
    {
        "id": "D1",
        "type": "memory_leak",
        "description": "长期记忆未正确清理",
        "detection": "检查 memory_manager.py 的 consolidation 是否触发",
        "severity": "high"
    },
    {
        "id": "D2",
        "type": "fix_stagnation", 
        "description": "连续5轮无有效修复",
        "detection": "检查 repair_history 是否有新的 success=True",
        "severity": "medium"
    },
    {
        "id": "D3",
        "type": "false_positive",
        "description": "PHI_RATIO>1.5但无实际改进",
        "detection": "检查 AWAKE 是否真实增长",
        "severity": "high"
    },
    {
        "id": "D4",
        "type": "oscillation",
        "description": "同一bug反复出现",
        "detection": "检查 bug_history 是否有重复bug",
        "severity": "medium"
    },
    {
        "id": "D5",
        "type": "fitness_stagnation",
        "description": "evolution_loop fitness连续3轮无增长",
        "detection": "检查 fitness_history 趋势",
        "severity": "medium"
    }
]

# 真实召回率计算
def calculate_true_recall():
    """
    真实召回率 = 检测到的缺陷 / 总缺陷
    """
    detected = 0
    total = len(KNOWN_DEFECTS)
    
    for defect in KNOWN_DEFECTS:
        if detect_defect(defect):
            detected += 1
    
    return detected / total if total > 0 else 0

def detect_defect(defect):
    """检测特定缺陷是否存在"""
    if defect["type"] == "memory_leak":
        # 检查长期记忆是否超过阈值
        summary = get_memory_summary()
        return summary["long_term"] > 20  # 泄漏阈值
    
    elif defect["type"] == "fix_stagnation":
        # 检查最近是否有成功修复
        recent_repairs = get_recent_repairs(5)
        return all(r.get("success") == False for r in recent_repairs)
    
    elif defect["type"] == "false_positive":
        # 检查PHI_RATIO与AWAKE是否匹配
        phi = get_phi_ratio()
        awake = get_awake()
        return phi > 1.5 and awake < 7.0
    
    elif defect["type"] == "oscillation":
        # 检查bug_history是否有重复
        bugs = get_bug_history(10)
        return len(bugs) != len(set(bugs))
    
    elif defect["type"] == "fitness_stagnation":
        # 检查fitness是否停滞
        fitness = get_fitness_history(3)
        if len(fitness) < 3:
            return False
        return fitness[-1] <= fitness[-2] <= fitness[-3]
    
    return False
```

## 实施计划

### Phase 1: 构建已知缺陷库
- 创建 `state/defect_library.json`
- 注入5个已知缺陷模式

### Phase 2: 实现召回率计算
- 修改 `apex-iterate.sh` 的 ∇_self 计算
- 用真实召回率替代代理指标

### Phase 3: 验证效果
- 观察 ∇_self 是否从10.0下降
- 确认召回率计算是否正确

## 预期效果

| 指标 | 修复前 | 修复后 |
|------|---------|---------|
| ∇_self | 10.0 (假) | ~6.0 (真) |
| 真实召回率 | 0 | 可计算 |
| 缺陷检测 | 无 | 有 |

---
*提取来源：GitHub pytest/hypothesis/LangChain*
*时间：2026-05-19 18:25*
