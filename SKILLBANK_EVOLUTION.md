# SkillBank 动态演进闭环

> 技能库自主演进，无需人工大规模标注
> 璇玑帝国 APEX · SearchSkill核心

---

## 一、演进机制

### 三角闭环

```
    检索执行
        ↓
    结果反馈 → 成功样本/失败案例
        ↓               ↓
    技能更新 ← 失败分析 ← 失效原因
        ↓
    知识蒸馏 → 新技能生成
        ↓
    自动入库 → 技能体系升级
```

---

## 二、自动技能生成

### 失败案例 → 新技能

```go
type FailureAnalyzer struct{}

func (fa *FailureAnalyzer) Analyze(trajectory Trajectory) *NewSkill {
    // 1. 识别失败点
    failureHop := fa.findFailureHop(trajectory)

    // 2. 归纳失败模式
    pattern := fa.inducePattern(failureHop)

    // 3. 生成新技能
    if fa.isNovelPattern(pattern) {
        return &NewSkill{
            SkillID:    generateSkillID(),
            Trigger:    pattern.keywords,
            Action:     pattern.solution,
            OutputFmt:  pattern.output,
            Confidence: pattern.support, // 支持度
        }
    }
    return nil
}

// 失败模式归纳
func (fa *FailureAnalyzer) inducePattern(hop *Hop) *FailurePattern {
    return &FailurePattern{
        keywords:  extractKeywords(hop.query),
        solution:  extractSolution(hop.failedResult),
        support:   calculateSupport(hop),
        confidence: hop.successRate,
    }
}
```

---

## 三、技能淘汰机制

### 低效技能自动淘汰

```go
func (sb *SkillBank) pruneLowPerforming() {
    var toRemove []string

    for id, card := range sb.cards {
        // 淘汰条件
        if card.useCount > 10 && card.successRate < 0.3 {
            toRemove = append(toRemove, id)
        }
    }

    // 批量删除
    for _, id := range toRemove {
        log.Info("Pruning low-performer: %s (rate=%.2f)", id, sb.cards[id].successRate)
        delete(sb.cards, id)
    }
}
```

---

## 四、知识蒸馏

### 多个相似技能 → 优质核心技能

```go
func (sb *SkillBank) distill() {
    // 1. 找到相似技能群
    clusters := sb.clusterSimilarSkills()

    // 2. 每个簇蒸馏出一个核心技能
    for _, cluster := range clusters {
        if len(cluster) >= 2 {
            distilled := sb.distillCluster(cluster)
            sb.cards[distilled.skillID] = distilled
            // 删除原技能
            for _, id := range cluster {
                if id != distilled.skillID {
                    delete(sb.cards, id)
                }
            }
        }
    }
}

// 蒸馏: 加权融合多个技能
func (sb *SkillBank) distillCluster(cluster []string) *SkillCard {
    var totalWeight float64
    var weightedTrigger []string
    var weightedAction string

    for _, id := range cluster {
        card := sb.cards[id]
        weight := card.useCount * card.successRate
        totalWeight += weight

        // 触发词加权融合
        for _, t := range card.trigger {
            weightedTrigger = append(weightedTrigger, t)
        }
    }

    return &SkillCard{
        SkillID:     "distilled_" + cluster[0],
        Trigger:     deduplicate(weightedTrigger),
        Action:      weightedAction,
        SuccessRate: totalWeight / float64(len(cluster)),
        UseCount:    100, // 蒸馏技能高优先级
    }
}
```

---

## 五、成功样本积累

### 自动收集高置信度轨迹

```go
type TrajectoryCollector struct {
    minConfidence float64
    storagePath   string
}

func (tc *TrajectoryCollector) shouldStore(result *SearchResult) bool {
    // 置信度 > 0.85 的成功轨迹才存储
    return result.confidence >= tc.minConfidence && result.success
}

func (tc *TrajectoryCollector) store(trajectory Trajectory) {
    path := fmt.Sprintf("%s/traj_%d.json", tc.storagePath, time.Now().Unix())
    data, _ := json.Marshal(trajectory)
    os.WriteFile(path, data, 0644)
}
```

---

## 六、SkillBank 持久化

### 自动checkpoint

```go
func (sb *SkillBank) autoCheckpoint() {
    // 每100次使用自动保存
    if sb.useCount % 100 == 0 {
        sb.Save()
        log.Info("SkillBank auto-checkpoint at use=%d", sb.useCount)
    }
}

// 与GitHub同步
func (sb *SkillBank) syncToGitHub(repoPath string) error {
    // 序列化
    data, _ := json.MarshalIndent(sb.cards, "", "  ")
    os.WriteFile(repoPath+"/skillbank.json", data, 0644)

    // Git push
    return runGitCommand(repoPath, "add", "skillbank.json",
        "commit", "-m", fmt.Sprintf("SkillBank auto-sync %d", sb.useCount),
        "push", "origin", "main")
}
```

---

## 七、演进指标

| 指标 | 目标 | 当前 |
|------|------|------|
| 技能总数 | 动态增长 | 8个 |
| 平均成功率 | >85% | ~85% |
| 淘汰率/月 | <5% | — |
| 新技能/月 | >3个 | — |
| 蒸馏次数 | 按需 | — |

---

## 八、璇玑帝国技能演进记录

```json
{
  "skillbank_evolution": {
    "last_updated": "2026-05-20T15:50:00+08:00",
    "total_skills": 8,
    "high_performers": [
      {"id": "apex_github_sync", "rate": 0.93, "uses": 47},
      {"id": "apex_metacognition", "rate": 0.91, "uses": 35},
      {"id": "apex_doubt", "rate": 0.90, "uses": 52}
    ],
    "candidates_for_distillation": [
      {"id": "search_general", "rate": 0.75, "uses": 12}
    ],
    "new_skills_discovered": [],
    "skills_pruned": []
  }
}
```

---

*来源: Reflexion + Self-Evolution + SkillBank*
*融合: 璇玑帝国 APEX*
