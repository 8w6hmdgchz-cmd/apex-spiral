package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"time"
)

type Check struct {
	Name       string  `json:"name"`
	Command    string  `json:"command"`
	Status     string  `json:"status"`
	Score      float64 `json:"score"`
	DurationMs int64   `json:"duration_ms"`
	Output     string  `json:"output,omitempty"`
	Error      string  `json:"error,omitempty"`
}

type Report struct {
	Status      string  `json:"status"`
	StartedAt   string  `json:"started_at"`
	ReadyScore  float64 `json:"ready_score"`
	AutoLearn   float64 `json:"auto_learn"`
	Passed      int     `json:"passed"`
	Checked     int     `json:"checked"`
	Checks      []Check `json:"checks"`
	Format      string  `json:"format"`
}

func run(root, name string, args ...string) Check {
	start := time.Now()
	cmd := exec.Command(args[0], args[1:]...)
	cmd.Dir = root
	out, err := cmd.CombinedOutput()
	c := Check{Name: name, Command: fmt.Sprintf("%v", args), DurationMs: time.Since(start).Milliseconds(), Output: string(out)}
	if err != nil {
		c.Status = "failed"
		c.Error = err.Error()
		return c
	}
	c.Status = "success"
	c.Score = 1
	return c
}

func main() {
	root := flag.String("root", ".", "workspace root")
	out := flag.String("out", "", "write JSON report")
	flag.Parse()
	abs, _ := filepath.Abs(*root)

	checks := []Check{
		run(abs, "mini_executor_selftest", filepath.Join(abs, "scripts/apex-mini-executor/apex-mini-executor"), "--mode", "selftest", "--workspace", abs),
		run(abs, "eval_harness_selftest", filepath.Join(abs, "scripts/apex-eval-harness/apex-eval-harness"), "--mode", "selftest", "--workspace", abs, "--out", filepath.Join(abs, "state/apex-eval-harness-latest.json")),
		run(abs, "evidence_validator_selftest", filepath.Join(abs, "scripts/apex-evidence-validator/apex-evidence-validator"), "--mode", "selftest", "--input", filepath.Join(abs, "state/apex-evidence-selftest.json"), "--out", filepath.Join(abs, "state/apex-evidence-validator-latest.json")),
		run(abs, "hygiene_classifier", filepath.Join(abs, "scripts/apex-hygiene/apex-hygiene"), "--root", abs, "--out", filepath.Join(abs, "state/apex-hygiene-latest.json")),
	}

	passed := 0
	for _, c := range checks {
		if c.Score == 1 {
			passed++
		}
	}
	ready := 0.0
	if len(checks) > 0 {
		ready = float64(passed) / float64(len(checks))
	}
	// Preserve the V10.1 extraction/generalization/summarization intent, but bind it to observed gates.
	autoLearn := ready * 0.92
	rep := Report{Status: "success", StartedAt: time.Now().Format(time.RFC3339), ReadyScore: ready, AutoLearn: autoLearn, Passed: passed, Checked: len(checks), Checks: checks, Format: "apex-dawn-gate-1.0"}
	if passed != len(checks) {
		rep.Status = "failed"
	}
	b, _ := json.MarshalIndent(rep, "", "  ")
	if *out != "" {
		_ = os.WriteFile(*out, b, 0644)
	}
	fmt.Println(string(b))
	if rep.Status != "success" {
		os.Exit(1)
	}
}
