# 多跳深度推理检索

> 解决长链路信息缺失、检索偏移问题
> 璇玑帝国 APEX · SearchSkill核心

---

## 一、问题定义

**多跳场景**：单次检索无法回答，需要多轮递进检索

```
问题: "APEX公式里哪个维度最容易产生幻觉，如何用防幻觉机制修复？"

跳1: "APEX公式的维度有哪些" → 需要知道有哪些维度
跳2: "哪个维度评分最低(ξ=0.70)" → 需要代入自身
跳3: "防幻觉机制是什么" → 需要找修复方案
```

---

## 二、多跳推理架构

### SkillChain 多跳技能链

```go
// 多跳技能链
type HopSkill struct {
    HopID    int
    Skill    string
    Query    string
    Result   string
    NextHop  *HopSkill
}

type MultiHopChain struct {
    ChainID   string
    Hops      []*HopSkill
    FinalAns  string
    Confidence float64
}
```

### 璇玑帝国内置多跳技能

```go
var MultiHopSkills = map[string][]string{
    "formula_analysis": {"apex_formula", "apex_doubt", "search_general"},
    "bug_fix":         {"apex_doubt", "search_general", "apex_reflection"},
    "evolution":       {"apex_evolution", "apex_skill_fetch", "apex_formula"},
    "memory_sync":     {"apex_github_sync", "apex_skill_fetch", "apex_metacognition"},
}
```

---

## 三、多跳执行流程

```
输入: "APEX哪个维度最弱？如何修复？"
  ↓
[Hop 1] Select: apex_formula
  ↓ "APEX公式有哪些维度"
  ↓ Result: Λ, Θ, K, ξ, Ψ, Φ, H, T, ε
  ↓
[Hop 2] Select: apex_doubt  
  ↓ "哪个维度评分最低"
  ↓ Result: ξ=0.70 (最低)
  ↓
[Hop 3] Select: apex_skill_fetch
  ↓ "防幻觉机制修复方案"
  ↓ Result: Φ_anti = 1 - ε_noise - ε_drift + θ_verify
  ↓
[Fusion] 多跳结果拼接 → 最终答案
```

---

## 四、Fusion融合层

```go
// 多跳结果融合
func (mhc *MultiHopChain) Fusion() string {
    // 1. 收集所有跳的结果
    var context strings.Builder
    for _, hop := range mhc.Hops {
        context.WriteString(fmt.Sprintf("[Hop%d-%s] %s\n",
            hop.HopID, hop.Skill, hop.Result))
    }

    // 2. 用LLM融合 (对接GPT/Claude)
    prompt := fmt.Sprintf(`
根据以下多跳检索结果，回答原问题：

%s

原问题: %s

要求：
1. 按逻辑顺序整合各跳信息
2. 指出关键发现
3. 给出具体方案
4. 置信度评估
`, context.String(), mhc.OriginalQuery)

    answer := llm.Generate(prompt)
    return answer
}
```

---

## 五、多跳搜索策略

### 依赖图策略

```
问题拆解 → 识别跳数 → 确定每跳技能 → 执行链

例子: "APEX + Hermes融合方案"
  拆解: ["APEX维度", "Hermes机制", "融合点"]
  跳数: 3跳
  技能链: apex_formula → apex_evolution → search_general
```

### 动态跳数判断

```go
func (mhc *MultiHopChain) estimateHopCount(query string) int {
    // 关键词判断跳数
    multiHopKeywords := []string{"如何", "怎么", "为什么", "哪个", "什么"}
    count := 0
    for _, kw := range multiHopKeywords {
        if strings.Contains(query, kw) {
            count++
        }
    }
    // 基础跳数
    if count == 0 {
        return 1
    }
    // 限制最大跳数
    return min(count+1, 5)
}
```

---

## 六、失败处理

### 某跳失败时的策略

```go
func (mhc *MultiHopChain) handleHopFailure(hopID int, err error) {
    // 策略1: 回退到单跳
    if hopID > 1 {
        log.Warn("Hop %d failed, falling back to single-hop", hopID)
        mhc.FallbackToSingleHop()
        return
    }

    // 策略2: 跳过当前跳，继续后续跳
    log.Warn("Hop %d failed, skipping to next", hopID)
    mhc.skipHop(hopID)

    // 策略3: 使用缓存结果
    if cached := mhc.getCachedResult(hopID); cached != "" {
        mhc.Hops[hopID].Result = cached
    }
}
```

---

## 七、与SkillBank集成

### 多跳技能自动发现

```go
func (sb *SkillBank) discoverMultiHopChain(query string) []string {
    // 分析问题关键词
    keywords := extractKeywords(query)

    // 查找相关技能
    var chain []string
    for _, kw := range keywords {
        if skill := sb.findBestSkill(kw); skill != "" {
            chain = append(chain, skill)
        }
    }

    // 去重 + 保持顺序
    chain = unique(preserveOrder(chain))

    return chain
}
```

---

## 八、评估指标

| 指标 | 目标 | 说明 |
|------|------|------|
| 跳数准确率 | >90% | 跳数估计准确 |
| 中间结果正确率 | >85% | 每跳结果有效 |
| 最终答案准确率 | >88% | 融合结果正确 |
| 幻觉率 | <5% | 无错误信息注入 |

---

*来源: HotpotQA + MuSiQue + SearchSkill*
*融合: 璇玑帝国 APEX*
