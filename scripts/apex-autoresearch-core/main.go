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

type Evidence struct {
	Source   string `json:"source"`
	Claim    string `json:"claim"`
	Kind     string `json:"kind"`
	Verified bool   `json:"verified"`
}
type Hypothesis struct {
	ID       string   `json:"id"`
	Text     string   `json:"text"`
	Support  []string `json:"support"`
	Risks    []string `json:"risks"`
	NextTest string   `json:"next_test"`
}
type ResearchPlan struct {
	ID               string       `json:"id"`
	CreatedAt        string       `json:"created_at"`
	Question         string       `json:"question"`
	Mode             string       `json:"mode"`
	EvidenceLedger   []Evidence   `json:"evidence_ledger"`
	Hypotheses       []Hypothesis `json:"hypotheses"`
	Critique         []string     `json:"critique"`
	VerificationPlan []string     `json:"verification_plan"`
	NoFakeData       []string     `json:"no_fake_data_rules"`
	NextAction       string       `json:"next_action"`
}

func main() {
	mode := flag.String("mode", "plan", "plan|ledger|hypotheses|critique")
	q := flag.String("question", "", "research question")
	evidenceFile := flag.String("evidence", "", "optional evidence JSON")
	out := flag.String("out", "", "optional output path")
	flag.Parse()
	if strings.TrimSpace(*q) == "" {
		fmt.Fprintln(os.Stderr, "Error: --question required")
		os.Exit(2)
	}
	ev := loadEvidence(*evidenceFile)
	plan := BuildPlan(*q, *mode, ev)
	switch *mode {
	case "ledger":
		emit(plan.EvidenceLedger, *out)
	case "hypotheses":
		emit(plan.Hypotheses, *out)
	case "critique":
		emit(plan.Critique, *out)
	default:
		emit(plan, *out)
	}
}

func BuildPlan(q, mode string, ev []Evidence) ResearchPlan {
	q = strings.TrimSpace(q)
	ev = normalizeEvidence(ev)
	hyps := makeHypotheses(q, ev)
	critique := makeCritique(q, ev, hyps)
	vp := verificationPlan(q)
	return ResearchPlan{ID: "apex-research-" + hash(q+time.Now().String()), CreatedAt: time.Now().Format(time.RFC3339), Question: q, Mode: mode, EvidenceLedger: ev, Hypotheses: hyps, Critique: critique, VerificationPlan: vp, NoFakeData: []string{"No claim without source evidence.", "Separate hypothesis from verified finding.", "Unknown effect size/p-value stays unknown.", "Local data results require executable script and artifact path."}, NextAction: nextAction(ev)}
}
func normalizeEvidence(ev []Evidence) []Evidence {
	for i := range ev {
		ev[i].Source = strings.TrimSpace(ev[i].Source)
		ev[i].Claim = strings.TrimSpace(ev[i].Claim)
		if ev[i].Kind == "" {
			ev[i].Kind = "unknown"
		}
		ev[i].Verified = ev[i].Source != "" && ev[i].Claim != ""
	}
	sort.Slice(ev, func(i, j int) bool { return ev[i].Source < ev[j].Source })
	return ev
}
func makeHypotheses(q string, ev []Evidence) []Hypothesis {
	base := []string{}
	for _, e := range ev {
		if e.Verified {
			base = append(base, e.Source)
		}
	}
	if len(base) == 0 {
		base = []string{"no_verified_source_yet"}
	}
	return []Hypothesis{{ID: "H1", Text: "Primary hypothesis for: " + q, Support: base, Risks: []string{"insufficient external validation", "confounding or selection bias", "literature/data mismatch"}, NextTest: "collect at least 3 independent evidence sources and run local reproducible analysis if data exists"}, {ID: "H0", Text: "Null/negative hypothesis: current evidence is insufficient or effect is not robust", Support: []string{"default skeptical prior"}, Risks: []string{"may miss weak but real signal"}, NextTest: "define falsification threshold before analysis"}}
}
func makeCritique(q string, ev []Evidence, hyps []Hypothesis) []string {
	c := []string{"Do not promote H1 unless evidence ledger has verified sources.", "Report negative findings explicitly.", "Check whether the question asks mechanism, association, prediction, or intervention."}
	if len(ev) < 3 {
		c = append(c, "Evidence ledger has fewer than 3 sources; research answer must stay provisional.")
	}
	if strings.Contains(strings.ToLower(q), "biomarker") {
		c = append(c, "Biomarker claims need effect size, uncertainty, multiple testing control, and external validation.")
	}
	return c
}
func verificationPlan(q string) []string {
	xs := []string{"build evidence ledger", "classify claims as verified/hypothesis/unknown", "define falsification criteria", "run local reproducible analysis if dataset exists", "write report with limitations"}
	if strings.Contains(strings.ToLower(q), "phn") {
		xs = append(xs, "for PHN: verify IFI6/IFN findings against existing project scripts and figures")
	}
	return xs
}
func nextAction(ev []Evidence) string {
	if len(ev) < 3 {
		return "collect_more_verified_evidence"
	}
	return "run_domain_specific_analysis"
}
func loadEvidence(path string) []Evidence {
	if path == "" {
		return nil
	}
	b, err := os.ReadFile(path)
	if err != nil {
		panic(err)
	}
	var ev []Evidence
	if err := json.Unmarshal(b, &ev); err != nil {
		panic(err)
	}
	return ev
}
func emit(v any, out string) {
	b, _ := json.MarshalIndent(v, "", "  ")
	if out != "" {
		os.WriteFile(out, append(b, '\n'), 0644)
	}
	fmt.Println(string(b))
}
func hash(s string) string { h := sha256.Sum256([]byte(s)); return hex.EncodeToString(h[:])[:10] }
