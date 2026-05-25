package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"strings"
	"time"
)

type GateInput struct {
	Task           string   `json:"task"`
	Requirements   []string `json:"requirements"`
	Architecture   []string `json:"architecture"`
	Tests          []string `json:"tests"`
	Implementation []string `json:"implementation"`
	Verification   []string `json:"verification"`
	Risks          []string `json:"risks"`
	Evidence       []string `json:"evidence"`
}
type GateResult struct {
	OK         bool     `json:"ok"`
	Stage      string   `json:"stage"`
	Score      float64  `json:"score"`
	Missing    []string `json:"missing"`
	Warnings   []string `json:"warnings"`
	NextAction string   `json:"next_action"`
	CreatedAt  string   `json:"created_at"`
}

func main() {
	mode := flag.String("mode", "full", "requirements|architecture|tests|implementation|verification|full|schema")
	input := flag.String("input", "", "json input path; stdin when empty")
	flag.Parse()
	if *mode == "schema" {
		schema()
		return
	}
	in, err := readInput(*input)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(2)
	}
	r := Gate(*mode, in)
	emit(r)
	if !r.OK {
		os.Exit(1)
	}
}
func readInput(path string) (GateInput, error) {
	var b []byte
	var err error
	if path == "" {
		b, err = os.ReadFile("/dev/stdin")
	} else {
		b, err = os.ReadFile(path)
	}
	if err != nil {
		return GateInput{}, err
	}
	var in GateInput
	err = json.Unmarshal(b, &in)
	return in, err
}
func Gate(mode string, in GateInput) GateResult {
	missing := []string{}
	warn := []string{}
	check := func(cond bool, msg string) {
		if !cond {
			missing = append(missing, msg)
		}
	}
	check(strings.TrimSpace(in.Task) != "", "task")
	if mode == "requirements" || mode == "full" {
		check(len(in.Requirements) > 0, "requirements")
		check(hasAcceptance(in.Requirements), "acceptance criteria")
	}
	if mode == "architecture" || mode == "full" {
		check(len(in.Architecture) > 0, "architecture")
	}
	if mode == "tests" || mode == "full" {
		check(len(in.Tests) > 0, "tests")
		check(hasConcrete(in.Tests), "concrete test command")
	}
	if mode == "implementation" || mode == "full" {
		check(len(in.Implementation) > 0, "implementation artifacts")
	}
	if mode == "verification" || mode == "full" {
		check(len(in.Verification) > 0, "verification evidence")
		check(len(in.Evidence) > 0, "evidence paths/output")
	}
	for _, x := range append(in.Evidence, in.Verification...) {
		l := strings.ToLower(x)
		if strings.Contains(l, "todo") || strings.Contains(l, "not run") || strings.Contains(l, "assume") {
			warn = append(warn, "weak evidence: "+x)
		}
	}
	score := score(missing, warn)
	ok := len(missing) == 0 && score >= 0.75
	return GateResult{OK: ok, Stage: mode, Score: score, Missing: missing, Warnings: warn, NextAction: next(missing, warn), CreatedAt: time.Now().Format(time.RFC3339)}
}
func hasAcceptance(xs []string) bool {
	for _, x := range xs {
		l := strings.ToLower(x)
		if strings.Contains(l, "accept") || strings.Contains(l, "验收") || strings.Contains(l, "must") || strings.Contains(l, "pass") {
			return true
		}
	}
	return false
}
func hasConcrete(xs []string) bool {
	for _, x := range xs {
		l := strings.ToLower(x)
		if strings.Contains(l, "go build") || strings.Contains(l, "cargo check") || strings.Contains(l, "pytest") || strings.Contains(l, "npm test") || strings.Contains(l, "bash -n") || strings.Contains(l, "selftest") {
			return true
		}
	}
	return false
}
func score(m, w []string) float64 {
	s := 1.0 - float64(len(m))*0.16 - float64(len(w))*0.05
	if s < 0 {
		return 0
	}
	return float64(int(s*1000+0.5)) / 1000
}
func next(m, w []string) string {
	if len(m) > 0 {
		return "fill_missing:" + strings.Join(m, ",")
	}
	if len(w) > 0 {
		return "strengthen_weak_evidence"
	}
	return "promote"
}
func emit(v any) { b, _ := json.MarshalIndent(v, "", "  "); fmt.Println(string(b)) }
func schema() {
	emit(GateInput{Task: "...", Requirements: []string{"must pass ..."}, Architecture: []string{"..."}, Tests: []string{"go build ..."}, Implementation: []string{"file path"}, Verification: []string{"command output"}, Evidence: []string{"artifact path"}})
}
