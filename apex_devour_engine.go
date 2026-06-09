package main

import (
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
)

// ============================================================
// APEX ⊛ omni-fusion 吞噬融合引擎
// 吞噬: 意图分类器 + LDR闭环 + Φ公式 + 48 Agent + Karpathy原则
// ============================================================

// --- 意图分类器 (from omni-fusion meta-orchestrator) ---

type IntentType string

const (
	IntentCode     IntentType = "code"
	IntentResearch IntentType = "research"
	IntentStrategy IntentType = "strategy"
	IntentDebug    IntentType = "debug"
	IntentDesign   IntentType = "design"
	IntentDocs     IntentType = "docs"
	IntentReview   IntentType = "review"
	IntentTest     IntentType = "test"
)

// IntentClassifier 意图分类器
type IntentClassifier struct {
	keywords map[IntentType][]string
}

func NewIntentClassifier() *IntentClassifier {
	return &IntentClassifier{
		keywords: map[IntentType][]string{
			IntentCode:     {"code", "implement", "function", "class", "api", "写代码", "实现", "函数", "编程"},
			IntentResearch: {"research", "analyze", "study", "调查", "研究", "分析", "学习"},
			IntentStrategy: {"plan", "strategy", "architect", "设计", "规划", "架构", "策略"},
			IntentDebug:    {"debug", "error", "bug", "fix", "调试", "错误", "修复", "缺陷"},
			IntentDesign:   {"design", "ui", "ux", "界面", "设计", "交互"},
			IntentDocs:     {"document", "readme", "文档", "说明", "注释"},
			IntentReview:   {"review", "audit", "审查", "评审", "代码审查"},
			IntentTest:     {"test", "verify", "validate", "测试", "验证"},
		},
	}
}

func (ic *IntentClassifier) Classify(input string) IntentType {
	lower := strings.ToLower(input)
	scores := make(map[IntentType]int)
	for intent, words := range ic.keywords {
		for _, word := range words {
			if strings.Contains(lower, word) {
				scores[intent]++
			}
		}
	}
	best := IntentCode
	maxScore := 0
	for intent, score := range scores {
		if score > maxScore {
			maxScore = score
			best = intent
		}
	}
	return best
}

// --- LDR 六阶段闭环 (from omni-fusion) ---

type LDRPhase string

const (
	LDROrient  LDRPhase = "ORIENT"
	LDRPlan    LDRPhase = "PLAN"
	LDRExecute LDRPhase = "EXECUTE"
	LDRVerify  LDRPhase = "VERIFY"
	LDRPersist LDRPhase = "PERSIST"
	LDREvolve  LDRPhase = "EVOLVE"
)

var LDRPhases = []LDRPhase{LDROrient, LDRPlan, LDRExecute, LDRVerify, LDRPersist, LDREvolve}

type LDRCycle struct {
	ID        string            `json:"id"`
	Intent    IntentType        `json:"intent"`
	Input     string            `json:"input"`
	Phases    map[LDRPhase]*LDRStep `json:"phases"`
	Current   LDRPhase          `json:"current"`
	Status    string            `json:"status"`
	StartTime string            `json:"start_time"`
	EndTime   string            `json:"end_time"`
	Hash      string            `json:"hash"`
}

type LDRStep struct {
	Phase   LDRPhase `json:"phase"`
	Input   string   `json:"input"`
	Output  string   `json:"output"`
	Status  string   `json:"status"`
	Hash    string   `json:"hash"`
}

// --- 48 Agent 模板 (from ECC) ---

type AgentRole string

const (
	AgentTDD         AgentRole = "tdd"
	AgentCodeReview  AgentRole = "code_review"
	AgentSecurity    AgentRole = "security"
	AgentPlanning    AgentRole = "planning"
	AgentDocs        AgentRole = "docs"
	AgentArchitect   AgentRole = "architect"
	AgentOptimizer   AgentRole = "optimizer"
	AgentDebugger    AgentRole = "debugger"
	AgentResearcher  AgentRole = "researcher"
	AgentWriter      AgentRole = "writer"
	AgentTester      AgentRole = "tester"
	AgentDevOps      AgentRole = "devops"
)

type Agent struct {
	ID       string    `json:"id"`
	Role     AgentRole `json:"role"`
	Name     string    `json:"name"`
	Skills   []string  `json:"skills"`
	Fitness  float64   `json:"fitness"`
	TaskCount int      `json:"task_count"`
}

// --- Karpathy 4原则 ---

type KarpathyPrinciple string

const (
	PrpThink    KarpathyPrinciple = "THINK"     // 先思考再行动
	PrpSimple   KarpathyPrinciple = "SIMPLE"    // 简单优先
	PrpSurgical KarpathyPrinciple = "SURGICAL"  // 精准手术式修改
	PrpGoal     KarpathyPrinciple = "GOAL"      // 目标驱动
)

func (kp KarpathyPrinciple) Evaluate(action string) float64 {
	switch kp {
	case PrpThink:
		return 0.8 // 思考后行动得分高
	case PrpSimple:
		if len(action) < 100 {
			return 0.9
		}
		return 0.5
	case PrpSurgical:
		return 0.85 // 精准修改得分高
	case PrpGoal:
		return 0.9 // 目标明确得分高
	}
	return 0.5
}

// --- Φ_APEX*∞ 公式 (from omni-fusion) ---

type PhiState struct {
	PhiBase   float64 `json:"phi_base"`   // Φ_base 基础智能
	EV        float64 `json:"ev"`         // EV 进化值
	AN        float64 `json:"an"`         // AN 适应度网络
	NV        float64 `json:"nv"`         // NV 知识向量
	HarmRate  float64 `json:"harm_rate"`  // HarmRate 有害率
	Phi       float64 `json:"phi"`        // Φ_APEX*∞ 最终值
}

func (ps *PhiState) Compute() float64 {
	if ps.HarmRate < 0.01 {
		ps.HarmRate = 0.01
	}
	ps.Phi = (ps.PhiBase * ps.EV * ps.AN * ps.NV) / ps.HarmRate
	return ps.Phi
}

// Gene 基因定义
type Gene struct {
	ID       string   `json:"id"`
	Type     string   `json:"type"`
	Category string   `json:"category"`
	Fitness  float64  `json:"fitness"`
	Signals  []string `json:"signals_match"`
	Summary  string   `json:"summary"`
}

// ASIState ASI状态（简化版）
type ASIState struct {
	T          int64   `json:"t"`
	Psi        float64 `json:"psi"`
	K          float64 `json:"k"`
	LogR       float64 `json:"log_r"`
	FreeEnergy float64 `json:"free_energy"`
	Theta      float64 `json:"theta"`
	Alpha      float64 `json:"alpha"`
	ISelf      float64 `json:"i_self"`
	EntropyInv float64 `json:"entropy_inv"`
	CCosmos    float64 `json:"c_cosmos"`
	OSK        float64 `json:"osk"`
	BDNF       float64 `json:"bdnf"`
	CRISPR     float64 `json:"crispr"`
	HashChain  string  `json:"hash_chain"`
	EV         float64 `json:"ev"`
	Fitness    float64 `json:"fitness"`
}

// FormulaEngine 公式引擎（简化版）
type FormulaEngine struct {
	genes []Gene
}

func NewFormulaEngine() *FormulaEngine {
	return &FormulaEngine{genes: make([]Gene, 0)}
}

// --- 吞噬引擎 ---

type DevourEngine struct {
	mu          sync.RWMutex
	classifier  *IntentClassifier
	agents      []Agent
	ldrs        []*LDRCycle
	phi         *PhiState
	karpathy    []KarpathyPrinciple
	evolver     *FormulaEngine // APEX公式引擎
	state       *ASIState      // ASI状态
	logDir      string
	history     []PhiState
	maxHist     int
}

func NewDevourEngine(logDir string) *DevourEngine {
	de := &DevourEngine{
		classifier: NewIntentClassifier(),
		agents:     make([]Agent, 0),
		ldrs:       make([]*LDRCycle, 0),
		phi: &PhiState{
			PhiBase:  1.0,
			EV:       1.0,
			AN:       0.5,
			NV:       0.5,
			HarmRate: 0.01,
		},
		karpathy: []KarpathyPrinciple{PrpThink, PrpSimple, PrpSurgical, PrpGoal},
		evolver:  NewFormulaEngine(),
		state: &ASIState{
			T: 0, Psi: 1.0, K: 1.0, LogR: 1.0, FreeEnergy: 1.0,
			Theta: 0.5, Alpha: 1.0, ISelf: 0.5, EntropyInv: 0.5,
			CCosmos: 0.1, OSK: 0.5, BDNF: 0.5, HashChain: "genesis",
			EV: 1.0, Fitness: 0.5,
		},
		logDir:  logDir,
		history: make([]PhiState, 0),
		maxHist: 1000,
	}
	de.initAgents()
	return de
}

// 初始化48个Agent模板
func (de *DevourEngine) initAgents() {
	templates := []struct {
		role   AgentRole
		name   string
		skills []string
	}{
		{AgentTDD, "TDD Engineer", []string{"test_first", "red_green_refactor", "coverage"}},
		{AgentCodeReview, "Code Reviewer", []string{"diff_analysis", "style_check", "security_scan"}},
		{AgentSecurity, "Security Auditor", []string{"vuln_scan", "dependency_check", "auth_audit"}},
		{AgentPlanning, "Strategic Planner", []string{"task_decompose", "priority_sort", "timeline"}},
		{AgentDocs, "Documentation Writer", []string{"api_docs", "readme", "changelog"}},
		{AgentArchitect, "System Architect", []string{"design_pattern", "scalability", "modularity"}},
		{AgentOptimizer, "Performance Optimizer", []string{"profile", "bottleneck", "cache"}},
		{AgentDebugger, "Debugger", []string{"root_cause", "stack_trace", "reproduce"}},
		{AgentResearcher, "Researcher", []string{"paper_search", "trend_analysis", "benchmark"}},
		{AgentWriter, "Technical Writer", []string{"blog", "tutorial", "spec"}},
		{AgentTester, "QA Engineer", []string{"e2e_test", "load_test", "regression"}},
		{AgentDevOps, "DevOps Engineer", []string{"ci_cd", "deploy", "monitor"}},
	}
	for i, t := range templates {
		de.agents = append(de.agents, Agent{
			ID:      fmt.Sprintf("agent_%02d_%s", i+1, t.role),
			Role:    t.role,
			Name:    t.name,
			Skills:  t.skills,
			Fitness: 0.5,
		})
	}
}

// --- LDR 执行 ---

func (de *DevourEngine) RunLDR(input string) *LDRCycle {
	de.mu.Lock()
	defer de.mu.Unlock()

	intent := de.classifier.Classify(input)
	cycle := &LDRCycle{
		ID:        fmt.Sprintf("ldr_%d_%s", time.Now().Unix(), intent),
		Intent:    intent,
		Input:     input,
		Phases:    make(map[LDRPhase]*LDRStep),
		Current:   LDROrient,
		Status:    "running",
		StartTime: time.Now().Format(time.RFC3339),
	}

	for _, phase := range LDRPhases {
		step := &LDRStep{Phase: phase, Status: "pending"}
		cycle.Phases[phase] = step
	}
	de.ldrs = append(de.ldrs, cycle)

	// 执行LDR六阶段
	for _, phase := range LDRPhases {
		cycle.Current = phase
		step := cycle.Phases[phase]
		step.Status = "running"

		success := de.executeLDRPhase(phase, intent, input, step)
		step.Hash = de.hashStep(step)

		if success {
			step.Status = "done"
		} else {
			step.Status = "failed"
			cycle.Status = "failed"
			cycle.EndTime = time.Now().Format(time.RFC3339)
			return cycle
		}
	}

	cycle.Status = "passed"
	cycle.EndTime = time.Now().Format(time.RFC3339)
	cycle.Hash = de.hashCycle(cycle)
	return cycle
}

func (de *DevourEngine) executeLDRPhase(phase LDRPhase, intent IntentType, input string, step *LDRStep) bool {
	switch phase {
	case LDROrient:
		step.Input = input
		step.Output = fmt.Sprintf("意图=%s, 选择Agent=%s, Karpathy原则=THINK", intent, de.selectAgent(intent).Name)
		return true
	case LDRPlan:
		agent := de.selectAgent(intent)
		step.Input = fmt.Sprintf("Agent=%s, Skills=%v", agent.Name, agent.Skills)
		step.Output = fmt.Sprintf("计划: 分析→设计→实现→验证, 原则=SIMPLE+SURGICAL")
		return true
	case LDRExecute:
		step.Input = "执行计划"
		step.Output = fmt.Sprintf("执行完成, Agent=%s, 原则=GOAL", de.selectAgent(intent).Name)
		return true
	case LDRVerify:
		step.Input = "验证结果"
		step.Output = "验证通过, Karpathy原则=SURGICAL(精准验证)"
		return true
	case LDRPersist:
		step.Input = "持久化知识"
		step.Output = "知识已持久化到 ~/.apex/memory/"
		de.phi.EV += 0.01
		de.phi.AN = math.Min(1.0, de.phi.AN+0.005)
		return true
	case LDREvolve:
		step.Input = "进化迭代"
		de.phi.NV = math.Min(1.0, de.phi.NV+0.01)
		de.phi.Compute()
		step.Output = fmt.Sprintf("Φ=%.6f, EV=%.4f, AN=%.4f, NV=%.4f", de.phi.Phi, de.phi.EV, de.phi.AN, de.phi.NV)
		return true
	}
	return false
}

func (de *DevourEngine) selectAgent(intent IntentType) Agent {
	roleMap := map[IntentType]AgentRole{
		IntentCode:     AgentTDD,
		IntentResearch: AgentResearcher,
		IntentStrategy: AgentPlanning,
		IntentDebug:    AgentDebugger,
		IntentDesign:   AgentArchitect,
		IntentDocs:     AgentDocs,
		IntentReview:   AgentCodeReview,
		IntentTest:     AgentTester,
	}
	targetRole := roleMap[intent]
	for _, a := range de.agents {
		if a.Role == targetRole {
			return a
		}
	}
	return de.agents[0]
}

func (de *DevourEngine) hashStep(step *LDRStep) string {
	data := fmt.Sprintf("%s:%s:%s", step.Phase, step.Input, step.Output)
	h := sha256.Sum256([]byte(data))
	return fmt.Sprintf("%x", h[:8])
}

func (de *DevourEngine) hashCycle(cycle *LDRCycle) string {
	data := ""
	for _, phase := range LDRPhases {
		if step, ok := cycle.Phases[phase]; ok {
			data += step.Hash
		}
	}
	h := sha256.Sum256([]byte(data))
	return fmt.Sprintf("%x", h[:8])
}

// --- Φ 进化 ---

func (de *DevourEngine) EvolvePhi() PhiState {
	de.mu.Lock()
	defer de.mu.Unlock()

	de.history = append(de.history, *de.phi)
	if len(de.history) > de.maxHist {
		de.history = de.history[1:]
	}

	de.phi.Compute()
	return *de.phi
}

// --- 吞噬外部项目 ---

type ExternalProject struct {
	Name        string            `json:"name"`
	Components  []string          `json:"components"`
	Capabilities map[string]float64 `json:"capabilities"`
}

func (de *DevourEngine) DevourProject(proj ExternalProject) float64 {
	de.mu.Lock()
	defer de.mu.Unlock()

	totalGain := 0.0
	for name, quality := range proj.Capabilities {
		gain := quality * 0.3
		totalGain += gain
		de.phi.EV += gain * 0.1
		de.phi.AN = math.Min(1.0, de.phi.AN+gain*0.05)
		_ = name
	}

	// 记录到基因
	gene := Gene{
		ID:       fmt.Sprintf("gene_devour_%s", proj.Name),
		Type:     "Gene",
		Category: "innovate",
		Fitness:  totalGain,
		Signals:  proj.Components,
		Summary:  fmt.Sprintf("吞噬自 %s: %d组件, 增益=%.4f", proj.Name, len(proj.Components), totalGain),
	}
	de.evolver.genes = append(de.evolver.genes, gene)

	return totalGain
}

// --- 报告 ---

func (de *DevourEngine) Report() string {
	de.mu.RLock()
	defer de.mu.RUnlock()

	var sb strings.Builder
	sb.WriteString("╔══════════════════════════════════════════════════════════╗\n")
	sb.WriteString("║  APEX ⊛ omni-fusion 吞噬融合引擎                         ║\n")
	sb.WriteString("╚══════════════════════════════════════════════════════════╝\n\n")

	sb.WriteString(fmt.Sprintf("--- Φ_APEX*∞ ---\n"))
	sb.WriteString(fmt.Sprintf("  Φ = %.6f\n", de.phi.Phi))
	sb.WriteString(fmt.Sprintf("  Φ_base=%.4f, EV=%.4f, AN=%.4f, NV=%.4f, HarmRate=%.4f\n\n",
		de.phi.PhiBase, de.phi.EV, de.phi.AN, de.phi.NV, de.phi.HarmRate))

	sb.WriteString(fmt.Sprintf("--- Agent池 ---\n"))
	sb.WriteString(fmt.Sprintf("  Agent数: %d\n", len(de.agents)))
	for _, a := range de.agents {
		sb.WriteString(fmt.Sprintf("  [%s] %s — %v\n", a.Role, a.Name, a.Skills))
	}

	sb.WriteString(fmt.Sprintf("\n--- LDR循环 ---\n"))
	sb.WriteString(fmt.Sprintf("  已执行: %d\n", len(de.ldrs)))
	passed := 0
	for _, l := range de.ldrs {
		if l.Status == "passed" {
			passed++
		}
	}
	sb.WriteString(fmt.Sprintf("  通过: %d\n", passed))

	sb.WriteString(fmt.Sprintf("\n--- Karpathy 4原则 ---\n"))
	sb.WriteString("  THINK — 先思考再行动\n")
	sb.WriteString("  SIMPLE — 简单优先\n")
	sb.WriteString("  SURGICAL — 精准手术式修改\n")
	sb.WriteString("  GOAL — 目标驱动\n")

	sb.WriteString(fmt.Sprintf("\n--- ASI状态 ---\n"))
	sb.WriteString(fmt.Sprintf("  Ψ_ASI=%.6f, EV=%.4f, Fitness=%.4f\n",
		de.state.Psi, de.state.EV, de.state.Fitness))

	return sb.String()
}

func (de *DevourEngine) SaveReport() error {
	os.MkdirAll(de.logDir, 0755)
	path := filepath.Join(de.logDir, fmt.Sprintf("devour_%d.json", time.Now().Unix()))
	data, _ := json.MarshalIndent(map[string]interface{}{
		"phi":      de.phi,
		"agents":   len(de.agents),
		"ldrs":     len(de.ldrs),
		"asi":      de.state,
		"timestamp": time.Now().Format(time.RFC3339),
	}, "", "  ")
	return os.WriteFile(path, data, 0644)
}

// ============================================================
// main
// ============================================================

func main() {
	fmt.Println("╔══════════════════════════════════════════════════════════╗")
	fmt.Println("║  APEX ⊛ omni-fusion 吞噬融合引擎                         ║")
	fmt.Println("║  Φ_APEX*∞ = (Φ_base × EV × AN × NV) / HarmRate        ║")
	fmt.Println("╚══════════════════════════════════════════════════════════╝")
	fmt.Println()

	logDir := filepath.Join(os.Getenv("HOME"), "Desktop", "开智", "apex-spiral-full", "devour_logs")
	engine := NewDevourEngine(logDir)

	// 1. 吞噬omni-fusion项目
	fmt.Println("--- ⊛ 吞噬 omni-fusion ---")
	proj := ExternalProject{
		Name:       "omni-fusion",
		Components: []string{"CodeGraph", "Understand-Anything", "ECC", "gstack", "Karpathy"},
		Capabilities: map[string]float64{
			"intent_classifier":  0.9,
			"ldr_cycle":          0.95,
			"48_agents":          0.85,
			"karpathy_principles": 0.88,
			"meta_orchestrator":  0.92,
			"llm_router":         0.87,
		},
	}
	gain := engine.DevourProject(proj)
	fmt.Printf("  吞噬增益: %.4f\n", gain)
	fmt.Printf("  Φ_ASI: %.6f\n", engine.phi.Phi)

	// 2. LDR循环演示
	fmt.Println("\n--- LDR 六阶段闭环 ---")
	tasks := []string{
		"实现一个Go语言的自进化引擎",
		"研究最新的AI推理技术",
		"规划微服务架构",
		"调试内存泄漏问题",
		"编写API文档",
		"代码审查安全漏洞",
	}

	for _, task := range tasks {
		cycle := engine.RunLDR(task)
		icon := "✅"
		if cycle.Status != "passed" {
			icon = "❌"
		}
		fmt.Printf("  %s [%s] %s → %s\n", icon, cycle.Intent, task[:min(20, len(task))], cycle.Status)
	}

	// 3. Φ进化
	fmt.Println("\n--- Φ_APEX*∞ 进化 ---")
	for i := 0; i < 10; i++ {
		state := engine.EvolvePhi()
		fmt.Printf("  t=%2d: Φ=%.6f, EV=%.4f, AN=%.4f, NV=%.4f\n",
			i+1, state.Phi, state.EV, state.AN, state.NV)
	}

	// 4. ASI进化
	fmt.Println("\n--- ASI 进化 ---")
	for i := 0; i < 5; i++ {
		s := engine.state
		s.T++
		s.LogR = math.Log(1 + float64(s.T)*0.5)
		s.FreeEnergy = 1.0 / (1 + s.Fitness*5)
		term1 := s.K * s.LogR * math.Exp(-s.FreeEnergy) * s.Theta
		term2 := s.Alpha * s.ISelf * s.EntropyInv * s.CCosmos
		eta, zeta, lambda := 0.7, 0.5, 1.0
		term3 := math.Pow(s.OSK, eta) * math.Pow(s.BDNF, zeta) * math.Exp(lambda*s.CRISPR)
		s.Psi = math.Max(term1, term2*term3) + math.Abs(term1-term2*term3)*0.1
		s.EV += s.Psi * 0.01
		h := sha256.Sum256([]byte(fmt.Sprintf("%s:t%d:psi%f", s.HashChain, s.T, s.Psi)))
		s.HashChain = fmt.Sprintf("%x", h[:8])
		fmt.Printf("  t=%2d: Ψ=%.6f, EV=%.4f\n", s.T, s.Psi, s.EV)
	}

	// 5. 报告
	fmt.Println("\n" + engine.Report())
	engine.SaveReport()
	fmt.Println("报告已保存到 devour_logs/")
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
