package main

import (
	"bufio"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"regexp"
	"sort"
	"strings"
	"time"
)

// ========================================
// APEX ΔE Quantum Router
// ========================================

type Model struct {
	Ref       string   `json:"ref"`
	Input     string   `json:"input"`
	Ctx       string   `json:"ctx"`
	Auth      bool     `json:"auth"`
	Tags      []string `json:"tags,omitempty"`
	Classes   []string `json:"classes"`
	Score     float64  `json:"score"`
	FreeScore float64  `json:"free_score"`
}

// APEX ΔE dimension scores per model
type ApexDeltaEScores struct {
	AlphaPsi  float64 `json:"alpha_psi"`  // Reasoning depth
	BetaOmega float64 `json:"beta_omega"` // Code capability
	LambdaPhi float64 `json:"lambda_phi"` // Knowledge breadth
	NablaTheta float64 `json:"nabla_theta"` // Iteration delta
	EvolCode  float64 `json:"evol_code"` // Self-evolution
}

type Route struct {
	Task                string          `json:"task"`
	Intent              string          `json:"intent"`
	Selected            string          `json:"selected"`
	Fallbacks           []string        `json:"fallbacks"`
	Classes             []string        `json:"classes"`
	TrajectoryHash      string          `json:"trajectory_hash"`
	TokenBudgetHint     string          `json:"token_budget_hint"`
	Reason              string          `json:"reason"`
	FullTrajectory      []string        `json:"full_tool_call_trajectory"`
	ApexDeltaE          float64         `json:"apex_delta_e"`
	ApexDimensions      ApexDeltaEScores `json:"apex_dimensions"`
	GeneratedAt         string          `json:"generated_at"`
}

func main() {
	mode := flag.String("mode", "route", "list|route|apex-calc")
	task := flag.String("task", "", "task description")
	jsonOut := flag.Bool("json", true, "json output")
	alpha := flag.Float64("alpha", 0, "αΨ score override")
	beta := flag.Float64("beta", 0, "βΩ score override")
	lambda := flag.Float64("lambda", 0, "λΦ score override")
	nabla := flag.Float64("nabla", 0, "∇Θ score override")
	evol := flag.Float64("evol", 0, "Evol_code score override")
	flag.Parse()

	if *mode == "apex-calc" {
		calcApexDeltaE(*alpha, *beta, *lambda, *nabla, *evol, *jsonOut)
		return
	}

	models, err := getModels()
	if err != nil {
		fatal(err)
	}
	for i := range models {
		classify(&models[i])
	}

	// Sort by combined APEX ΔE score
	scoreModel(&models)
	sort.Slice(models, func(i, j int) bool {
		return models[i].Score > models[j].Score
	})

	switch *mode {
	case "list":
		printJSON(models, *jsonOut)
	case "route":
		r := chooseRoute(models, *task)
		printJSON(r, *jsonOut)
	default:
		fatal(fmt.Errorf("unknown mode %q", *mode))
	}
}

// ========================================
// APEX ΔE Scoring
// ========================================

func calcApexDeltaE(alpha, beta, lambda, nabla, evol float64, jsonOut bool) {
	// Clamp to 0-100
	clamp := func(v float64) float64 {
		if v < 0 { return 0 }
		if v > 100 { return 100 }
		return v
	}
	a, b, l, n, e := clamp(alpha), clamp(beta), clamp(lambda), clamp(nabla), clamp(evol)
	total := a + b + l + n + e

	// Find bottleneck
	type dim struct {
		name string
		val  float64
	}
	dims := []dim{
		{"αΨ", a}, {"βΩ", b}, {"λΦ", l}, {"∇Θ", n}, {"Evol_code", e},
	}
	bottleneck := dims[0].name
	minVal := dims[0].val
	for _, d := range dims[1:] {
		if d.val < minVal {
			minVal = d.val
			bottleneck = d.name
		}
	}

	// Diagnosis
	var issues []string
	if a < 30 { issues = append(issues, fmt.Sprintf("αΨ=%.1f LLM路由能力不足", a)) }
	if b < 30 { issues = append(issues, fmt.Sprintf("βΩ=%.1f 代码架构需要重构", b)) }
	if l < 30 { issues = append(issues, fmt.Sprintf("λΦ=%.1f 知识吸收不足", l)) }
	if n < 10 { issues = append(issues, fmt.Sprintf("∇Θ=%.1f 认知迭代停滞", n)) }
	if e < 20 { issues = append(issues, fmt.Sprintf("Evol_code=%.1f 自演化效率低", e)) }
	if len(issues) == 0 { issues = append(issues, "ALL DIMENSIONS STABLE ✓") }

	// Generate directive
	directives := map[string]string{
		"αΨ": "IMPROVE_LLM_ROUTING: optimize model selection, add fallbacks, reduce latency",
		"βΩ": "REFACTOR_CODE: restructure core architecture, fix vulnerabilities, optimize performance",
		"λΦ": "EXPAND_KNOWLEDGE: scavenge new sources, absorb recent papers, refresh stale data",
		"∇Θ": "ACCELERATE_ITERATION: increase evolution frequency, push harder deltas",
		"Evol_code": "ENHANCE_SELF_MODIFICATION: improve code gen quality, increase test coverage",
	}
	directive := directives[bottleneck]
	if directive == "" {
		directive = "MAINTAIN: all dimensions stable, continue current trajectory"
	}

	result := map[string]interface{}{
		"apex_delta_e": map[string]float64{
			"αΨ": a, "βΩ": b, "λΦ": l, "∇Θ": n, "Evol_code": e,
		},
		"total":            total,
		"max_possible":     500.0,
		"progress_pct":     fmt.Sprintf("%.1f%%", total/500.0*100),
		"bottleneck":       bottleneck,
		"directive":        directive,
		"issues":           issues,
		"timestamp":        time.Now().UTC().Format(time.RFC3339),
	}

	if jsonOut {
		b, _ := json.MarshalIndent(result, "", "  ")
		fmt.Println(string(b))
	} else {
		fmt.Printf("APEX ΔE = %.1f / 500 (%.1f%%)\n", total, total/500.0*100)
		fmt.Printf("Bottleneck: %s → %s\n", bottleneck, directive)
		for _, issue := range issues {
			fmt.Printf("  • %s\n", issue)
		}
	}
}

// ========================================
// Model Classification & Scoring
// ========================================

// Classify sets tags and classes based on model provider/name
func classify(m *Model) {
	ref := strings.ToLower(m.Ref)
	
	// Default classes
	m.Classes = []string{}

	if strings.Contains(ref, "deepseek/deepseek-v4") || strings.Contains(ref, "deepseek/deepseek-reasoner") {
		m.Classes = append(m.Classes, "B:高端推理", "code")
		m.Tags = append(m.Tags, "high-reasoning", "code-capable")
	}
	if strings.Contains(ref, "deepseek/deepseek-chat") || strings.Contains(ref, "scnet/qwen") {
		m.Classes = append(m.Classes, "C:代码开发")
		m.Tags = append(m.Tags, "code", "execution")
	}
	if strings.Contains(ref, "freemodel/gpt-5") {
		m.Classes = append(m.Classes, "B:高端推理", "audit")
		m.Tags = append(m.Tags, "high-reasoning", "audit-capable")
	}
	if strings.Contains(ref, "xiaomimimo") {
		m.Classes = append(m.Classes, "A:免费池")
		m.Tags = append(m.Tags, "free", "fast")
	}
	if strings.Contains(ref, "minimax") {
		if strings.Contains(ref, "2.7") {
			m.Classes = append(m.Classes, "B:高端推理")
		} else {
			m.Classes = append(m.Classes, "A:免费池")
		}
		m.Tags = append(m.Tags, "balanced")
	}
	if strings.Contains(ref, "zai/glm-5") && !strings.Contains(ref, "v") {
		m.Classes = append(m.Classes, "B:高端推理")
		m.Tags = append(m.Tags, "high-reasoning")
	}
	if strings.Contains(ref, "zai/glm") && strings.Contains(ref, "v") {
		m.Classes = append(m.Classes, "D:多模态")
		m.Tags = append(m.Tags, "vision")
	}
	if strings.Contains(ref, "codex") {
		m.Classes = append(m.Classes, "C:代码开发")
		m.Tags = append(m.Tags, "code", "expert-code")
	}
	
	if len(m.Classes) == 0 {
		m.Classes = append(m.Classes, "A:免费池")
	}
}

// scoreModel calculates combined APEX ΔE score for each model
func scoreModel(models *[]Model) {
	for i := range *models {
		m := &(*models)[i]
		alpha, beta, lambda, nabla, evol := modelDimensionScores(m)
		m.Score = alpha + beta + lambda + nabla + evol
	}
}

// modelDimensionScores returns APEX ΔE dimension scores for a model
func modelDimensionScores(m *Model) (alpha, beta, lambda, nabla, evol float64) {
	ref := strings.ToLower(m.Ref)

	// αΨ - Reasoning/logic capability
	if containsAny(ref, "deepseek-v4-pro", "gpt-5.5", "glm-5.1") {
		alpha = 95
	} else if containsAny(ref, "deepseek-reasoner", "gpt-5.3", "minimax-2.7") {
		alpha = 88
	} else if containsAny(ref, "deepseek-chat", "scnet/qwen", "glm-5") {
		alpha = 78
	} else if containsAny(ref, "qwen", "glm-4") {
		alpha = 70
	} else {
		alpha = 60
	}

	// βΩ - Code architecture capability
	if containsAny(ref, "codex", "deepseek-v4") {
		beta = 92
	} else if containsAny(ref, "qwen", "scnet/qwen") {
		beta = 85
	} else if containsAny(ref, "gpt-5") {
		beta = 80
	} else if containsAny(ref, "glm-5", "minimax-2.7") {
		beta = 75
	} else {
		beta = 65
	}

	// λΦ - Knowledge breadth
	if containsAny(ref, "gpt-5", "deepseek-v") {
		lambda = 90
	} else if containsAny(ref, "minimax-2.7", "glm-5") {
		lambda = 82
	} else if containsAny(ref, "qwen", "scnet") {
		lambda = 80
	} else {
		lambda = 70
	}

	// ∇Θ - Iteration delta (how fast this model can iterate)
	if containsAny(ref, "deepseek-chat", "qwen", "fast") {
		nabla = 75
	} else if containsAny(ref, "gpt-5.3", "codex") {
		nabla = 70
	} else {
		nabla = 60
	}

	// Evol_code - Self-evolution support
	if containsAny(ref, "deepseek-v4", "codex") {
		evol = 85
	} else if containsAny(ref, "gpt-5") {
		evol = 78
	} else if containsAny(ref, "scnet", "qwen") {
		evol = 72
	} else {
		evol = 60
	}

	return
}

func containsAny(s string, subs ...string) bool {
	for _, sub := range subs {
		if strings.Contains(s, sub) {
			return true
		}
	}
	return false
}

// ========================================
// Routing Engine
// ========================================

func chooseRoute(models []Model, task string) Route {
	intent := classifyIntent(task)
	selected, fallbacks := selectModels(models, models, intent)
	hash := generateTrajectoryHash(task, selected.Ref)

	// Generate full trajectory
	trajectory := generateTrajectory(task, intent, selected.Ref)
	
	// Calculate APEX ΔE for selected model
	a, b, l, n, e := modelDimensionScores(&selected)
	apexTotal := a + b + l + n + e

	return Route{
		Task:    task,
		Intent:  intent,
		Selected: selected.Ref,
		Fallbacks: fallbacks,
		Classes: selected.Classes,
		TrajectoryHash: hash[:12],
		TokenBudgetHint: tokenBudget(intent),
		Reason: fmt.Sprintf("APEX ΔE=%.0f | %s | bottleneck: %s", 
			apexTotal, intent, bottleneckLabel(a, b, l, n, e)),
		FullTrajectory: trajectory,
		ApexDeltaE: apexTotal,
		ApexDimensions: ApexDeltaEScores{
			AlphaPsi: a, BetaOmega: b, LambdaPhi: l,
			NablaTheta: n, EvolCode: e,
		},
		GeneratedAt: time.Now().UTC().Format(time.RFC3339),
	}
}

func bottleneckLabel(alpha, beta, lambda, nabla, evol float64) string {
	min := alpha
	name := "αΨ"
	if beta < min { min = beta; name = "βΩ" }
	if lambda < min { min = lambda; name = "λΦ" }
	if nabla < min { min = nabla; name = "∇Θ" }
	if evol < min { name = "Evol_code" }
	return name
}

func classifyIntent(task string) string {
	t := strings.ToLower(task)
	switch {
	case strings.Contains(t, "code"), strings.Contains(t, "代码"), 
		 strings.Contains(t, "rust"), strings.Contains(t, "go"), 
		 strings.Contains(t, "compile"), strings.Contains(t, "编译"):
		return "code"
	case strings.Contains(t, "audit"), strings.Contains(t, "审计"), 
		 strings.Contains(t, "review"), strings.Contains(t, "review"),
		 strings.Contains(t, "check"):
		return "audit"
	case strings.Contains(t, "image"), strings.Contains(t, "图片"), 
		 strings.Contains(t, "photo"), strings.Contains(t, "draw"):
		return "vision"
	case strings.Contains(t, "研究"), strings.Contains(t, "research"),
		 strings.Contains(t, "analyze"), strings.Contains(t, "分析"),
		 strings.Contains(t, "医学"), strings.Contains(t, "medical"):
		return "research"
	case strings.Contains(t, "chat"), strings.Contains(t, "talk"),
		 strings.Contains(t, "闲聊"):
		return "chat"
	default:
		return "general"
	}
}

func selectModels(models, _ []Model, intent string) (Model, []string) {
	var candidates []Model
	for _, m := range models {
		for _, c := range m.Classes {
			switch intent {
			case "code":
				if strings.Contains(c, "代码") || strings.Contains(c, "B:高端推理") {
					candidates = append(candidates, m)
				}
			case "audit":
				if strings.Contains(c, "高端推理") || strings.Contains(c, "audit") {
					candidates = append(candidates, m)
				}
			case "research":
				if strings.Contains(c, "B:高端推理") || strings.Contains(c, "C:代码开发") {
					candidates = append(candidates, m)
				}
			case "vision":
				if strings.Contains(c, "D:多模态") {
					candidates = append(candidates, m)
				}
			default:
				if strings.Contains(c, "A:免费池") || strings.Contains(c, "B:高端推理") {
					candidates = append(candidates, m)
				}
			}
		}
	}

	if len(candidates) == 0 {
		candidates = models
	}

	sort.Slice(candidates, func(i, j int) bool {
		return candidates[i].Score > candidates[j].Score
	})

	var fallbacks []string
	for i := 1; i < len(candidates) && i <= 3; i++ {
		fallbacks = append(fallbacks, candidates[i].Ref)
	}

	if len(candidates) == 0 {
		return Model{Ref: "unknown"}, fallbacks
	}
	return candidates[0], fallbacks
}

func generateTrajectory(task, intent, selectedModel string) []string {
	trajectory := []string{
		fmt.Sprintf("1. classify_intent → %s", intent),
		fmt.Sprintf("2. compute_apex_de → sort by αΨ+βΩ+λΦ+∇Θ+Evol_code"),
		fmt.Sprintf("3. select_model → %s (APEX ΔE max)", selectedModel),
		fmt.Sprintf("4. generate_trajectory → %d steps", 5),
		"5. execute → spawn sub-agents if complex task",
		"6. verify → run output gate (evidence/code-test/format)",
		"7. cache → store trajectory hash for reuse",
	}
	return trajectory
}

func generateTrajectoryHash(task, model string) string {
	h := sha256.Sum256([]byte(task + model + time.Now().Format("2006-01-02")))
	return hex.EncodeToString(h[:])
}

func tokenBudget(intent string) string {
	switch intent {
	case "code": return "HIGH (8K-32K)"
	case "chat": return "LOW (2K-8K)"
	case "audit", "research": return "MAX (32K+)"
	default: return "MEDIUM (4K-16K)"
	}
}

// ========================================
// Utilities
// ========================================

func getModels() ([]Model, error) {
	seen := map[string]Model{}
	commands := [][]string{
		{"models", "list"},
		{"models", "list", "--all", "--provider", "zai"},
		{"models", "list", "--all", "--provider", "deepseek"},
		{"models", "list", "--all", "--provider", "minimax-portal"},
	}
	
	for _, args := range commands {
		out, err := exec.Command("openclaw", args...).CombinedOutput()
		if err != nil && len(out) == 0 {
			continue
		}
		for _, m := range parseModels(string(out)) {
			seen[m.Ref] = m
		}
	}

	var models []Model
	for _, m := range seen {
		if m.Auth {
			models = append(models, m)
		}
	}
	return models, nil
}

func parseModels(s string) []Model {
	var out []Model
	scanner := bufio.NewScanner(strings.NewReader(s))
	re := regexp.MustCompile(`^([^\s]+)\s+([^\s]+)\s+([^\s]+)\s+([^\s]+)\s+([^\s]+)\s*(.*)$`)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "Ref") || strings.HasPrefix(line, "---") {
			continue
		}
		parts := re.FindStringSubmatch(line)
		if len(parts) < 6 {
			continue
		}
		m := Model{
			Ref:   parts[1],
			Input: parts[2],
			Ctx:   parts[3],
		}
		m.Auth = strings.Contains(parts[5], "yes") || strings.Contains(parts[5], "true")
		out = append(out, m)
	}
	return out
}

func printJSON(v interface{}, indent bool) {
	if indent {
		b, _ := json.MarshalIndent(v, "", "  ")
		fmt.Println(string(b))
	} else {
		b, _ := json.Marshal(v)
		fmt.Println(string(b))
	}
}

func fatal(err error) {
	fmt.Fprintf(os.Stderr, "Error: %v\n", err)
	os.Exit(1)
}

// ========================================
// File watcher for autonomous evolution
// ========================================

