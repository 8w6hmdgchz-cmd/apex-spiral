package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"sort"
	"strings"
	"time"
)

// APEX Devour Engine
// Devour = discover -> rank -> install -> audit -> distill -> reimplement -> verify -> archive.
// No fake data: candidate stars/commits must come from explicit evidence or installed source.

type Candidate struct {
	Repo        string   `json:"repo"`
	URL         string   `json:"url,omitempty"`
	Stars       int      `json:"stars,omitempty"`
	Commit      string   `json:"commit,omitempty"`
	Evidence    []string `json:"evidence,omitempty"`
	LocalPath   string   `json:"local_path,omitempty"`
	License     string   `json:"license,omitempty"`
	CoreSignals []string `json:"core_signals,omitempty"`
}

type Formula struct {
	RLBase     string   `json:"rl_base"`
	APEXARL    string   `json:"apex_arl"`
	ITotal     string   `json:"i_total"`
	CThink     string   `json:"c_think"`
	FiveLayers []string `json:"five_layers"`
	Hierarchy  string   `json:"hierarchy"`
}

type DevourPlan struct {
	ID          string      `json:"id"`
	CreatedAt   string      `json:"created_at"`
	Need        string      `json:"need"`
	Formula     Formula     `json:"formula"`
	Candidates  []Candidate `json:"candidates"`
	Top3        []Candidate `json:"top3"`
	Pipeline    []Stage     `json:"pipeline"`
	SafetyGates []string    `json:"safety_gates"`
	APEXScore   Score       `json:"apex_score"`
	NextAction  string      `json:"next_action"`
}

type Stage struct {
	Name    string   `json:"name"`
	Purpose string   `json:"purpose"`
	Inputs  []string `json:"inputs"`
	Outputs []string `json:"outputs"`
	Verify  []string `json:"verify"`
	NoFake  bool     `json:"no_fake_data"`
}

type Score struct {
	MetaG       float64 `json:"meta_g"`
	Reflect     float64 `json:"reflect"`
	LongPlan    float64 `json:"long_plan"`
	Evidence    float64 `json:"evidence"`
	Integration float64 `json:"integration"`
	Composite   float64 `json:"composite"`
}

func main() {
	mode := flag.String("mode", "plan", "formula|plan|rank|pipeline|score")
	need := flag.String("need", "", "user need/capability to devour")
	candidatesFile := flag.String("candidates", "", "optional JSON candidate list")
	out := flag.String("out", "", "optional output JSON path")
	flag.Parse()

	if *mode == "formula" {
		emit(defaultFormula(), *out)
		return
	}
	if strings.TrimSpace(*need) == "" {
		fmt.Fprintln(os.Stderr, "Error: --need is required")
		os.Exit(1)
	}
	candidates := loadCandidates(*candidatesFile)
	plan := BuildPlan(*need, candidates)
	switch *mode {
	case "plan":
		emit(plan, *out)
	case "rank":
		emit(plan.Top3, *out)
	case "pipeline":
		emit(plan.Pipeline, *out)
	case "score":
		emit(plan.APEXScore, *out)
	default:
		fmt.Fprintf(os.Stderr, "unknown mode: %s\n", *mode)
		os.Exit(1)
	}
}

func BuildPlan(need string, candidates []Candidate) DevourPlan {
	need = strings.TrimSpace(need)
	for i := range candidates {
		candidates[i].Evidence = compact(candidates[i].Evidence)
		candidates[i].CoreSignals = compact(candidates[i].CoreSignals)
	}
	top3 := rankTop3(candidates)
	score := scorePlan(need, top3)
	return DevourPlan{
		ID:          "apex-devour-" + shortHash(need+time.Now().Format("20060102150405")),
		CreatedAt:   time.Now().Format(time.RFC3339),
		Need:        need,
		Formula:     defaultFormula(),
		Candidates:  candidates,
		Top3:        top3,
		Pipeline:    defaultPipeline(),
		SafetyGates: safetyGates(),
		APEXScore:   score,
		NextAction:  nextAction(score, len(top3)),
	}
}

func defaultFormula() Formula {
	return Formula{
		RLBase:  `RL_base = π(a|s) -> R -> ∇π`,
		APEXARL: `APEX_ARL = RL ∪ {MetaG, Reflect, LongPlan}`,
		ITotal:  `I_total = M_base × C_think`,
		CThink:  `C_think = G_set + P_decompose + S_review`,
		FiveLayers: []string{
			`G_self != G_env`,
			`P_n = Split(G_total)`,
			`π_t = f(π_{t-1}, ΔE)`,
			`R_meta = Eval(Logic)`,
			`S_fix = Error -> Policy`,
		},
		Hierarchy: `ApexAgent ⊃ AgenticRL ⊃ StandardRL`,
	}
}

func defaultPipeline() []Stage {
	return []Stage{
		{Name: "discover", Purpose: "find top open-source references for the user need", Inputs: []string{"need", "web/GitHub/search evidence"}, Outputs: []string{"candidate list with source evidence"}, Verify: []string{"each candidate has repo URL or installed source"}, NoFake: true},
		{Name: "rank", Purpose: "select high-signal top3", Inputs: []string{"candidates"}, Outputs: []string{"top3 ranked list"}, Verify: []string{"ranking fields are explicit, missing stars are not invented"}, NoFake: true},
		{Name: "install", Purpose: "install selected repos through SSH/sparse/shallow clone", Inputs: []string{"top3 repos"}, Outputs: []string{"vendor or third_party source snapshots"}, Verify: []string{"commit hash recorded", "local files inspected"}, NoFake: true},
		{Name: "audit", Purpose: "read architecture/license/security/build surfaces", Inputs: []string{"installed source"}, Outputs: []string{"audit ledger"}, Verify: []string{"no secrets", "license noted", "unsafe operations blocked"}, NoFake: true},
		{Name: "distill", Purpose: "extract core capability pattern", Inputs: []string{"audit ledger", "source files"}, Outputs: []string{"formula", "interfaces", "skill contract"}, Verify: []string{"source paths cited", "no blind copy"}, NoFake: true},
		{Name: "reimplement", Purpose: "implement deterministic local core in Rust/C/Go", Inputs: []string{"distilled design"}, Outputs: []string{"local binary/skill"}, Verify: []string{"build passes", "smoke test passes"}, NoFake: true},
		{Name: "integrate", Purpose: "connect to OpenClaw/APEX harness/fusion engine", Inputs: []string{"binary", "SKILL.md", "harness"}, Outputs: []string{"callable skill"}, Verify: []string{"CLI invocation", "skill docs", "benchmark entry"}, NoFake: true},
		{Name: "archive", Purpose: "write evomap gene and failure/metric records", Inputs: []string{"verification"}, Outputs: []string{"EVOLUTION_MAP", "metrics", "memory"}, Verify: []string{"git diff shows records"}, NoFake: true},
	}
}

func safetyGates() []string {
	return []string{
		"No virtual stars/commits/benchmarks: unknown fields stay empty.",
		"External writes require explicit approval unless committing this workspace's own code.",
		"Secrets or private data hits block export/push.",
		"Do not claim superiority until local benchmark proves a measurable delta.",
		"Devour means learn, abstract, and reimplement safely; not license-unsafe copying.",
		"Every promoted capability needs build/check/smoke-test evidence.",
	}
}

func rankTop3(cs []Candidate) []Candidate {
	sorted := append([]Candidate(nil), cs...)
	sort.SliceStable(sorted, func(i, j int) bool {
		si := candidateScore(sorted[i])
		sj := candidateScore(sorted[j])
		if si == sj {
			return sorted[i].Repo < sorted[j].Repo
		}
		return si > sj
	})
	if len(sorted) > 3 {
		return sorted[:3]
	}
	return sorted
}

func candidateScore(c Candidate) int {
	score := c.Stars
	if c.Commit != "" {
		score += 1000000
	}
	if c.LocalPath != "" {
		score += 500000
	}
	score += len(c.Evidence) * 1000
	score += len(c.CoreSignals) * 500
	return score
}

func scorePlan(need string, top3 []Candidate) Score {
	metaG := 0.78
	reflect := 0.82
	longPlan := 0.86
	evidence := 0.35
	integration := 0.70
	if len(top3) > 0 {
		evidence += 0.15
	}
	if len(top3) >= 3 {
		evidence += 0.20
	}
	for _, c := range top3 {
		if c.Commit != "" && c.LocalPath != "" {
			evidence += 0.05
		}
		if len(c.CoreSignals) > 0 {
			integration += 0.03
		}
	}
	if strings.Contains(strings.ToLower(need), "mcp") || strings.Contains(strings.ToLower(need), "cli") {
		integration += 0.05
	}
	evidence = cap01(evidence)
	integration = cap01(integration)
	comp := 0.20*metaG + 0.20*reflect + 0.20*longPlan + 0.20*evidence + 0.20*integration
	return Score{round(metaG), round(reflect), round(longPlan), round(evidence), round(integration), round(comp)}
}

func nextAction(s Score, n int) string {
	if n < 3 {
		return "discover_more_candidates_before_claiming_top3"
	}
	if s.Evidence < 0.75 {
		return "install_and_record_commits_for_top3"
	}
	if s.Integration < 0.80 {
		return "build_local_rust_go_core_and_smoke_test"
	}
	return "promote_to_apex_harness_after_benchmark"
}

func loadCandidates(path string) []Candidate {
	if path == "" {
		return nil
	}
	b, err := os.ReadFile(path)
	if err != nil {
		panic(err)
	}
	var cs []Candidate
	if err := json.Unmarshal(b, &cs); err != nil {
		panic(err)
	}
	return cs
}

func emit(v any, out string) {
	b, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		panic(err)
	}
	if out != "" {
		if err := os.WriteFile(out, append(b, '\n'), 0644); err != nil {
			panic(err)
		}
	}
	fmt.Println(string(b))
}

func compact(xs []string) []string {
	seen := map[string]bool{}
	var out []string
	for _, x := range xs {
		x = strings.TrimSpace(x)
		if x != "" && !seen[x] {
			seen[x] = true
			out = append(out, x)
		}
	}
	return out
}

func shortHash(s string) string { h := sha256.Sum256([]byte(s)); return hex.EncodeToString(h[:])[:10] }
func cap01(x float64) float64 {
	if x < 0 {
		return 0
	}
	if x > 1 {
		return 1
	}
	return x
}
func round(x float64) float64 { return float64(int(x*1000+0.5)) / 1000 }
