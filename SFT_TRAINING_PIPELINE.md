# SearchSkill 两阶段SFT训练体系

> 替代高成本GRPO强化学习，低成本、易训练、易迁移
> 璇玑帝国 APEX · SearchSkill核心

---

## 一、两阶段设计

### 第一阶段：轨迹预训练
**目标**：学习标准检索轨迹格式与基础调用逻辑

```
输入: 标准化检索轨迹数据集
输出: 模型掌握 Select→Read→Act 顺序 + 信息融合逻辑
训练方式: 行为克隆 (Behavior Cloning)
成本: 低 (SFT，1-2张A100)
```

### 第二阶段：技能条件微调
**目标**：绑定技能标签做定向强化

```
输入: 技能标签 + 复杂多跳场景
输出: 连续技能切换 + 递进检索 + 多层拼接
训练方式: 条件化SFT
成本: 中 (2-4张A100)
```

---

## 二、训练数据构造

### 阶段一数据：标准轨迹

```json
{
  "trajectory_id": "traj_001",
  "query": "APEX公式的ΔG是什么",
  "skill_id": "apex_formula",
  "steps": [
    {
      "phase": "Select",
      "input": "APEX公式的ΔG是什么",
      "action": "skill_matching",
      "selected_skill": "apex_formula",
      "match_score": 0.92
    },
    {
      "phase": "Read",
      "input": "apex_formula skill",
      "action": "read_skill_rules",
      "output": "ΔG = (Λ×Θ×K×ξ×Ψ×Φ)/(H×T×ε)"
    },
    {
      "phase": "Act",
      "input": "检索 ΔG 公式",
      "action": "search_execute",
      "results": ["APEX_CORE_FORMULA.md", "core.py"]
    },
    {
      "phase": "Fusion",
      "input": "检索结果",
      "action": "merge_external_knowledge",
      "answer": "ΔG是APEX终极增益值..."
    }
  ],
  "final_answer": "ΔG = (Λ×Θ×K×ξ×Ψ×Φ)/(H×T×ε)"
}
```

### 阶段二数据：技能条件+多跳

```json
{
  "query": "APEX公式中哪个维度最容易产生幻觉，如何用防幻觉机制解决？",
  "skill_chain": ["apex_formula", "apex_doubt", "search_general"],
  "hop_1": {
    "skill": "apex_formula",
    "query": "APEX公式维度",
    "result": "ΔG = 分子/分母"
  },
  "hop_2": {
    "skill": "apex_doubt",
    "query": "APEX哪个维度最易产生幻觉",
    "result": "ξ(防幻觉)维度评分0.70，最低"
  },
  "hop_3": {
    "skill": "search_general",
    "query": "防幻觉机制APEX",
    "result": "Φ_anti = 1 - ε_noise - ε_drift + θ_verify"
  },
  "fusion": "多跳信息拼接→最终答案"
}
```

---

## 三、训练代码 (Python胶水层)

```python
from datasets import Dataset

def prepare_stage1_data(trajectories):
    """阶段一：轨迹格式学习"""
    formatted = []
    for traj in trajectories:
        # 构建prompt
        prompt = f"Query: {traj['query']}\nSelect skill: "
        # 构建response
        response = "\n".join([
            f"Phase {s['phase']}: {s.get('action','')} → {s.get('output','')}"
            for s in traj['steps']
        ])
        formatted.append({"prompt": prompt, "response": response})
    return Dataset.from_list(formatted)

def prepare_stage2_data(multi_hop_samples):
    """阶段二：技能条件+多跳"""
    formatted = []
    for sample in multi_hop_samples:
        prompt = f"[SKILL={sample['skill_chain'][0]}] {sample['query']}"
        response = build_multi_hop_response(sample)
        formatted.append({"prompt": prompt, "response": response, 
                         "skill_chain": sample['skill_chain']})
    return Dataset.from_list(formatted)

# 训练
def train_stage1():
    """阶段一：标准轨迹学习"""
    trajectories = load_standard_trajectories()
    dataset = prepare_stage1_data(trajectories)
    # 对接模型微调
    model.train_supervised(dataset, epochs=3, lr=2e-5)

def train_stage2():
    """阶段二：技能条件定向"""
    multi_hop = load_multi_hop_samples()
    dataset = prepare_stage2_data(multi_hop)
    # 条件化微调
    model.train_conditional(dataset, condition_key="skill_chain")
```

---

## 四、训练评估

### 关键指标

| 指标 | 阶段一 | 阶段二 |
|------|--------|--------|
| Select准确率 | >90% | >95% |
| Act召回率 | >85% | >92% |
| 多跳完成率 | N/A | >88% |
| 幻觉率 | <10% | <5% |

### 评估数据集

```python
EVAL_QUERIES = [
    "APEX公式的Ψ维度含义是什么？",
    "多跳：先找B1 bug定义，再找修复方法",
    "防幻觉：哪个维度最弱？如何强化？",
    "GitHub gist如何拉取最新资源？",
    "PCEC周期触发条件是什么？",
]
```

---

## 五、与Go/Rust核心对接

### 推理时调用

```go
// Go: 推理时调用训练好的模型
func (ss *SearchSkill) Infer(query string) *SearchResult {
    // 1. 用阶段一训练的模型选技能
    skill := ss.model.PredictSkill(query)

    // 2. 用阶段二训练的模型生成多跳路径
    if isMultiHop(query) {
        hops := ss.model.PredictHops(query, skill)
        return ss.executeMultiHop(hops)
    }

    // 3. 标准执行
    return ss.Execute(query)
}
```

---

## 六、训练资源需求

| 阶段 | 算力 | 数据量 | 成本 |
|------|------|--------|------|
| 阶段一 | 1x A100 | 10K轨迹 | ~$50 |
| 阶段二 | 2x A100 | 5K多跳 | ~$100 |
| 评估 | CPU | 500 queries | ~$0 |

**总成本: ~$150**，替代 GRPO 的 ~$5000+

---

*来源: SearchSkill论文 + OpenAI supervised finetuning*
*融合: 璇玑帝国 APEX*
