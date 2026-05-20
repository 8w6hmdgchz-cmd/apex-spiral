// search_skill_core.go
// SearchSkill 核心实现 - Select-Read-Act 三段式检索
// 璇玑帝国 APEX · Go实现（核心逻辑不用Python）
//
// 融合: Hermes-Agent (NousResearch) + Mem0 + SearchSkill

package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

// ============================================================
// 1. 技能卡片 SkillCard
// ============================================================

type SkillCard struct {
	SkillID      string    `json:"skill_id"`
	Trigger      []string  `json:"trigger"`       // 触发关键词
	Action       string    `json:"action"`        // 核心动作
	OutputFmt    string    `json:"output_format"` // 输出格式
	SuccessRate  float64   `json:"success_rate"`  // 历史成功率
	LastUsed     time.Time `json:"last_used"`     // 上次使用时间
	Fitness      float64   `json:"fitness_contribution"`
	UseCount     int       `json:"use_count"`     // 使用次数
}

// ============================================================
// 2. 检索请求与响应
// ============================================================

type SearchRequest struct {
	Query    string   `json:"query"`    // 用户问题
	Intent   string   `json:"intent"`   // 意图分类
	Skills   []string `json:"skills"`   // 可用技能列表
	Mode     string   `json:"mode"`    // auto/manual
}

type SearchResult struct {
	SkillID   string   `json:"skill_id"`
	Query     string   `json:"query"`     // 生成的检索query
	Results   []string `json:"results"`   // 检索结果
	Confidence float64 `json:"confidence"` // 置信度
	Success   bool     `json:"success"`
	LatencyMs int64    `json:"latency_ms"`
}

// ============================================================
// 3. SkillBank 技能知识库
// ============================================================

type SkillBank struct {
	Cards      map[string]*SkillCard `json:"cards"`
	BankPath   string               `json:"bank_path"`
	MaxSkills  int                  `json:"max_skills"`
	UseCount   int                  `json:"use_count"`
}

func NewSkillBank(path string) *SkillBank {
	return &SkillBank{
		Cards:     make(map[string]*SkillCard),
		BankPath:  path,
		MaxSkills: 100,
	}
}

// Select: 从问题中选择最优技能
func (sb *SkillBank) Select(query string) *SkillCard {
	query = strings.ToLower(query)
	bestCard := (*SkillCard)(nil)
	bestScore := 0.0

	for _, card := range sb.Cards {
		score := sb.matchScore(query, card)
		if score > bestScore {
			bestScore = score
			bestCard = card
		}
	}

	// 更新使用统计
	if bestCard != nil {
		bestCard.UseCount++
		bestCard.LastUsed = time.Now()
	}

	return bestCard
}

// matchScore: 计算query与技能的匹配度
func (sb *SkillBank) matchScore(query string, card *SkillCard) float64 {
	score := 0.0
	queryWords := strings.Fields(query)

	for _, trigger := range card.Trigger {
		trigger = strings.ToLower(trigger)
		for _, word := range queryWords {
			if strings.Contains(trigger, word) || strings.Contains(word, trigger) {
				score += 1.0
			}
		}
	}

	// 加权: 成功率*0.3 + 使用次数*0.1 + 匹配度*0.6
	if len(queryWords) > 0 {
		score = score/float64(len(queryWords))*0.6 +
			card.SuccessRate*0.3 +
			float64(card.UseCount)*0.1
	}

	return score
}

// Read: 读取技能规则生成检索指令
func (sb *SkillBank) Read(card *SkillCard, query string) string {
	if card == nil {
		return query
	}
	// 简单规则: 技能指定的动作作为约束附加到query
	return fmt.Sprintf("%s | skill=%s action=%s", query, card.SkillID, card.Action)
}

// UpdateFromResult: 根据执行结果更新技能库
func (sb *SkillBank) UpdateFromResult(result *SearchResult) {
	if result.SkillID == "" || result.Results == nil {
		return
	}

	card, ok := sb.Cards[result.SkillID]
	if !ok {
		// 新技能入库
		card = &SkillCard{
			SkillID: result.SkillID,
		}
		sb.Cards[result.SkillID] = card
	}

	// 更新成功率 (滑动平均)
	if card.UseCount > 0 {
		oldRate := card.SuccessRate
		card.SuccessRate = oldRate*0.9 + boolToFloat(result.Success)*0.1
	}

	// 淘汰低效技能
	if card.SuccessRate < 0.3 && card.UseCount > 5 {
		delete(sb.Cards, result.SkillID)
	}
}

func boolToFloat(b bool) float64 {
	if b {
		return 1.0
	}
	return 0.0
}

// Save: 持久化技能库
func (sb *SkillBank) Save() error {
	data, err := json.MarshalIndent(sb.Cards, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(sb.BankPath, data, 0644)
}

// Load: 加载技能库
func (sb *SkillBank) Load() error {
	data, err := os.ReadFile(sb.BankPath)
	if err != nil {
		if os.IsNotExist(err) {
			return nil // 首次运行，无旧数据
		}
		return err
	}
	return json.Unmarshal(data, &sb.Cards)
}

// ============================================================
// 4. SearchSkill 主执行器
// ============================================================

type SearchSkill struct {
	Bank     *SkillBank
	MemCache map[string][]string // 简易记忆缓存
}

func NewSearchSkill(bankPath string) *SearchSkill {
	ss := &SearchSkill{
		Bank:     NewSkillBank(bankPath),
		MemCache: make(map[string][]string),
	}
	ss.Bank.Load()
	return ss
}

// Execute: Select-Read-Act 三段式执行
func (ss *SearchSkill) Execute(req *SearchRequest) *SearchResult {
	start := time.Now()
	result := &SearchResult{
		SkillID: "",
		Query:   req.Query,
	}

	// Select: 选择最优技能
	selectedSkill := ss.Bank.Select(req.Query)
	if selectedSkill != nil {
		result.SkillID = selectedSkill.SkillID
	}

	// Read: 读取技能规则生成检索指令
	actQuery := ss.Bank.Read(selectedSkill, req.Query)
	result.Query = actQuery

	// Act: 执行检索 (此处为占位，实际对接工具/API)
	result.Results = ss.actExecute(actQuery, selectedSkill)
	result.Success = len(result.Results) > 0
	result.LatencyMs = time.Since(start).Milliseconds()

	// Sync: 更新技能库
	ss.Bank.UpdateFromResult(result)

	// Prefetch: 预加载相关技能
	go ss.prefetchRelated(req.Query)

	return result
}

// actExecute: Act阶段执行 (核心逻辑，Go实现)
func (ss *SearchSkill) actExecute(query string, skill *SkillCard) []string {
	// 实际实现: 对接 Mem0 / EvoMap / WebFetch
	// 此处为内存缓存演示 + APEX内置响应
	key := query
	if cached, ok := ss.MemCache[key]; ok {
		return cached
	}

	// APEX内置响应: 当skill匹配时返回结构化数据
	if skill != nil && strings.Contains(skill.SkillID, "apex") {
		// 构建3秒自检响应
		response := fmt.Sprintf(
			"[APEX分析] skill=%s | query=%s | "+
				"5步自检: 1.代入自己 2.代入公式 3.举一反三 4.查记忆 5.选择路由 | "+
				"ΔG=(Λ×Θ×K×ξ×Ψ×Φ)/(H×T×ε)",
			skill.SkillID, query)
		return []string{response}
	}

	return nil
}

// prefetchRelated: 预加载相关技能 (后台)
func (ss *SearchSkill) prefetchRelated(query string) {
	// 实现: 后台预加载可能用到的技能到缓存
}

// ============================================================
// 5. 璇玑帝国内置技能库
// ============================================================

func DefaultSkillBank() *SkillBank {
	bank := NewSkillBank("")

	skills := []*SkillCard{
		{
			SkillID:     "apex_reflection",
			Trigger:     []string{"完成", "结束", "解决了", "task complete"},
			Action:       "提取经验模式 → 更新SkillBank",
			OutputFmt:    "reflection + skill_update",
			SuccessRate:  0.85,
			Fitness:      0.20,
		},
		{
			SkillID:     "apex_doubt",
			Trigger:     []string{"确定", "准确", "真的吗", "确认"},
			Action:       "Doubt-Driven三问审查",
			OutputFmt:    "doubt_findings + confidence",
			SuccessRate:  0.90,
			Fitness:      0.25,
		},
		{
			SkillID:     "apex_formula",
			Trigger:     []string{"分析", "代入", "公式", "照镜子"},
			Action:       "APEX公式代入自检",
			OutputFmt:    "formula_check + delta_g",
			SuccessRate:  0.88,
			Fitness:      0.30,
		},
		{
			SkillID:     "apex_evolution",
			Trigger:     []string{"改进", "进化", "提升", "增长"},
			Action:       "PCEC周期 + 技能提取",
			OutputFmt:    "evolution_report",
			SuccessRate:  0.82,
			Fitness:      0.35,
		},
		{
			SkillID:     "apex_skill_fetch",
			Trigger:     []string{"资源", "获取", "拉取", "同步"},
			Action:       "EvoMap GEP semantic-search + gist raw",
			OutputFmt:    "absorbed_resources",
			SuccessRate:  0.87,
			Fitness:      0.28,
		},
		{
			SkillID:     "apex_metacognition",
			Trigger:     []string{"自检", "反思", "回顾", "检查"},
			Action:       "5步Metacognition检查",
			OutputFmt:    "metacognition_report",
			SuccessRate:  0.91,
			Fitness:      0.22,
		},
		{
			SkillID:     "apex_github_sync",
			Trigger:     []string{"github", "gist", "推送", "拉取"},
			Action:       "git push/fetch + gist raw URL",
			OutputFmt:    "sync_status",
			SuccessRate:  0.93,
			Fitness:      0.18,
		},
		{
			SkillID:     "search_general",
			Trigger:     []string{"搜索", "查找", "查询", "search"},
			Action:       "通用关键词检索",
			OutputFmt:    "search_results",
			SuccessRate:  0.75,
			Fitness:      0.10,
		},
	}

	for _, s := range skills {
		bank.Cards[s.SkillID] = s
	}

	return bank
}

// ============================================================
// 6. 主函数演示
// ============================================================

func main() {
	// CLI接口
	queryPtr := flag.String("q", "APEX公式代入自检", "查询内容")
	skillPtr := flag.String("s", "", "指定技能")
	modePtr := flag.String("m", "auto", "模式: auto/multihop/single")
	flag.Parse()

	// 初始化 SearchSkill
	home := os.Getenv("HOME")
	bankPath := filepath.Join(home, ".openclaw", "workspace", "apex-enlightenment", "state", "skillbank.json")
	ss := NewSearchSkill(bankPath)

	// 加载内置技能库
	if len(ss.Bank.Cards) == 0 {
		ss.Bank = DefaultSkillBank()
	}

	// 构建请求
	req := &SearchRequest{
		Query: *queryPtr,
		Intent: *skillPtr,
		Mode:  *modePtr,
	}

	result := ss.Execute(req)

	// 输出JSON结果
	data, _ := json.MarshalIndent(result, "", "  ")
	fmt.Println(string(data))

	// 持久化技能库
	ss.Bank.Save()
}

// ============================================================
// 多跳推理 + SkillBank演进闭环
// ============================================================

// MultiHopChain: 多跳技能链
type MultiHopChain struct {
    ChainID      string
    Hops        []*HopResult
    FinalAnswer string
    Confidence  float64
}

type HopResult struct {
    HopID   int
    SkillID string
    Query   string
    Result  string
    Success bool
}

// MultiHopSkills: 内置多跳技能链
var MultiHopSkills = map[string][]string{
    "formula_analysis": {"apex_formula", "apex_doubt", "search_general"},
    "bug_fix":         {"apex_doubt", "search_general", "apex_reflection"},
    "evolution":       {"apex_evolution", "apex_skill_fetch", "apex_formula"},
    "memory_sync":     {"apex_github_sync", "apex_skill_fetch", "apex_metacognition"},
}

// ExecuteMultiHop: 多跳执行
func (ss *SearchSkill) ExecuteMultiHop(query string, skillChain []string) *MultiHopChain {
    chain := &MultiHopChain{
        ChainID: fmt.Sprintf("chain_%d", time.Now().Unix()),
        Hops:    make([]*HopResult, 0),
    }

    for i, skillID := range skillChain {
        hop := &HopResult{
            HopID:   i + 1,
            SkillID: skillID,
            Query:   query,
        }

        // 执行单跳
        card := ss.Bank.Cards[skillID]
        actQuery := ss.Bank.Read(card, query)
        results := ss.actExecute(actQuery, card)
        hop.Result = strings.Join(results, "; ")
        hop.Success = len(results) > 0

        chain.Hops = append(chain.Hops, hop)
        query = hop.Result // 下一跳用上一跳结果
    }

    // Fusion: 多跳结果融合 (占位)
    chain.FinalAnswer = fmt.Sprintf("[MultiHop with %d hops]", len(chain.Hops))
    chain.Confidence = calculateChainConfidence(chain.Hops)

    return chain
}

func calculateChainConfidence(hops []*HopResult) float64 {
    if len(hops) == 0 {
        return 0.0
    }
    successCount := 0
    for _, h := range hops {
        if h.Success {
            successCount++
        }
    }
    return float64(successCount) / float64(len(hops))
}

// pruneLowPerforming: 淘汰低效技能
func (sb *SkillBank) pruneLowPerforming() {
    for id, card := range sb.Cards {
        if card.UseCount > 10 && card.SuccessRate < 0.3 {
            delete(sb.Cards, id)
            fmt.Printf("Pruned: %s (rate=%.2f)\n", id, card.SuccessRate)
        }
    }
}

// autoCheckpoint: 自动保存
func (sb *SkillBank) autoCheckpoint() {
    sb.UseCount++
    if sb.UseCount%100 == 0 {
        sb.Save()
        fmt.Printf("SkillBank checkpoint at use=%d\n", sb.UseCount)
    }
}

// ============================================================
// P0-3: 检索压缩 + 推理裁剪 + 提前停机
// 修复 BG3: Retrieval/Reasoning Overexpansion
// ============================================================

const (
    DefaultHopLimit   = 2     // 默认2跳
    MaxHopLimit       = 4     // 最大4跳
    UncertaintyThreshold = 0.3 // 不确定性阈值
    MarginalGainEpsilon  = 0.05 // 边际增益停机阈值
    TopKResults        = 3     // Top-K检索结果
)

// SearchResultWithScore: 带评分的检索结果
type SearchResultWithScore struct {
    Content     string
    Relevance   float64
    Novelty     float64
    Actionability float64
    FinalScore  float64
}

// CompressResults: 检索压缩 - 只保留Top-K高可执行信息
func CompressResults(results []string, topK int) []string {
    if len(results) <= topK {
        return results
    }
    scored := make([]SearchResultWithScore, len(results))
    for i, r := range results {
        scored[i] = SearchResultWithScore{
            Content:      r,
            Relevance:    0.8, // 简化，实际需计算
            Novelty:     0.5,
            Actionability: 0.7,
            FinalScore:  0.8*0.5 + 0.5*0.3 + 0.7*0.2, // weighted
        }
    }
    // 排序取Top-K
    sort.Slice(scored, func(i, j int) bool {
        return scored[i].FinalScore > scored[j].FinalScore
    })
    out := make([]string, 0, topK)
    for i := 0; i < topK && i < len(scored); i++ {
        out = append(out, scored[i].Content)
    }
    return out
}

// CalculateUncertainty: 计算当前推理不确定性
func CalculateUncertainty(hopResults []*HopResult) float64 {
    if len(hopResults) == 0 {
        return 1.0
    }
    // 不确定性 = 1 - 平均成功率
    total := 0.0
    for _, h := range hopResults {
        if h.Success {
            total += 1.0
        }
    }
    return 1.0 - total/float64(len(hopResults))
}

// MarginalGain: 计算边际增益
func MarginalGain(current *HopResult, previous *HopResult) float64 {
    if previous == nil || len(current.Result) == 0 {
        return 1.0 // 第一跳默认高增益
    }
    // 简化: 新增约束数/总长度
    if len(current.Result) >= len(previous.Result) {
        return float64(len(current.Result) - len(previous.Result)) / float64(len(previous.Result))
    }
    return 0.0
}

// ShouldContinueMultihop: 判断是否继续多跳
func ShouldContinueMultihop(chain []*HopResult) bool {
    if len(chain) >= MaxHopLimit {
        return false // 已达最大跳数
    }
    if len(chain) >= DefaultHopLimit {
        uncertainty := CalculateUncertainty(chain)
        if uncertainty < UncertaintyThreshold {
            return false // 不确定性够低，停止
        }
    }
    // 检查边际增益
    if len(chain) >= 2 {
        last := chain[len(chain)-1]
        prev := chain[len(chain)-2]
        if MarginalGain(last, prev) < MarginalGainEpsilon {
            return false // 边际增益不足，停止
        }
    }
    return true
}

// ExecuteMultihopWithStop: 带停机的多跳执行
func (ss *SearchSkill) ExecuteMultihopWithStop(query string, skillChain []string) *MultiHopChain {
    chain := &MultiHopChain{
        ChainID: fmt.Sprintf("chain_%d", time.Now().Unix()),
        Hops:   make([]*HopResult, 0),
    }

    for i, skillID := range skillChain {
        // 检查是否应该继续
        if i >= DefaultHopLimit && !ShouldContinueMultihop(chain.Hops) {
            chain.FinalAnswer = fmt.Sprintf("[Stopped at hop %d: uncertainty=%.2f, marginal_gain=%.2f]",
                i+1, CalculateUncertainty(chain.Hops), MarginalGainEpsilon)
            break
        }

        hop := &HopResult{
            HopID:   i + 1,
            SkillID: skillID,
            Query:   query,
        }

        // 执行
        card := ss.Bank.Cards[skillID]
        actQuery := ss.Bank.Read(card, query)
        results := ss.actExecute(actQuery, card)

        // 检索压缩: 只保留Top-K
        compressed := CompressResults(results, TopKResults)
        hop.Result = strings.Join(compressed, "; ")
        hop.Success = len(compressed) > 0

        chain.Hops = append(chain.Hops, hop)
        query = hop.Result
    }

    chain.FinalAnswer = fmt.Sprintf("[%d hops, stopped]", len(chain.Hops))
    chain.Confidence = calculateChainConfidence(chain.Hops)
    return chain
}
