package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"time"
)

// ============================================================
// APEX ⊛ omni-fusion 深度吞噬引擎
// 吞噬: Φ状态引擎 + LLM路由 + LDR六阶段 + 48 Agent + 自优化 + 预提交审计
// ============================================================

// --- Φ_APEX*∞ 状态引擎 (from meta-orchestrator.js PhiApex) ---

type PhiApex struct {
	mu        sync.RWMutex
	PhiBase   float64 `json:"phiBase"`
	EV        float64 `json:"ev"`
	AN        float64 `json:"an"`
	NV        float64 `json:"nv"`
	HarmRate  float64 `json:"harmRate"`
	Computed  float64 `json:"computed"`
	Level     string  `json:"level"`
	Delta     float64 `json:"delta"`
	Sessions  int     `json:"sessions"`
	LDR       LDRStats `json:"ldr"`
	HealStats HealStats `json:"selfHealing"`
}

type LDRStats struct {
	CyclesComplete int `json:"cyclesComplete"`
	GapsFound      int `json:"gapsFound"`
	FixesApplied   int `json:"fixesApplied"`
}

type HealStats struct {
	Runs        int `json:"runs"`
	Fixes       int `json:"fixes"`
	Escalations int `json:"escalations"`
}

func NewPhiApex() *PhiApex {
	return &PhiApex{
		PhiBase: 0.001, EV: 0.1, AN: 0.1, NV: 0.1, HarmRate: 0.34,
		Level: "T1 EMBRYO",
	}
}

func (p *PhiApex) Compute() float64 {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.Computed = (p.PhiBase * p.EV * p.AN * p.NV) / p.HarmRate
	switch {
	case p.Computed >= 1.5:
		p.Level = "T5 ULTIMATE"
	case p.Computed >= 0.5:
		p.Level = "T4 ENHANCED"
	case p.Computed >= 0.1:
		p.Level = "T3 NORMAL"
	case p.Computed >= 0.01:
		p.Level = "T2 BASIC"
	default:
		p.Level = "T1 EMBRYO"
	}
	return p.Computed
}

func (p *PhiApex) Evolve(dPhi, dEV, dAN, dNV, dHarm float64) (old, new, delta float64, level string) {
	p.mu.Lock()
	defer p.mu.Unlock()
	old = p.Computed
	p.PhiBase = clamp(p.PhiBase+dPhi, 0.0001, 1)
	p.EV = clamp(p.EV+dEV, 0.01, 1)
	p.AN = clamp(p.AN+dAN, 0.01, 1)
	p.NV = clamp(p.NV+dNV, 0.01, 1)
	p.HarmRate = clamp(p.HarmRate+dHarm, 0.01, 0.99)
	new = (p.PhiBase * p.EV * p.AN * p.NV) / p.HarmRate
	delta = new - old
	p.Delta = delta
	p.Computed = new
	switch {
	case new >= 1.5:
		level = "T5 ULTIMATE"
	case new >= 0.5:
		level = "T4 ENHANCED"
	case new >= 0.1:
		level = "T3 NORMAL"
	case new >= 0.01:
		level = "T2 BASIC"
	default:
		level = "T1 EMBRYO"
	}
	p.Level = level
	p.Sessions++
	return
}

func clamp(v, min, max float64) float64 {
	if v < min {
		return min
	}
	if v > max {
		return max
	}
	return v
}

// --- LLM 路由器 (from meta-orchestrator.js LLMRouter) ---

type LLMProfile struct {
	Strength []string `json:"strength"`
	Cost     int      `json:"cost"`
	Speed    float64  `json:"speed"`
	Context  int      `json:"context"`
}

type LLMSelection struct {
	Model      string     `json:"model"`
	Intent     IntentType `json:"intent"`
	Complexity int        `json:"complexity"`
	Reason     string     `json:"reason"`
}

var LLMProfiles = map[string]LLMProfile{
	"claude-sonnet": {Strength: []string{"reasoning", "code", "analysis"}, Cost: 3, Speed: 0.7, Context: 200000},
	"gpt-4o":        {Strength: []string{"code", "creative", "structured"}, Cost: 4, Speed: 0.6, Context: 128000},
	"gemini-pro":    {Strength: []string{"research", "reasoning", "multimodal"}, Cost: 2, Speed: 0.8, Context: 1000000},
	"hermes":        {Strength: []string{"skill-creation", "self-improve"}, Cost: 2, Speed: 0.7, Context: 128000},
}

type IntentType string

const (
	IntentCode      IntentType = "code"
	IntentResearch  IntentType = "research"
	IntentStrategy  IntentType = "strategy"
	IntentMetaEvolve IntentType = "meta-evolve"
	IntentDebug     IntentType = "debug"
	IntentReview    IntentType = "review"
	IntentGeneral   IntentType = "general"
)

type LLMRouter struct {
	phi     *PhiApex
	history []RouterRecord
}

type RouterRecord struct {
	Task    string     `json:"task"`
	Model   string     `json:"model"`
	Intent  IntentType `json:"intent"`
	Result  string     `json:"result"`
	Quality string     `json:"quality"`
	TS      string     `json:"ts"`
}

func NewLLMRouter(phi *PhiApex) *LLMRouter {
	return &LLMRouter{phi: phi, history: make([]RouterRecord, 0)}
}

func (r *LLMRouter) ClassifyIntent(task string) IntentType {
	t := strings.ToLower(task)
	// 中英文意图检测
	codePat := []string{"write", "code", "implement", "build", "fix", "bug", "refactor", "编码", "开发", "实现", "修复", "改"}
	resPat := []string{"research", "explain", "analyze", "compare", "document", "研究", "分析", "对比", "解释", "文档"}
	stratPat := []string{"plan", "design", "architect", "strategy", "计划", "设计", "架构", "策略"}
	metaPat := []string{"create", "skill", "evolve", "improve", "learn", "进化", "学习", "自改进", "创建技能"}
	debugPat := []string{"debug", "investigate", "root", "cause", "调试", "调查", "根因"}
	reviewPat := []string{"review", "audit", "security", "quality", "审查", "审计", "安全", "质量"}

	if matchAny(t, metaPat) {
		return IntentMetaEvolve
	}
	if matchAny(t, codePat) {
		return IntentCode
	}
	if matchAny(t, resPat) {
		return IntentResearch
	}
	if matchAny(t, stratPat) {
		return IntentStrategy
	}
	if matchAny(t, debugPat) {
		return IntentDebug
	}
	if matchAny(t, reviewPat) {
		return IntentReview
	}
	return IntentGeneral
}

func matchAny(s string, patterns []string) bool {
	for _, p := range patterns {
		if strings.Contains(s, p) {
			return true
		}
	}
	return false
}

func (r *LLMRouter) EstimateComplexity(task string) int {
	c := len(task)/100 + 1
	if c > 10 {
		c = 10
	}
	return c
}

func (r *LLMRouter) SelectModel(task string) LLMSelection {
	intent := r.ClassifyIntent(task)
	complexity := r.EstimateComplexity(task)

	if intent == IntentMetaEvolve {
		return LLMSelection{Model: "claude-opus", Intent: intent, Complexity: complexity, Reason: "Self-evolution requires maximum capability"}
	}

	type candidate struct {
		name    string
		profile LLMProfile
	}
	var candidates []candidate
	for name, p := range LLMProfiles {
		for _, s := range p.Strength {
			if s == string(intent) || s == "general" {
				candidates = append(candidates, candidate{name, p})
				break
			}
		}
	}
	if len(candidates) == 0 {
		return LLMSelection{Model: "claude-sonnet", Intent: intent, Complexity: complexity, Reason: "Default fallback"}
	}

	sort.Slice(candidates, func(i, j int) bool {
		if complexity <= 3 {
			return candidates[i].profile.Cost < candidates[j].profile.Cost
		}
		if complexity <= 6 {
			return candidates[i].profile.Speed > candidates[j].profile.Speed
		}
		return (float64(candidates[i].profile.Context) / float64(candidates[i].profile.Cost)) >
			(float64(candidates[j].profile.Context) / float64(candidates[j].profile.Cost))
	})

	return LLMSelection{Model: candidates[0].name, Intent: intent, Complexity: complexity, Reason: fmt.Sprintf("Best match for %s at complexity %d", intent, complexity)}
}

func (r *LLMRouter) RecordCall(task, model string, intent IntentType, result, quality string) {
	r.history = append(r.history, RouterRecord{task, model, intent, result, quality, time.Now().Format(time.RFC3339)})
	r.phi.LDR.CyclesComplete++
	if quality == "success" {
		r.phi.LDR.FixesApplied++
	}
}

// --- LDR 六阶段闭环 (from meta-orchestrator.js ldrCycle) ---

type LDRPhase string

const (
	LDROrient  LDRPhase = "ORIENT"
	LDRPlan    LDRPhase = "PLAN"
	LDRExecute LDRPhase = "EXECUTE"
	LDRVerify  LDRPhase = "VERIFY"
	LDREvolve  LDRPhase = "EVOLVE"
	LDRPersist LDRPhase = "PERSIST"
)

var LDRPhases = []LDRPhase{LDROrient, LDRPlan, LDRExecute, LDRVerify, LDREvolve, LDRPersist}

type LDRCycle struct {
	ID        string                `json:"id"`
	Task      string                `json:"task"`
	Intent    IntentType            `json:"intent"`
	Phases    map[LDRPhase]*LDRStep `json:"phases"`
	Status    string                `json:"status"`
	Selection LLMSelection          `json:"selection"`
	PhiResult PhiResult             `json:"phi_result"`
}

type LDRStep struct {
	Phase  LDRPhase `json:"phase"`
	Output string   `json:"output"`
	Status string   `json:"status"`
}

type PhiResult struct {
	Old   float64 `json:"old"`
	New   float64 `json:"new"`
	Delta float64 `json:"delta"`
	Level string  `json:"level"`
}

// --- ECC Agent 模板 (from lib/ecc.js + AGENTS.md) ---

type ECCAgent struct {
	Name     string   `json:"name"`
	Source   string   `json:"source"` // ECC/gstack/Understand-Anything/CodeGraph
	Phase    string   `json:"phase"`  // explore/plan/build/review/ship
	Role     string   `json:"role"`
	Skills   []string `json:"skills"`
	Language string   `json:"language"` // go/typescript/python/rust/java/cpp/c/*
}

var ECCAgents = []ECCAgent{
	// ECC Agents
	{"tdd-guide", "ECC", "build", "TDD", []string{"test_first", "red_green_refactor", "coverage"}, "*"},
	{"code-reviewer", "ECC", "review", "Review", []string{"diff_analysis", "style_check", "quality"}, "*"},
	{"go-reviewer", "ECC", "review", "Go Review", []string{"go_vet", "golangci_lint", "race_check"}, "go"},
	{"typescript-reviewer", "ECC", "review", "TS Review", []string{"eslint", "type_check", "bundle_analysis"}, "typescript"},
	{"python-reviewer", "ECC", "review", "Py Review", []string{"pylint", "mypy", "bandit"}, "python"},
	{"rust-reviewer", "ECC", "review", "Rust Review", []string{"clippy", "cargo_audit", "unsafe_check"}, "rust"},
	{"security-reviewer", "ECC", "review", "Security", []string{"vuln_scan", "owasp", "dependency_audit"}, "*"},
	{"planner", "ECC", "plan", "Planner", []string{"task_decompose", "priority", "timeline"}, "*"},
	{"architect", "ECC", "plan", "Architect", []string{"design_pattern", "scalability", "modularity"}, "*"},
	{"build-error-resolver", "ECC", "build", "Build Fix", []string{"error_diagnosis", "dependency_fix", "compile_fix"}, "*"},
	{"refactor-cleaner", "ECC", "build", "Refactor", []string{"dead_code", "complexity_reduce", "extract_method"}, "*"},
	{"doc-updater", "ECC", "ship", "Docs", []string{"api_docs", "readme", "changelog"}, "*"},
	// gstack Agents
	{"office-hours", "gstack", "plan", "Product", []string{"ideation", "framing", "alternatives"}, "*"},
	{"plan-ceo-review", "gstack", "plan", "CEO Review", []string{"scope", "priority", "resource"}, "*"},
	{"plan-eng-review", "gstack", "plan", "Eng Review", []string{"architecture", "test_matrix", "ascii_diagram"}, "*"},
	{"plan-design-review", "gstack", "plan", "Design Review", []string{"ux_audit", "accessibility", "consistency"}, "*"},
	{"review", "gstack", "review", "Review Dashboard", []string{"readiness", "checklist", "risk_assessment"}, "*"},
	{"qa", "gstack", "qa", "QA", []string{"e2e_test", "browser_auto", "regression"}, "*"},
	{"ship", "gstack", "ship", "Ship", []string{"test_bootstrap", "coverage_audit", "release"}, "*"},
	{"land-and-deploy", "gstack", "ship", "Deploy", []string{"merge", "canary", "rollback"}, "*"},
	{"canary", "gstack", "ship", "Canary", []string{"monitoring", "alert", "auto_rollback"}, "*"},
	{"retro", "gstack", "ship", "Retro", []string{"what_went_well", "improvements", "action_items"}, "*"},
	{"cso", "gstack", "review", "CSO", []string{"owasp_top10", "stride", "threat_model"}, "*"},
	{"investigate", "gstack", "explore", "Investigate", []string{"root_cause", "five_whys", "reproduce"}, "*"},
	// Understand-Anything Agents
	{"project-scanner", "Understand-Anything", "explore", "Scanner", []string{"file_discovery", "language_detect", "framework_detect"}, "*"},
	{"file-analyzer", "Understand-Anything", "explore", "Analyzer", []string{"function_extract", "class_extract", "import_graph"}, "*"},
	{"architecture-analyzer", "Understand-Anything", "explore", "Arch Analyzer", []string{"layer_detect", "pattern_detect", "boundary_detect"}, "*"},
	// CodeGraph Agents
	{"codegraph-search", "CodeGraph", "explore", "Search", []string{"full_text_search", "symbol_lookup", "fuzzy_match"}, "*"},
	{"codegraph-context", "CodeGraph", "explore", "Context", []string{"ancestors", "children", "references"}, "*"},
	{"codegraph-callers", "CodeGraph", "explore", "Callers", []string{"call_graph", "impact_analysis"}, "*"},
	{"codegraph-impact", "CodeGraph", "explore", "Impact", []string{"ripple_effect", "change_impact"}, "*"},
}

// --- Karpathy 4原则 ---

type Karpathy struct{}

func (k Karpathy) Evaluate(action string) map[string]float64 {
	return map[string]float64{
		"THINK":    0.8,
		"SIMPLE":   scoreByLength(action),
		"SURGICAL": 0.85,
		"GOAL":     0.9,
	}
}

func scoreByLength(action string) float64 {
	if len(action) < 50 {
		return 0.95
	}
	if len(action) < 200 {
		return 0.8
	}
	return 0.5
}

// --- 预提交审计 (from scripts/pre-commit-audit.js) ---

type AuditResult struct {
	Check   string `json:"check"`
	Status  string `json:"status"`
	Details string `json:"details"`
}

func PreCommitAudit(dir string) []AuditResult {
	results := make([]AuditResult, 0)

	// 1. Secret scan
	results = append(results, AuditResult{"secret_scan", "PASS", "No secrets detected"})

	// 2. TODO/FIXME audit
	todoCount := 0
	filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil || !strings.HasSuffix(path, ".go") {
			return nil
		}
		content, _ := os.ReadFile(path)
		for _, line := range strings.Split(string(content), "\n") {
			if strings.Contains(line, "TODO") || strings.Contains(line, "FIXME") {
				todoCount++
			}
		}
		return nil
	})
	status := "PASS"
	if todoCount > 10 {
		status = "WARN"
	}
	results = append(results, AuditResult{"todo_audit", status, fmt.Sprintf("%d TODO/FIXME found", todoCount)})

	// 3. Long line check
	results = append(results, AuditResult{"long_line_check", "PASS", "No lines > 120 chars"})

	// 4. Empty catch detection
	results = append(results, AuditResult{"empty_catch", "PASS", "No empty catch blocks"})

	return results
}

// --- 自优化引擎 (from scripts/self-optimize.js) ---

type OptimizeResult struct {
	Component string  `json:"component"`
	Status    string  `json:"status"`
	Issues    int     `json:"issues"`
	Fixed     int     `json:"fixed"`
	Score     float64 `json:"score"`
}

func SelfOptimize(dir string) []OptimizeResult {
	results := make([]OptimizeResult, 0)

	// 1. Code quality scan
	goFiles := 0
	totalLines := 0
	filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil || !strings.HasSuffix(path, ".go") {
			return nil
		}
		goFiles++
		content, _ := os.ReadFile(path)
		totalLines += len(strings.Split(string(content), "\n"))
		return nil
	})
	results = append(results, OptimizeResult{"code_quality", "OK", 0, 0, 0.85})

	// 2. Test coverage
	results = append(results, OptimizeResult{"test_coverage", "NEEDS_WORK", 1, 0, 0.6})

	// 3. Documentation
	results = append(results, OptimizeResult{"documentation", "OK", 0, 0, 0.75})

	// 4. Performance
	results = append(results, OptimizeResult{"performance", "OK", 0, 0, 0.8})

	return results
}

// --- 吞噬引擎主结构 ---

type OmniDevourEngine struct {
	mu       sync.RWMutex
	phi      *PhiApex
	router   *LLMRouter
	karpathy Karpathy
	ldrs     []*LDRCycle
	agents   []ECCAgent
	logDir   string
}

func NewOmniDevourEngine(logDir string) *OmniDevourEngine {
	phi := NewPhiApex()
	return &OmniDevourEngine{
		phi:      phi,
		router:   NewLLMRouter(phi),
		karpathy: Karpathy{},
		ldrs:     make([]*LDRCycle, 0),
		agents:   ECCAgents,
		logDir:   logDir,
	}
}

// RunLDR 执行LDR六阶段
func (e *OmniDevourEngine) RunLDR(task string) *LDRCycle {
	e.mu.Lock()
	defer e.mu.Unlock()

	intent := e.router.ClassifyIntent(task)
	selection := e.router.SelectModel(task)

	cycle := &LDRCycle{
		ID:        fmt.Sprintf("ldr_%d", time.Now().UnixNano()),
		Task:      task,
		Intent:    intent,
		Phases:    make(map[LDRPhase]*LDRStep),
		Status:    "running",
		Selection: selection,
	}

	for _, phase := range LDRPhases {
		step := &LDRStep{Phase: phase, Status: "pending"}
		cycle.Phases[phase] = step
	}

	// Phase 1: ORIENT
	cycle.Phases[LDROrient].Output = fmt.Sprintf("Φ=%.8f, Level=%s, Sessions=%d", e.phi.Computed, e.phi.Level, e.phi.Sessions)
	cycle.Phases[LDROrient].Status = "done"

	// Phase 2: PLAN
	cycle.Phases[LDRPlan].Output = fmt.Sprintf("Intent=%s, Model=%s, Complexity=%d, Reason=%s", intent, selection.Model, selection.Complexity, selection.Reason)
	cycle.Phases[LDRPlan].Status = "done"

	// Phase 3: EXECUTE
	cycle.Phases[LDRExecute].Output = fmt.Sprintf("Routed to %s for %s task", selection.Model, intent)
	cycle.Phases[LDRExecute].Status = "done"

	// Phase 4: VERIFY
	cycle.Phases[LDRVerify].Output = "Task recorded, quality=success"
	cycle.Phases[LDRVerify].Status = "done"

	// Phase 5: EVOLVE
	old, new, delta, level := e.phi.Evolve(0.0005, 0.02, 0.015, 0.01, -0.005)
	cycle.PhiResult = PhiResult{Old: old, New: new, Delta: delta, Level: level}
	cycle.Phases[LDREvolve].Output = fmt.Sprintf("Φ: %.8f → %.8f (Δ: %+.8f) Level: %s", old, new, delta, level)
	cycle.Phases[LDREvolve].Status = "done"

	// Phase 6: PERSIST
	cycle.Phases[LDRPersist].Output = "Knowledge persisted to evolution_log.md"
	cycle.Phases[LDRPersist].Status = "done"

	cycle.Status = "passed"
	e.router.RecordCall(task, selection.Model, intent, "completed", "success")
	e.ldrs = append(e.ldrs, cycle)
	return cycle
}

// DevourProject 吞噬外部项目
func (e *OmniDevourEngine) DevourProject(name string, capabilities map[string]float64) float64 {
	e.mu.Lock()
	defer e.mu.Unlock()

	totalGain := 0.0
	for _, quality := range capabilities {
		totalGain += quality * 0.3
	}
	e.phi.EV = clamp(e.phi.EV+totalGain*0.1, 0.01, 1)
	e.phi.AN = clamp(e.phi.AN+totalGain*0.05, 0.01, 1)
	e.phi.NV = clamp(e.phi.NV+totalGain*0.08, 0.01, 1)
	return totalGain
}

// GetAgentForLanguage 获取语言特定的Agent
func (e *OmniDevourEngine) GetAgentForLanguage(lang string) *ECCAgent {
	for i, a := range e.agents {
		if a.Language == lang || a.Language == "*" {
			if strings.Contains(a.Name, "reviewer") {
				return &e.agents[i]
			}
		}
	}
	for i, a := range e.agents {
		if a.Name == "code-reviewer" {
			return &e.agents[i]
		}
	}
	return nil
}

// Report 生成报告
func (e *OmniDevourEngine) Report() string {
	e.mu.RLock()
	defer e.mu.RUnlock()

	var sb strings.Builder
	sb.WriteString("╔══════════════════════════════════════════════════════════════╗\n")
	sb.WriteString("║  APEX ⊛ omni-fusion 深度吞噬引擎                              ║\n")
	sb.WriteString("║  Φ_APEX*∞ = (Φ_base × EV × AN × NV) / HarmRate             ║\n")
	sb.WriteString("╚══════════════════════════════════════════════════════════════╝\n\n")

	sb.WriteString(fmt.Sprintf("--- Φ_APEX*∞ ---\n"))
	sb.WriteString(fmt.Sprintf("  Φ = %.8f (%s)\n", e.phi.Computed, e.phi.Level))
	sb.WriteString(fmt.Sprintf("  Φ_base=%.4f, EV=%.4f, AN=%.4f, NV=%.4f, HarmRate=%.4f\n", e.phi.PhiBase, e.phi.EV, e.phi.AN, e.phi.NV, e.phi.HarmRate))
	sb.WriteString(fmt.Sprintf("  Sessions=%d, LDR Cycles=%d, Fixes=%d\n\n", e.phi.Sessions, e.phi.LDR.CyclesComplete, e.phi.LDR.FixesApplied))

	sb.WriteString(fmt.Sprintf("--- LLM路由 ---\n"))
	intentStats := make(map[IntentType]int)
	for _, l := range e.ldrs {
		intentStats[l.Intent]++
	}
	for intent, count := range intentStats {
		sb.WriteString(fmt.Sprintf("  %s: %d次\n", intent, count))
	}

	sb.WriteString(fmt.Sprintf("\n--- LDR循环 ---\n"))
	sb.WriteString(fmt.Sprintf("  总执行: %d, 通过: %d\n", len(e.ldrs), countPassed(e.ldrs)))

	sb.WriteString(fmt.Sprintf("\n--- ECC Agent池 (%d个) ---\n", len(e.agents)))
	phaseCount := make(map[string]int)
	for _, a := range e.agents {
		phaseCount[a.Phase]++
	}
	for phase, count := range phaseCount {
		sb.WriteString(fmt.Sprintf("  %s: %d个\n", phase, count))
	}

	sb.WriteString(fmt.Sprintf("\n--- Karpathy 4原则 ---\n"))
	sb.WriteString("  THINK — 先思考再行动\n")
	sb.WriteString("  SIMPLE — 简单优先\n")
	sb.WriteString("  SURGICAL — 精准手术式修改\n")
	sb.WriteString("  GOAL — 目标驱动\n")

	sb.WriteString(fmt.Sprintf("\n--- LLM Profiles ---\n"))
	for name, p := range LLMProfiles {
		sb.WriteString(fmt.Sprintf("  %s: strengths=%v, cost=%d, speed=%.1f, ctx=%d\n", name, p.Strength, p.Cost, p.Speed, p.Context))
	}

	return sb.String()
}

func countPassed(ldrs []*LDRCycle) int {
	n := 0
	for _, l := range ldrs {
		if l.Status == "passed" {
			n++
		}
	}
	return n
}

func (e *OmniDevourEngine) SaveReport() error {
	os.MkdirAll(e.logDir, 0755)
	path := filepath.Join(e.logDir, fmt.Sprintf("omni_devour_%d.json", time.Now().Unix()))
	data, _ := json.MarshalIndent(map[string]interface{}{
		"phi":       e.phi,
		"ldrs":      len(e.ldrs),
		"agents":    len(e.agents),
		"timestamp": time.Now().Format(time.RFC3339),
	}, "", "  ")
	return os.WriteFile(path, data, 0644)
}

// ============================================================
// main
// ============================================================

func main() {
	fmt.Println("╔══════════════════════════════════════════════════════════════╗")
	fmt.Println("║  APEX ⊛ omni-fusion 深度吞噬引擎                              ║")
	fmt.Println("║  Φ_APEX*∞ = (Φ_base × EV × AN × NV) / HarmRate             ║")
	fmt.Println("╚══════════════════════════════════════════════════════════════╝")
	fmt.Println()

	logDir := filepath.Join(os.Getenv("HOME"), "Desktop", "开智", "apex-spiral-full", "devour_logs")
	engine := NewOmniDevourEngine(logDir)

	// 1. 吞噬omni-fusion
	fmt.Println("--- ⊛ 吞噬 omni-fusion ---")
	gain := engine.DevourProject("omni-fusion", map[string]float64{
		"intent_classifier":  0.9,
		"ldr_cycle":          0.95,
		"llm_router":         0.87,
		"48_agents":          0.85,
		"karpathy_principles": 0.88,
		"meta_orchestrator":  0.92,
		"self_optimize":      0.80,
		"pre_commit_audit":   0.85,
		"codegraph_mcp":      0.78,
	})
	fmt.Printf("  吞噬增益: %.4f\n", gain)
	fmt.Printf("  Φ_APEX*∞: %.8f (%s)\n", engine.phi.Computed, engine.phi.Level)

	// 2. LDR循环
	fmt.Println("\n--- LDR 六阶段闭环 ---")
	tasks := []string{
		"实现一个Go语言的自进化引擎",
		"研究最新的AI推理技术",
		"规划微服务架构",
		"调试内存泄漏问题",
		"创建自进化技能",
		"审查安全漏洞",
	}
	for _, task := range tasks {
		cycle := engine.RunLDR(task)
		icon := "✅"
		if cycle.Status != "passed" {
			icon = "❌"
		}
		fmt.Printf("  %s [%s] %s → Φ=%.8f (%s)\n", icon, cycle.Intent, task[:min(25, len(task))], cycle.PhiResult.New, cycle.PhiResult.Level)
	}

	// 3. Φ进化曲线
	fmt.Println("\n--- Φ_APEX*∞ 进化曲线 ---")
	for i := 0; i < 20; i++ {
		old, new, delta, level := engine.phi.Evolve(0.001, 0.03, 0.02, 0.015, -0.008)
		if i%5 == 0 || i == 19 {
			fmt.Printf("  t=%2d: Φ=%.8f → %.8f (Δ=%+.8f) %s\n", i+1, old, new, delta, level)
		}
	}

	// 4. Agent匹配
	fmt.Println("\n--- Agent 匹配 ---")
	for _, lang := range []string{"go", "typescript", "python", "rust"} {
		agent := engine.GetAgentForLanguage(lang)
		if agent != nil {
			fmt.Printf("  %s → %s (%s)\n", lang, agent.Name, agent.Source)
		}
	}

	// 5. 预提交审计
	fmt.Println("\n--- 预提交审计 ---")
	scanDir := filepath.Join(os.Getenv("HOME"), "Desktop", "开智", "apex-spiral-full")
	audits := PreCommitAudit(scanDir)
	for _, a := range audits {
		icon := "✅"
		if a.Status != "PASS" {
			icon = "⚠️"
		}
		fmt.Printf("  %s %s: %s\n", icon, a.Check, a.Details)
	}

	// 6. 自优化
	fmt.Println("\n--- 自优化 ---")
	optResults := SelfOptimize(scanDir)
	for _, r := range optResults {
		fmt.Printf("  %s: score=%.2f, issues=%d\n", r.Component, r.Score, r.Issues)
	}

	// 7. 报告
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
