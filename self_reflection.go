// self_reflection.go — 自我反思机制
// AGI核心能力之一：能知道自己在做什么、为什么、有什么缺陷
//
// 自我反思层次:
//   L1. 元认知监控 — 知道自己"知道什么"
//   L2. 过程反思 — 知道"怎么做的"
//   L3. 策略反思 — 知道"为什么这么做"
//   L4. 缺陷识别 — 知道"哪里做错了"
//   L5. 自我改进 — 知道"应该怎么改"

package main

import (
	"encoding/json"
	"fmt"
	"math"
	"net/http"
	"strings"
	"time"
)

// SelfReflectionLevel 自我反思等级
type SelfReflectionLevel int

const (
	L1_MetaCognition SelfReflectionLevel = iota + 1 // 元认知监控
	L2_ProcessReflection                             // 过程反思
	L3_StrategyReflection                           // 策略反思
	L4_DefectIdentification                         // 缺陷识别
	L5_SelfImprovement                              // 自我改进
)

// ReflectionRecord 反思记录
type ReflectionRecord struct {
	Timestamp   time.Time `json:"timestamp"`
	Level       SelfReflectionLevel `json:"level"`
	Question    string `json:"question"`      // 反思问题
	Answer      string `json:"answer"`        // 反思答案
	Confidence  float64 `json:"confidence"`  // 答案置信度
	Insight    string `json:"insight"`       // 洞察/领悟
	ActionItems []string `json:"action_items"` // 改进行动项
}

// SelfModel 自我模型 — 存储对自己的认知
type SelfModel struct {
	Capabilities    map[string]float64 `json:"capabilities"`    // 能力清单 + 置信度
	Limitations    []string `json:"limitations"`              // 已知缺陷
	Patterns       map[string]string `json:"patterns"`        // 行为模式
	StrengthAreas  []string `json:"strength_areas"`          // 强项
	WeakAreas      []string `json:"weak_areas"`              // 弱项
	LearningQueue  []string `json:"learning_queue"`          // 待学习项
}

// SelfReflectionEngine 自我反思引擎
type SelfReflectionEngine struct {
	Model       *SelfModel
	History     []*ReflectionRecord
	MaxHistory  int
}

// NewSelfReflectionEngine 创建自我反思引擎
func NewSelfReflectionEngine() *SelfReflectionEngine {
	return &SelfReflectionEngine{
		Model: &SelfModel{
			Capabilities: map[string]float64{
				"pattern_recognition": 0.9,
				"logical_reasoning": 0.85,
				"code_generation": 0.88,
				"abstract_reasoning": 0.82,
				"self_reflection": 0.75,
				"strategy_adaptation": 0.78,
				"memory_retrieval": 0.80,
				"error_detection": 0.85,
				"multi_step_planning": 0.70,
				"creative_thinking": 0.72,
			},
			Limitations: []string{
				"容易陷入局部最优",
				"长程依赖把握不足",
				"元认知精度有待提升",
				"某些领域知识不足",
			},
			Patterns: map[string]string{},
			StrengthAreas: []string{
				"代码生成和调试",
				"模式识别",
				"多步骤推理",
			},
			WeakAreas: []string{
				"创意类任务",
				"长期规划",
				"自我反思精度",
			},
			LearningQueue: []string{},
		},
		History: make([]*ReflectionRecord, 0),
		MaxHistory: 1000,
	}
}

// reflectOnTask 对任务进行自我反思
func (sre *SelfReflectionEngine) reflectOnTask(task string, result string, success bool) *ReflectionRecord {
	record := &ReflectionRecord{
		Timestamp: time.Now(),
		Level: L1_MetaCognition,
	}

	// L1: 元认知监控 — 我知道什么？
	record.Question = fmt.Sprintf("任务'%s'的结果是%v，我的认知边界在哪里？", task, success)
	record.Answer = sre.reflectOnKnowledge(task, result)

	// L2: 过程反思 — 我是怎么做的？
	if len(sre.History) > 0 {
		sre.reflectOnProcess(record, task, result)
	}

	// L3: 策略反思 — 为什么这么做？
	sre.reflectOnStrategy(record, task, result, success)

	// L4: 缺陷识别 — 哪里做错了？
	if !success {
		sre.identifyDefect(record, task, result)
	}

	// L5: 自我改进建议
	sre.generateImprovement(record, task, result, success)

	// 置信度评估
	record.Confidence = sre.assessConfidence(record)

	sre.addRecord(record)

	return record
}

// reflectOnKnowledge 元认知监控
func (sre *SelfReflectionEngine) reflectOnKnowledge(task, result string) string {
	taskLower := strings.ToLower(task)

	// 检查能力匹配
	var relevantSkills []string
	for skill, conf := range sre.Model.Capabilities {
		if conf > 0.8 {
			relevantSkills = append(relevantSkills, fmt.Sprintf("%s(%.0f%%)", skill, conf*100))
		}
	}

	// 检查是否涉及弱项
	var warnings []string
	for _, weak := range sre.Model.WeakAreas {
		if strings.Contains(taskLower, weak) {
			warnings = append(warnings, fmt.Sprintf("警告: 涉及弱项'%s'", weak))
		}
	}

	answer := fmt.Sprintf("任务涉及%v领域，我有较高置信度的技能:%v", task, relevantSkills)
	if len(warnings) > 0 {
		answer += fmt.Sprintf("，但需要注意:%v", warnings)
	}

	return answer
}

// reflectOnProcess 过程反思
func (sre *SelfReflectionEngine) reflectOnProcess(record *ReflectionRecord, task, result string) {
	record.Level = L2_ProcessReflection
	record.Question = fmt.Sprintf("任务'%s'的处理过程是否最优？", task)

	// 分析历史模式
	var recentPatterns []string
	for i := len(sre.History) - 1; i >= 0 && i >= len(sre.History)-5; i-- {
		if sre.History[i].Level >= L2_ProcessReflection {
			recentPatterns = append(recentPatterns, sre.History[i].Insight)
		}
	}

	processInsight := "处理过程符合常规模式"
	if len(recentPatterns) > 3 {
		processInsight = fmt.Sprintf("检测到重复模式:%v", recentPatterns[:3])
	}

	record.Insight = processInsight
}

// reflectOnStrategy 策略反思
func (sre *SelfReflectionEngine) reflectOnStrategy(record *ReflectionRecord, task, result string, success bool) {
	record.Level = L3_StrategyReflection
	record.Question = fmt.Sprintf("为什么选择当前策略处理'%s'？", task)

	taskLower := strings.ToLower(task)

	// 选择策略的理由
	var strategyReason string
	if strings.Contains(taskLower, "code") || strings.Contains(taskLower, "编程") {
		strategyReason = "使用了代码生成策略，因为任务涉及编程"
	} else if strings.Contains(taskLower, "why") || strings.Contains(taskLower, "为什么") {
		strategyReason = "使用了因果推理策略，因为任务是问为什么"
	} else if strings.Contains(taskLower, "how") || strings.Contains(taskLower, "如何") {
		strategyReason = "使用了过程分解策略，因为任务是问如何做"
	} else {
		strategyReason = "使用了通用推理策略"
	}

	if success {
		strategyReason += "，策略有效"
	} else {
		strategyReason += "，但策略可能不是最优"
	}

	record.Insight = strategyReason
}

// identifyDefect 缺陷识别
func (sre *SelfReflectionEngine) identifyDefect(record *ReflectionRecord, task, result string) {
	record.Level = L4_DefectIdentification
	record.Question = fmt.Sprintf("任务'%s'失败的根本原因是什么？", task)

	// 分析可能的缺陷原因
	var defects []string

	// 检查是否涉及弱项
	taskLower := strings.ToLower(task)
	for _, weak := range sre.Model.WeakAreas {
		if strings.Contains(taskLower, weak) {
			defects = append(defects, fmt.Sprintf("能力不足: %s", weak))
		}
	}

	// 检查是否有类似失败模式
	failurePattern := sre.detectFailurePattern(task)
	if failurePattern != "" {
		defects = append(defects, fmt.Sprintf("重复失败: %s", failurePattern))
	}

	// 检查知识缺口
	if len(result) > 0 && strings.Contains(result, "我不知道") {
		defects = append(defects, "知识缺口: 训练数据覆盖不足")
	}

	if len(defects) == 0 {
		defects = append(defects, "未知原因: 需要进一步分析")
	}

	record.Insight = fmt.Sprintf("缺陷分析:%v", defects)

	// 更新自我模型
	for _, defect := range defects {
		if !contains(sre.Model.Limitations, defect) {
			sre.Model.Limitations = append(sre.Model.Limitations, defect)
		}
	}
}

// detectFailurePattern 检测失败模式
func (sre *SelfReflectionEngine) detectFailurePattern(task string) string {
	var similarTasks []string
	for _, rec := range sre.History {
		if !strings.Contains(rec.Answer, "成功") && rec.Level >= L3_StrategyReflection {
			if hasOverlap(rec.Question, task) {
				similarTasks = append(similarTasks, rec.Question)
			}
		}
	}

	if len(similarTasks) > 2 {
		return fmt.Sprintf("类似任务'%s'失败%d次", task, len(similarTasks))
	}
	return ""
}

// generateImprovement 生成改进建议
func (sre *SelfReflectionEngine) generateImprovement(record *ReflectionRecord, task, result string, success bool) {
	record.Level = L5_SelfImprovement
	record.Question = fmt.Sprintf("任务'%s'如何改进？", task)

	var actions []string

	if !success {
		actions = append(actions, "分析失败原因，调整策略")
	}

	// 根据任务类型建议
	taskLower := strings.ToLower(task)
	if strings.Contains(taskLower, "code") || strings.Contains(taskLower, "编程") {
		actions = append(actions, "加强代码模式学习")
		actions = append(actions, "添加更多代码示例到基因池")
	}
	if strings.Contains(taskLower, "why") || strings.Contains(taskLower, "为什么") {
		actions = append(actions, "增加因果推理链深度")
	}
	if len(result) > 1000 {
		actions = append(actions, "考虑分步处理，避免输出过长")
	}

	// 根据反思级别建议
	if record.Confidence < 0.7 {
		actions = append(actions, "提高元认知精度: 添加更多自我检查点")
		actions = append(actions, "考虑寻求外部验证")
	}

	record.ActionItems = actions
	record.Insight = fmt.Sprintf("改进建议:%v", actions)
}

// assessConfidence 评估反思置信度
func (sre *SelfReflectionEngine) assessConfidence(record *ReflectionRecord) float64 {
	conf := 0.5 // 基础置信度

	// 历史记录越多，置信度越高
	if len(sre.History) > 10 {
		conf += 0.1
	}
	if len(sre.History) > 50 {
		conf += 0.1
	}

	// 有具体洞察
	if len(record.Insight) > 20 {
		conf += 0.1
	}

	// 有具体行动项
	if len(record.ActionItems) > 0 {
		conf += 0.1
	}

	return math.Min(0.95, conf)
}

// addRecord 添加反思记录
func (sre *SelfReflectionEngine) addRecord(record *ReflectionRecord) {
	sre.History = append(sre.History, record)
	if len(sre.History) > sre.MaxHistory {
		sre.History = sre.History[1:]
	}
}

// GetSelfAssessment 获取自我评估
func (sre *SelfReflectionEngine) GetSelfAssessment() string {
	var sb strings.Builder

	sb.WriteString("=== 自我评估报告 ===\n\n")

	sb.WriteString("【能力分布】\n")
	for skill, conf := range sre.Model.Capabilities {
		bar := strings.Repeat("█", int(conf*10)) + strings.Repeat("░", 10-int(conf*10))
		sb.WriteString(fmt.Sprintf("  %-25s [%s] %.0f%%\n", skill, bar, conf*100))
	}

	sb.WriteString("\n【已知缺陷】\n")
	for i, lim := range sre.Model.Limitations {
		sb.WriteString(fmt.Sprintf("  %d. %s\n", i+1, lim))
	}

	sb.WriteString("\n【强项领域】\n")
	for _, area := range sre.Model.StrengthAreas {
		sb.WriteString(fmt.Sprintf("  ✓ %s\n", area))
	}

	sb.WriteString("\n【弱项领域】\n")
	for _, area := range sre.Model.WeakAreas {
		sb.WriteString(fmt.Sprintf("  ✗ %s\n", area))
	}

	sb.WriteString("\n【待学习】\n")
	if len(sre.Model.LearningQueue) == 0 {
		sb.WriteString("  (无)\n")
	} else {
		for _, item := range sre.Model.LearningQueue {
			sb.WriteString(fmt.Sprintf("  → %s\n", item))
		}
	}

	sb.WriteString(fmt.Sprintf("\n【反思历史】%d条\n", len(sre.History)))

	return sb.String()
}

// GetRecentInsights 获取最近的洞察
func (sre *SelfReflectionEngine) GetRecentInsights(n int) []string {
	if n > len(sre.History) {
		n = len(sre.History)
	}
	insights := make([]string, 0, n)
	for i := len(sre.History) - n; i < len(sre.History); i++ {
		if sre.History[i].Insight != "" {
			insights = append(insights, fmt.Sprintf("[L%d] %s",
				sre.History[i].Level, sre.History[i].Insight))
		}
	}
	return insights
}

// AskSelf 自我追问 — 主动反思
func (sre *SelfReflectionEngine) AskSelf(question string) *ReflectionRecord {
	record := &ReflectionRecord{
		Timestamp: time.Now(),
		Level: L1_MetaCognition,
		Question: question,
	}

	// 分析问题类型
	questionLower := strings.ToLower(question)

	if strings.Contains(questionLower, "我") && strings.Contains(questionLower, "知") {
		record.Answer = sre.answerWhatIKnow(question)
		record.Level = L1_MetaCognition
	} else if strings.Contains(questionLower, "我") && (strings.Contains(questionLower, "做") || strings.Contains(questionLower, "处理")) {
		record.Answer = sre.answerHowIDid(question)
		record.Level = L2_ProcessReflection
	} else if strings.Contains(questionLower, "为什么") {
		record.Answer = sre.answerWhy(question)
		record.Level = L3_StrategyReflection
	} else if strings.Contains(questionLower, "错误") || strings.Contains(questionLower, "失败") {
		record.Answer = sre.answerMyMistakes(question)
		record.Level = L4_DefectIdentification
	} else if strings.Contains(questionLower, "改") || strings.Contains(questionLower, "进步") {
		record.Answer = sre.answerHowToImprove(question)
		record.Level = L5_SelfImprovement
	} else {
		record.Answer = sre.answerWhatIKnow(question)
		record.Level = L1_MetaCognition
	}

	record.Confidence = sre.assessConfidence(record)
	sre.addRecord(record)

	return record
}

// 回答"我知道什么"
func (sre *SelfReflectionEngine) answerWhatIKnow(question string) string {
	var knows []string
	for skill, conf := range sre.Model.Capabilities {
		if conf > 0.7 {
			knows = append(knows, fmt.Sprintf("%s(%.0f%%)", skill, conf*100))
		}
	}
	return fmt.Sprintf("我擅长:%v", knows)
}

// 回答"我是怎么做的"
func (sre *SelfReflectionEngine) answerHowIDid(question string) string {
	if len(sre.History) == 0 {
		return "无历史记录，无法分析"
	}

	var processes []string
	count := 0
	for i := len(sre.History) - 1; i >= 0 && count < 3; i-- {
		if sre.History[i].Level >= L2_ProcessReflection {
			processes = append(processes, sre.History[i].Insight)
			count++
		}
	}
	return fmt.Sprintf("近期处理模式:%v", processes)
}

// 回答"为什么"
func (sre *SelfReflectionEngine) answerWhy(question string) string {
	// 分析选择的原因
	var reasons []string

	// 检查能力匹配
	for skill, conf := range sre.Model.Capabilities {
		if conf > 0.8 {
			reasons = append(reasons, fmt.Sprintf("我有较高置信度的%s技能", skill))
		}
	}

	// 检查限制
	for _, lim := range sre.Model.Limitations {
		if strings.Contains(question, lim) {
			reasons = append(reasons, fmt.Sprintf("但受限于:%s", lim))
		}
	}

	if len(reasons) == 0 {
		return "基于通用推理策略"
	}
	return fmt.Sprintf("原因:%v", reasons)
}

// 回答"我的错误"
func (sre *SelfReflectionEngine) answerMyMistakes(question string) string {
	var mistakes []string
	for _, rec := range sre.History[len(sre.History)-min(10, len(sre.History)):] {
		if rec.Level >= L4_DefectIdentification {
			mistakes = append(mistakes, rec.Insight)
		}
	}
	if len(mistakes) == 0 {
		return "未检测到明显错误模式"
	}
	return fmt.Sprintf("近期缺陷:%v", mistakes)
}

// 回答"如何改进"
func (sre *SelfReflectionEngine) answerHowToImprove(question string) string {
	var improvements []string

	// 根据弱项建议
	for _, weak := range sre.Model.WeakAreas {
		improvements = append(improvements, fmt.Sprintf("加强%s能力", weak))
	}

	// 根据反思历史建议
	var lowConfActions []string
	for _, rec := range sre.History {
		if rec.Confidence < 0.7 && len(rec.ActionItems) > 0 {
			lowConfActions = append(lowConfActions, rec.ActionItems...)
		}
	}
	if len(lowConfActions) > 0 {
		improvements = append(improvements, fmt.Sprintf("提高反思精度:%v", lowConfActions[:3]))
	}

	if len(improvements) == 0 {
		return "继续保持当前状态"
	}
	return fmt.Sprintf("改进建议:%v", improvements)
}

// 辅助函数
func contains(slice []string, item string) bool {
	for _, s := range slice {
		if s == item {
			return true
		}
	}
	return false
}

func hasOverlap(a, b string) bool {
	aWords := strings.Fields(strings.ToLower(a))
	bWords := strings.Fields(strings.ToLower(b))
	for _, wa := range aWords {
		for _, wb := range bWords {
			if wa == wb && len(wa) > 3 {
				return true
			}
		}
	}
	return false
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// ============ API接口 ============

type ReflectRequest struct {
	Task    string `json:"task"`
	Result  string `json:"result"`
	Success bool   `json:"success"`
}

type AskRequest struct {
	Question string `json:"question"`
}

var selfReflectionEngine *SelfReflectionEngine

func init() {
	selfReflectionEngine = NewSelfReflectionEngine()
}

// selfReflectHandler 自我反思API
func selfReflectHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req ReflectRequest
	json.NewDecoder(r.Body).Decode(&req)

	record := selfReflectionEngine.reflectOnTask(req.Task, req.Result, req.Success)
	json.NewEncoder(w).Encode(record)
}

// askSelfHandler 自我追问API
func askSelfHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req AskRequest
	json.NewDecoder(r.Body).Decode(&req)

	record := selfReflectionEngine.AskSelf(req.Question)
	json.NewEncoder(w).Encode(record)
}

// selfAssessHandler 自我评估API
func selfAssessHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	assessment := selfReflectionEngine.GetSelfAssessment()
	json.NewEncoder(w).Encode(map[string]string{
		"assessment": assessment,
	})
}

// selfInsightsHandler 洞察API
func selfInsightsHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	insights := selfReflectionEngine.GetRecentInsights(10)
	json.NewEncoder(w).Encode(map[string]interface{}{
		"insights": insights,
		"count": len(insights),
	})
}

// selfHealthHandler 健康检查
func selfHealthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status": "ok",
		"service": "self_reflection",
		"reflection_count": len(selfReflectionEngine.History),
		"levels": []string{"L1_MetaCognition", "L2_ProcessReflection", "L3_StrategyReflection", "L4_DefectIdentification", "L5_SelfImprovement"},
	})
}

var selfReflectionMux *http.ServeMux

func mainSelfReflectionServer() {
	selfReflectionEngine = NewSelfReflectionEngine()

	selfReflectionMux = http.NewServeMux()
	selfReflectionMux.HandleFunc("/reflect", selfReflectHandler)
	selfReflectionMux.HandleFunc("/ask", askSelfHandler)
	selfReflectionMux.HandleFunc("/assess", selfAssessHandler)
	selfReflectionMux.HandleFunc("/insights", selfInsightsHandler)
	selfReflectionMux.HandleFunc("/health", selfHealthHandler)

	fmt.Println("[自我反思引擎] 服务启动在 :8094")
	fmt.Println("  /reflect - 反思任务")
	fmt.Println("  /ask     - 自我追问")
	fmt.Println("  /assess  - 自我评估")
	fmt.Println("  /insights - 最近洞察")
	http.ListenAndServe(":8094", selfReflectionMux)
}

func main() {
	mainSelfReflectionServer()
}
