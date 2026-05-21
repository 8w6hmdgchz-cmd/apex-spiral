// reasoning_chain.go — 多步推理链
// AGI核心能力：能进行多步推理、自我追问、验证假设

package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"time"
)

// ReasoningStep 推理步骤
type ReasoningStep struct {
	StepNum    int       `json:"step_num"`
	Question   string    `json:"question"`    // 当前问题/假设
	Answer     string    `json:"answer"`      // 推理出的答案
	Confidence float64   `json:"confidence"`  // 置信度 0-1
	Reasoning  string    `json:"reasoning"`   // 推理过程
	Evidence   []string  `json:"evidence"`    // 支持证据
	Gaps       []string  `json:"gaps"`       // 知识缺口
	NextStep   string    `json:"next_step"`  // 下一步建议
	Timestamp  time.Time `json:"timestamp"`
}

// ReasoningChain 推理链
type ReasoningChain struct {
	ID           string          `json:"id"`
	Goal         string          `json:"goal"`          // 最终目标
	InitialQuery string          `json:"initial_query"` // 初始问题
	Steps        []*ReasoningStep `json:"steps"`
	CurrentStep  int             `json:"current_step"`
	Conclusion   string          `json:"conclusion"`    // 最终结论
	Confidence   float64         `json:"confidence"`    // 最终置信度
	Verified     bool            `json:"verified"`      // 是否验证
	CreatedAt    time.Time       `json:"created_at"`
	UpdatedAt    time.Time       `json:"updated_at"`
}

// ReasoningEngine 推理引擎
type ReasoningEngine struct {
	Chains   map[string]*ReasoningChain
	MaxSteps int
}

// NewReasoningEngine 创建推理引擎
func NewReasoningEngine() *ReasoningEngine {
	return &ReasoningEngine{
		Chains:   make(map[string]*ReasoningChain),
		MaxSteps: 10,
	}
}

// CreateChain 创建推理链
func (re *ReasoningEngine) CreateChain(query string) *ReasoningChain {
	id := fmt.Sprintf("chain_%d", time.Now().UnixNano())
	chain := &ReasoningChain{
		ID:           id,
		Goal:         query,
		InitialQuery: query,
		Steps:        make([]*ReasoningStep, 0),
		CurrentStep:  0,
		CreatedAt:    time.Now(),
		UpdatedAt:    time.Now(),
	}
	re.Chains[id] = chain
	return chain
}

// AddStep 添加推理步骤
func (rc *ReasoningChain) AddStep(question, answer, reasoning string, confidence float64) *ReasoningStep {
	step := &ReasoningStep{
		StepNum:    len(rc.Steps) + 1,
		Question:   question,
		Answer:     answer,
		Confidence: confidence,
		Reasoning:  reasoning,
		Evidence:   make([]string, 0),
		Gaps:       make([]string, 0),
		NextStep:   "",
		Timestamp:  time.Now(),
	}
	rc.Steps = append(rc.Steps, step)
	rc.CurrentStep = step.StepNum
	rc.UpdatedAt = time.Now()
	return step
}

// AddEvidence 添加证据
func (rs *ReasoningStep) AddEvidence(evidence string) {
	rs.Evidence = append(rs.Evidence, evidence)
}

// AddGap 添加知识缺口
func (rs *ReasoningStep) AddGap(gap string) {
	rs.Gaps = append(rs.Gaps, gap)
}

// SetNextStep 设置下一步
func (rs *ReasoningStep) SetNextStep(next string) {
	rs.NextStep = next
}

// Complete 完成推理链
func (rc *ReasoningChain) Complete(conclusion string, confidence float64) {
	rc.Conclusion = conclusion
	rc.Confidence = confidence
	rc.UpdatedAt = time.Now()
}

// VerifyChain 验证推理链
func (rc *ReasoningChain) VerifyChain() (bool, []string) {
	var issues []string

	// 检查步骤数
	if len(rc.Steps) == 0 {
		issues = append(issues, "推理链为空")
		return false, issues
	}

	// 检查置信度
	lowConfSteps := 0
	for _, step := range rc.Steps {
		if step.Confidence < 0.6 {
			lowConfSteps++
			issues = append(issues, fmt.Sprintf("步骤%d置信度低: %.2f", step.StepNum, step.Confidence))
		}
	}

	// 检查知识缺口
	for _, step := range rc.Steps {
		if len(step.Gaps) > 2 {
			issues = append(issues, fmt.Sprintf("步骤%d有%d个知识缺口", step.StepNum, len(step.Gaps)))
		}
	}

	// 检查结论
	if rc.Conclusion == "" {
		issues = append(issues, "推理链未得出结论")
	}

	if lowConfSteps > len(rc.Steps)/2 {
		issues = append(issues, "超过50%步骤置信度低")
	}

	return len(issues) == 0, issues
}

// GenerateSubQuestions 从问题生成子问题
func (re *ReasoningEngine) GenerateSubQuestions(question string) []string {
	var subQuestions []string
	qLower := strings.ToLower(question)

	// 是什么/什么是
	if strings.Contains(qLower, "什么") {
		subQuestions = append(subQuestions, "这个概念的定义是什么？")
		subQuestions = append(subQuestions, "它有哪些主要特征？")
		subQuestions = append(subQuestions, "它与其他相关概念有什么区别？")
	}

	// 为什么
	if strings.Contains(qLower, "为什么") {
		subQuestions = append(subQuestions, "导致这个结果的根本原因是什么？")
		subQuestions = append(subQuestions, "有哪些可能的解释？")
		subQuestions = append(subQuestions, "有什么证据支持这个解释？")
	}

	// 如何/怎么
	if strings.Contains(qLower, "如何") || strings.Contains(qLower, "怎么") {
		subQuestions = append(subQuestions, "这个过程的第一步是什么？")
		subQuestions = append(subQuestions, "有哪些关键步骤？")
		subQuestions = append(subQuestions, "可能遇到什么困难？")
	}

	// 是否
	if strings.Contains(qLower, "是否") || strings.Contains(qLower, "能不能") {
		subQuestions = append(subQuestions, "支持这个观点的证据有哪些？")
		subQuestions = append(subQuestions, "反对这个观点的证据有哪些？")
		subQuestions = append(subQuestions, "有没有反例？")
	}

	// 默认：深入分析
	if len(subQuestions) == 0 {
		subQuestions = append(subQuestions, "这个问题的核心是什么？")
		subQuestions = append(subQuestions, "有哪些已知信息？")
		subQuestions = append(subQuestions, "需要什么额外信息？")
	}

	return subQuestions
}

// SelfQuestion 自我追问
func (re *ReasoningEngine) SelfQuestion(chain *ReasoningChain) string {
	if len(chain.Steps) == 0 {
		return "这个问题的定义是什么？"
	}

	lastStep := chain.Steps[len(chain.Steps)-1]

	// 根据置信度追问
	if lastStep.Confidence < 0.7 {
		if len(lastStep.Gaps) > 0 {
			return fmt.Sprintf("关于'%s'，%s这个缺口如何填补？", lastStep.Question, lastStep.Gaps[0])
		}
		return fmt.Sprintf("'%s'这个答案的依据是什么？", lastStep.Answer)
	}

	// 根据问题类型追问
	if strings.Contains(strings.ToLower(lastStep.Question), "为什么") {
		return "这个因果关系是否唯一？有没有其他可能的解释？"
	}

	if strings.Contains(strings.ToLower(lastStep.Question), "如何") {
		return "这个方法在其他场景也适用吗？"
	}

	// 默认：验证结论
	return "这个结论在极端情况下还成立吗？"
}

// BuildChain 构建完整推理链
func (re *ReasoningEngine) BuildChain(query string, maxDepth int) *ReasoningChain {
	chain := re.CreateChain(query)

	currentQuery := query
	for i := 0; i < maxDepth && i < re.MaxSteps; i++ {
		// 生成子问题
		subQuestions := re.GenerateSubQuestions(currentQuery)
		if len(subQuestions) == 0 {
			break
		}

		// 选择第一个子问题进行推理
		subQ := subQuestions[0]

		// 模拟推理过程（实际应该调用LLM）
		answer := re.SimulateReasoning(subQ)
		confidence := re.AssessConfidence(answer)

		step := chain.AddStep(subQ, answer, "基于逻辑推理", confidence)

		// 识别知识缺口
		if confidence < 0.8 {
			step.AddGap("需要更多信息来验证")
		}

		// 自我追问
		nextQ := re.SelfQuestion(chain)
		step.SetNextStep(nextQ)

		// 检查是否达到目标
		if re.IsGoalReached(chain) {
			break
		}

		currentQuery = nextQ
	}

	// 生成结论
	chain.Complete(re.GenerateConclusion(chain), re.CalculateOverallConfidence(chain))

	return chain
}

// SimulateReasoning 模拟推理
func (re *ReasoningEngine) SimulateReasoning(question string) string {
	qLower := strings.ToLower(question)

	if strings.Contains(qLower, "定义") || strings.Contains(qLower, "是什么") {
		return "根据分析，这是关于" + question + "的概念"
	}
	if strings.Contains(qLower, "为什么") {
		return "可能的原因是" + question + "背后的逻辑"
	}
	if strings.Contains(qLower, "如何") {
		return "可以通过以下步骤实现" + question
	}

	return "基于当前信息得出初步结论"
}

// AssessConfidence 评估置信度
func (re *ReasoningEngine) AssessConfidence(answer string) float64 {
	base := 0.6

	// 有具体细节增加置信度
	if len(answer) > 50 {
		base += 0.1
	}
	if strings.Contains(answer, "因为") || strings.Contains(answer, "所以") {
		base += 0.1
	}
	if strings.Contains(answer, "根据") || strings.Contains(answer, "证据") {
		base += 0.1
	}

	// 限制范围
	if base > 0.95 {
		base = 0.95
	}

	return base
}

// IsGoalReached 判断目标是否达成
func (re *ReasoningEngine) IsGoalReached(chain *ReasoningChain) bool {
	if len(chain.Steps) < 2 {
		return false
	}

	// 置信度足够高
	lastConf := chain.Steps[len(chain.Steps)-1].Confidence
	if lastConf > 0.85 {
		return true
	}

	// 知识缺口已填补
	gapCount := 0
	for _, step := range chain.Steps {
		gapCount += len(step.Gaps)
	}
	if gapCount == 0 && len(chain.Steps) >= 3 {
		return true
	}

	return false
}

// GenerateConclusion 生成结论
func (re *ReasoningEngine) GenerateConclusion(chain *ReasoningChain) string {
	if len(chain.Steps) == 0 {
		return "无法得出结论"
	}

	var sb strings.Builder
	sb.WriteString("结论：")

	// 综合各步骤答案
	var answers []string
	for _, step := range chain.Steps {
		if step.Answer != "" {
			answers = append(answers, step.Answer)
		}
	}

	if len(answers) > 0 {
		sb.WriteString(strings.Join(answers, "；"))
	} else {
		sb.WriteString("基于推理链分析得出")
	}

	return sb.String()
}

// CalculateOverallConfidence 计算整体置信度
func (re *ReasoningEngine) CalculateOverallConfidence(chain *ReasoningChain) float64 {
	if len(chain.Steps) == 0 {
		return 0
	}

	total := 0.0
	for _, step := range chain.Steps {
		total += step.Confidence
	}

	avg := total / float64(len(chain.Steps))

	// 考虑步骤数调整
	depthPenalty := float64(len(chain.Steps)) * 0.02
	if depthPenalty > 0.2 {
		depthPenalty = 0.2
	}

	// 考虑知识缺口调整
	gapPenalty := 0.0
	for _, step := range chain.Steps {
		gapPenalty += float64(len(step.Gaps)) * 0.05
	}
	if gapPenalty > 0.3 {
		gapPenalty = 0.3
	}

	return avg - depthPenalty - gapPenalty
}

// GetChain 获取推理链
func (re *ReasoningEngine) GetChain(id string) *ReasoningChain {
	if chain, ok := re.Chains[id]; ok {
		return chain
	}
	return nil
}

// GetChainSummary 获取推理链摘要
func (rc *ReasoningChain) GetSummary() string {
	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("=== 推理链 %s ===\n", rc.ID))
	sb.WriteString(fmt.Sprintf("目标: %s\n", rc.Goal))
	sb.WriteString(fmt.Sprintf("步骤数: %d\n", len(rc.Steps)))
	sb.WriteString(fmt.Sprintf("最终置信度: %.2f\n", rc.Confidence))
	sb.WriteString(fmt.Sprintf("结论: %s\n", rc.Conclusion))

	if rc.Verified {
		sb.WriteString("状态: 已验证 ✓\n")
	} else {
		sb.WriteString("状态: 未验证\n")
	}

	return sb.String()
}

// ============ API ============

var reasoningEngine *ReasoningEngine

func init() {
	reasoningEngine = NewReasoningEngine()
}

type BuildRequest struct {
	Query    string `json:"query"`
	MaxDepth int    `json:"max_depth"`
}

type AddStepRequest struct {
	ChainID   string `json:"chain_id"`
	Question  string `json:"question"`
	Answer    string `json:"answer"`
	Reasoning string `json:"reasoning"`
	Confidence float64 `json:"confidence"`
}

type VerifyRequest struct {
	ChainID string `json:"chain_id"`
}

func buildChainHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req BuildRequest
	json.NewDecoder(r.Body).Decode(&req)

	if req.MaxDepth == 0 {
		req.MaxDepth = 5
	}

	chain := reasoningEngine.BuildChain(req.Query, req.MaxDepth)
	json.NewEncoder(w).Encode(chain)
}

func getChainHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	id := r.URL.Query().Get("id")
	chain := reasoningEngine.GetChain(id)
	if chain == nil {
		json.NewEncoder(w).Encode(map[string]string{"error": "chain not found"})
		return
	}
	json.NewEncoder(w).Encode(chain)
}

func addStepHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req AddStepRequest
	json.NewDecoder(r.Body).Decode(&req)

	chain := reasoningEngine.GetChain(req.ChainID)
	if chain == nil {
		json.NewEncoder(w).Encode(map[string]string{"error": "chain not found"})
		return
	}

	step := chain.AddStep(req.Question, req.Answer, req.Reasoning, req.Confidence)
	json.NewEncoder(w).Encode(step)
}

func completeChainHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req struct {
		ChainID    string  `json:"chain_id"`
		Conclusion string  `json:"conclusion"`
		Confidence float64 `json:"confidence"`
	}
	json.NewDecoder(r.Body).Decode(&req)

	chain := reasoningEngine.GetChain(req.ChainID)
	if chain == nil {
		json.NewEncoder(w).Encode(map[string]string{"error": "chain not found"})
		return
	}

	chain.Complete(req.Conclusion, req.Confidence)
	json.NewEncoder(w).Encode(chain)
}

func verifyChainHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req VerifyRequest
	json.NewDecoder(r.Body).Decode(&req)

	chain := reasoningEngine.GetChain(req.ChainID)
	if chain == nil {
		json.NewEncoder(w).Encode(map[string]string{"error": "chain not found"})
		return
	}

	valid, issues := chain.VerifyChain()
	chain.Verified = valid

	json.NewEncoder(w).Encode(map[string]interface{}{
		"chain_id": req.ChainID,
		"valid": valid,
		"issues": issues,
	})
}

func selfQuestionHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req struct {
		ChainID string `json:"chain_id"`
	}
	json.NewDecoder(r.Body).Decode(&req)

	chain := reasoningEngine.GetChain(req.ChainID)
	if chain == nil {
		json.NewEncoder(w).Encode(map[string]string{"error": "chain not found"})
		return
	}

	question := reasoningEngine.SelfQuestion(chain)
	json.NewEncoder(w).Encode(map[string]string{"next_question": question})
}

func reasoningHealthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status": "ok",
		"service": "reasoning_chain",
		"active_chains": len(reasoningEngine.Chains),
		"max_steps": reasoningEngine.MaxSteps,
	})
}

func mainReasoningServer() {
	reasoningEngine = NewReasoningEngine()

	mux := http.NewServeMux()
	mux.HandleFunc("/build", buildChainHandler)
	mux.HandleFunc("/get", getChainHandler)
	mux.HandleFunc("/add_step", addStepHandler)
	mux.HandleFunc("/complete", completeChainHandler)
	mux.HandleFunc("/verify", verifyChainHandler)
	mux.HandleFunc("/self_question", selfQuestionHandler)
	mux.HandleFunc("/health", reasoningHealthHandler)

	fmt.Println("[推理链引擎] 服务启动在 :8097")
	fmt.Println("  /build        - 构建推理链")
	fmt.Println("  /get          - 获取推理链")
	fmt.Println("  /add_step     - 添加推理步骤")
	fmt.Println("  /complete     - 完成推理链")
	fmt.Println("  /verify       - 验证推理链")
	fmt.Println("  /self_question - 自我追问")
	http.ListenAndServe(":8097", mux)
}

func main() {
	mainReasoningServer()
}
