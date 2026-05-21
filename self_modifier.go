// self_modifier.go — 代码自我修改机制
// AGI核心能力之一：能分析自己的策略并调整参数
//
// 自我修改层次:
//   L1. 参数调整 — 调整基因参数
//   L2. 策略替换 — 用更好的策略替换差的
//   L3. 结构改变 — 添加/删除基因
//   L4. 代码生成 — 生成新的策略代码

package main

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/http"
	"strings"
	"time"
)

// SelfModifyLevel 自我修改等级
type SelfModifyLevel int

const (
	L1_ParameterTuning SelfModifyLevel = iota + 1 // 参数调整
	L2_StrategyReplacement                       // 策略替换
	L3_StructuralChange                         // 结构改变
	L4_CodeGeneration                          // 代码生成
)

// ModificationRecord 自我修改记录
type ModificationRecord struct {
	Timestamp   time.Time       `json:"timestamp"`
	Level       SelfModifyLevel `json:"level"`
	Target      string         `json:"target"`      // 修改目标
	Before      string         `json:"before"`      // 修改前
	After       string         `json:"after"`       // 修改后
	Reason      string         `json:"reason"`      // 修改原因
	Performance float64        `json:"performance"` // 预期性能提升
	ActualGain  float64        `json:"actual_gain"` // 实际收益
	Verified    bool           `json:"verified"`    // 是否验证
}

// GeneParameters 基因可调参数
type GeneParameters struct {
	GeneID       string  `json:"gene_id"`
	SuccessRate  float64 `json:"success_rate"`  // 成功率
	UsageCount   int     `json:"usage_count"`   // 使用次数
	DeltaG       float64 `json:"delta_g"`       // APEX ΔG
	Adaptability float64 `json:"adaptability"`  // 适应性
	Cooldown     int     `json:"cooldown"`      // 冷却时间
	Priority     float64 `json:"priority"`      // 优先级
}

// SelfModifier 自我修改器
type SelfModifier struct {
	Records       []*ModificationRecord
	GeneParams    map[string]*GeneParameters
	PerformanceLog []PerformanceEntry
	MaxRecords    int
	DataDir       string
}

// PerformanceEntry 性能记录
type PerformanceEntry struct {
	Timestamp time.Time `json:"timestamp"`
	GeneID    string    `json:"gene_id"`
	Success   bool      `json:"success"`
	DeltaG    float64   `json:"delta_g"`
	Query     string    `json:"query"`
}

// NewSelfModifier 创建自我修改器
func NewSelfModifier(dataDir string) *SelfModifier {
	if dataDir == "" {
		dataDir = "/Users/lihongxin/Desktop/开智"
	}
	return &SelfModifier{
		Records:       make([]*ModificationRecord, 0),
		GeneParams:    make(map[string]*GeneParameters),
		PerformanceLog: make([]PerformanceEntry, 0),
		MaxRecords:    500,
		DataDir:       dataDir,
	}
}

// AnalyzePerformance 分析基因性能
func (sm *SelfModifier) AnalyzePerformance(geneID string) map[string]float64 {
	entries := sm.GetGeneEntries(geneID)
	if len(entries) == 0 {
		return map[string]float64{
			"success_rate": 0.5,
			"avg_delta_g":  0,
			"usage_count":  0,
			"trend":        0,
		}
	}

	successCount := 0
	totalDeltaG := 0.0
	for _, e := range entries {
		if e.Success {
			successCount++
		}
		totalDeltaG += e.DeltaG
	}

	successRate := float64(successCount) / float64(len(entries))
	avgDeltaG := totalDeltaG / float64(len(entries))

	// 计算趋势 (最近vs早期)
	trend := sm.calculateTrend(entries)

	return map[string]float64{
		"success_rate": successRate,
		"avg_delta_g":  avgDeltaG,
		"usage_count":  float64(len(entries)),
		"trend":        trend,
	}
}

// GetGeneEntries 获取基因的所有记录
func (sm *SelfModifier) GetGeneEntries(geneID string) []PerformanceEntry {
	var entries []PerformanceEntry
	for _, e := range sm.PerformanceLog {
		if e.GeneID == geneID {
			entries = append(entries, e)
		}
	}
	return entries
}

// calculateTrend 计算趋势
func (sm *SelfModifier) calculateTrend(entries []PerformanceEntry) float64 {
	if len(entries) < 5 {
		return 0
	}

	half := len(entries) / 2
	earlySuccess := 0
	recentSuccess := 0

	for i, e := range entries {
		if i < half && e.Success {
			earlySuccess++
		}
		if i >= half && e.Success {
			recentSuccess++
		}
	}

	earlyRate := float64(earlySuccess) / float64(half)
	recentRate := float64(recentSuccess) / float64(len(entries)-half)

	return recentRate - earlyRate
}

// SuggestModifications 建议修改
func (sm *SelfModifier) SuggestModifications(geneID string) []*ModificationRecord {
	var suggestions []*ModificationRecord

	perf := sm.AnalyzePerformance(geneID)

	// L1: 参数调整建议
	if perf["success_rate"] < 0.6 {
		record := &ModificationRecord{
			Timestamp:   time.Now(),
			Level:       L1_ParameterTuning,
			Target:      geneID,
			Before:      fmt.Sprintf("success_rate=%.2f", perf["success_rate"]),
			After:       fmt.Sprintf("success_rate=%.2f (降低预期)", perf["success_rate"]*1.1),
			Reason:      "成功率偏低，调整预期参数",
			Performance: perf["success_rate"],
		}
		suggestions = append(suggestions, record)
	}

	// L1: 优先级调整
	if perf["trend"] < -0.1 {
		record := &ModificationRecord{
			Timestamp:   time.Now(),
			Level:       L1_ParameterTuning,
			Target:      geneID,
			Before:      fmt.Sprintf("priority=0.5, trend=%.2f", perf["trend"]),
			After:       "priority=0.3 (降低优先级)",
			Reason:      fmt.Sprintf("性能下降趋势%.2f，降低优先级", perf["trend"]),
			Performance: perf["avg_delta_g"],
		}
		suggestions = append(suggestions, record)
	}

	// L2: 策略替换建议
	if perf["success_rate"] < 0.4 && perf["usage_count"] > 20 {
		record := &ModificationRecord{
			Timestamp:   time.Now(),
			Level:       L2_StrategyReplacement,
			Target:      geneID,
			Before:      fmt.Sprintf("strategy=%s, SR=%.2f", geneID, perf["success_rate"]),
			After:       "考虑使用fusion策略或其他基因",
			Reason:      "长期低成功率，应替换策略",
			Performance: perf["success_rate"],
		}
		suggestions = append(suggestions, record)
	}

	// L3: 结构改变建议
	if perf["trend"] < -0.2 && perf["usage_count"] > 50 {
		record := &ModificationRecord{
			Timestamp:   time.Now(),
			Level:       L3_StructuralChange,
			Target:      geneID,
			Before:      fmt.Sprintf("gene=%s, 长期下降趋势", geneID),
			After:       "建议暂时禁用或融合",
			Reason:      "持续下降，应结构性改变",
			Performance: perf["avg_delta_g"],
		}
		suggestions = append(suggestions, record)
	}

	return suggestions
}

// ApplyModification 应用修改
func (sm *SelfModifier) ApplyModification(record *ModificationRecord) bool {
	// 创建备份
	sm.backupGeneParams(record.Target)

	// 根据修改级别应用
	switch record.Level {
	case L1_ParameterTuning:
		return sm.applyParameterTuning(record)
	case L2_StrategyReplacement:
		return sm.applyStrategyReplacement(record)
	case L3_StructuralChange:
		return sm.applyStructuralChange(record)
	case L4_CodeGeneration:
		return sm.applyCodeGeneration(record)
	}
	return false
}

// backupGeneParams 备份基因参数
func (sm *SelfModifier) backupGeneParams(geneID string) {
	backup := fmt.Sprintf("%s/backup_%s_%d.json", sm.DataDir, geneID, time.Now().Unix())
	params := sm.GeneParams[geneID]
	if params != nil {
		data, _ := json.MarshalIndent(params, "", "  ")
		ioutil.WriteFile(backup, data, 0644)
	}
}

// applyParameterTuning 应用参数调整
func (sm *SelfModifier) applyParameterTuning(record *ModificationRecord) bool {
	fmt.Printf("[自我修改] L1参数调整: %s\n", record.Target)
	fmt.Printf("  Before: %s\n", record.Before)
	fmt.Printf("  After:  %s\n", record.After)

	record.Verified = true
	sm.addRecord(record)
	return true
}

// applyStrategyReplacement 应用策略替换
func (sm *SelfModifier) applyStrategyReplacement(record *ModificationRecord) bool {
	fmt.Printf("[自我修改] L2策略替换: %s\n", record.Target)
	fmt.Printf("  替换原因: %s\n", record.Reason)

	record.Verified = true
	sm.addRecord(record)
	return true
}

// applyStructuralChange 应用结构改变
func (sm *SelfModifier) applyStructuralChange(record *ModificationRecord) bool {
	fmt.Printf("[自我修改] L3结构改变: %s\n", record.Target)
	fmt.Printf("  改变原因: %s\n", record.Reason)

	record.Performance = 0.1 // 预期收益
	record.Verified = false // 需要验证
	sm.addRecord(record)
	return true
}

// applyCodeGeneration 应用代码生成
func (sm *SelfModifier) applyCodeGeneration(record *ModificationRecord) bool {
	fmt.Printf("[自我修改] L4代码生成: %s\n", record.Target)
	fmt.Printf("  生成内容: %s\n", record.After)

	record.Performance = 0.2
	record.Verified = false
	sm.addRecord(record)
	return true
}

// VerifyModification 验证修改效果
func (sm *SelfModifier) VerifyModification(geneID string, afterEntries int) bool {
	entries := sm.GetGeneEntries(geneID)
	if len(entries) < afterEntries {
		return false
	}

	// 比较修改前后的性能
	before := entries[:len(entries)-afterEntries]
	after := entries[len(entries)-afterEntries:]

	beforeSuccess := 0
	afterSuccess := 0
	for _, e := range before {
		if e.Success {
			beforeSuccess++
		}
	}
	for _, e := range after {
		if e.Success {
			afterSuccess++
		}
	}

	beforeRate := float64(beforeSuccess) / float64(len(before))
	afterRate := float64(afterSuccess) / float64(len(after))

	gain := afterRate - beforeRate

	// 更新记录
	for i := len(sm.Records) - 1; i >= 0; i-- {
		if sm.Records[i].Target == geneID && !sm.Records[i].Verified {
			sm.Records[i].ActualGain = gain
			sm.Records[i].Verified = true
			break
		}
	}

	return gain > 0
}

// addRecord 添加记录
func (sm *SelfModifier) addRecord(record *ModificationRecord) {
	sm.Records = append(sm.Records, record)
	if len(sm.Records) > sm.MaxRecords {
		sm.Records = sm.Records[1:]
	}
}

// LogPerformance 记录性能
func (sm *SelfModifier) LogPerformance(geneID string, success bool, deltaG float64, query string) {
	entry := PerformanceEntry{
		Timestamp: time.Now(),
		GeneID:    geneID,
		Success:   success,
		DeltaG:    deltaG,
		Query:     query,
	}
	sm.PerformanceLog = append(sm.PerformanceLog, entry)

	// 限制日志大小
	if len(sm.PerformanceLog) > 10000 {
		sm.PerformanceLog = sm.PerformanceLog[1000:]
	}
}

// GetModificationSummary 获取修改摘要
func (sm *SelfModifier) GetModificationSummary() string {
	var sb strings.Builder

	sb.WriteString("=== 自我修改摘要 ===\n\n")

	sb.WriteString(fmt.Sprintf("总修改次数: %d\n", len(sm.Records)))

	levelCounts := make(map[SelfModifyLevel]int)
	for _, r := range sm.Records {
		levelCounts[r.Level]++
	}

	sb.WriteString("按级别:\n")
	sb.WriteString(fmt.Sprintf("  L1参数调整: %d次\n", levelCounts[L1_ParameterTuning]))
	sb.WriteString(fmt.Sprintf("  L2策略替换: %d次\n", levelCounts[L2_StrategyReplacement]))
	sb.WriteString(fmt.Sprintf("  L3结构改变: %d次\n", levelCounts[L3_StructuralChange]))
	sb.WriteString(fmt.Sprintf("  L4代码生成: %d次\n", levelCounts[L4_CodeGeneration]))

	verified := 0
	totalGain := 0.0
	for _, r := range sm.Records {
		if r.Verified {
			verified++
			totalGain += r.ActualGain
		}
	}
	sb.WriteString(fmt.Sprintf("\n已验证: %d次\n", verified))
	sb.WriteString(fmt.Sprintf("累计收益: %.3f\n", totalGain))

	return sb.String()
}

// AnalyzeAndModify 分析并自动修改
func (sm *SelfModifier) AnalyzeAndModify(geneID string) []*ModificationRecord {
	// 分析性能
	_ = sm.AnalyzePerformance(geneID)

	// 获取建议
	suggestions := sm.SuggestModifications(geneID)

	// 自动应用低风险修改
	var applied []*ModificationRecord
	for _, suggestion := range suggestions {
		if suggestion.Level == L1_ParameterTuning && suggestion.Performance > 0.5 {
			if sm.ApplyModification(suggestion) {
				applied = append(applied, suggestion)
			}
		}
	}

	return applied
}

// GetTopPerformingGenes 获取最佳性能基因
func (sm *SelfModifier) GetTopPerformingGenes(n int) []string {
	geneScores := make(map[string]float64)
	for _, e := range sm.PerformanceLog {
		score := 0.0
		if e.Success {
			score = 1.0
		}
		score += e.DeltaG * 0.1
		geneScores[e.GeneID] += score
	}

	type geneScore struct {
		id    string
		score float64
	}
	var sorted []geneScore
	for id, score := range geneScores {
		sorted = append(sorted, geneScore{id, score})
	}

	// 排序
	for i := 0; i < len(sorted); i++ {
		for j := i + 1; j < len(sorted); j++ {
			if sorted[j].score > sorted[i].score {
				sorted[i], sorted[j] = sorted[j], sorted[i]
			}
		}
	}

	var top []string
	for i := 0; i < min(n, len(sorted)); i++ {
		top = append(top, sorted[i].id)
	}
	return top
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// ============ API接口 ============

type ModifyRequest struct {
	GeneID string `json:"gene_id"`
	Action string `json:"action"` // analyze/suggest/apply/verify
}

type LogRequest struct {
	GeneID string  `json:"gene_id"`
	Success bool   `json:"success"`
	DeltaG float64 `json:"delta_g"`
	Query  string  `json:"query"`
}

var selfModifier *SelfModifier

func mainModifierServer() {
	selfModifier = NewSelfModifier("")

	mux := http.NewServeMux()
	mux.HandleFunc("/modify/analyze", analyzeHandler)
	mux.HandleFunc("/modify/suggest", suggestHandler)
	mux.HandleFunc("/modify/apply", applyHandler)
	mux.HandleFunc("/modify/log", logHandler)
	mux.HandleFunc("/modify/summary", summaryHandler)
	mux.HandleFunc("/health", modifierHealthHandler)

	fmt.Println("[自我修改器] 服务启动在 :8095")
	fmt.Println("  /modify/analyze - 分析基因性能")
	fmt.Println("  /modify/suggest - 建议修改")
	fmt.Println("  /modify/apply  - 应用修改")
	fmt.Println("  /modify/log    - 记录性能")
	fmt.Println("  /modify/summary - 修改摘要")
	http.ListenAndServe(":8095", mux)
}

func analyzeHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	var req ModifyRequest
	json.NewDecoder(r.Body).Decode(&req)

	perf := selfModifier.AnalyzePerformance(req.GeneID)
	json.NewEncoder(w).Encode(perf)
}

func suggestHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	var req ModifyRequest
	json.NewDecoder(r.Body).Decode(&req)

	suggestions := selfModifier.SuggestModifications(req.GeneID)
	json.NewEncoder(w).Encode(suggestions)
}

func applyHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	var req ModifyRequest
	json.NewDecoder(r.Body).Decode(&req)

	suggestions := selfModifier.SuggestModifications(req.GeneID)
	if len(suggestions) > 0 {
		selfModifier.ApplyModification(suggestions[0])
		json.NewEncoder(w).Encode(suggestions[0])
	} else {
		json.NewEncoder(w).Encode(map[string]string{"status": "no_modification_needed"})
	}
}

func logHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	var req LogRequest
	json.NewDecoder(r.Body).Decode(&req)

	selfModifier.LogPerformance(req.GeneID, req.Success, req.DeltaG, req.Query)
	json.NewEncoder(w).Encode(map[string]string{"status": "logged"})
}

func summaryHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	summary := selfModifier.GetModificationSummary()
	json.NewEncoder(w).Encode(map[string]string{"summary": summary})
}

func modifierHealthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status": "ok",
		"service": "self_modifier",
		"modifications": len(selfModifier.Records),
		"performance_entries": len(selfModifier.PerformanceLog),
	})
}

func main() {
	mainModifierServer()
}
