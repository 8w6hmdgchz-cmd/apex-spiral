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

type GateRun struct {
	Name       string `json:"name"`
	Command    string `json:"command"`
	OutputPath string `json:"output_path"`
	OK         bool   `json:"ok"`
	DurationMs int64  `json:"duration_ms"`
	Error      string `json:"error,omitempty"`
}

type FusionReport struct {
	ID          string    `json:"id"`
	StartedAt   string    `json:"started_at"`
	Status      string    `json:"status"`
	Root        string    `json:"root"`
	Gates       []GateRun `json:"gates"`
	Evidence    []string  `json:"evidence"`
	PhiMirror   string    `json:"phi_mirror"`
	Next        string    `json:"next"`
	Format      string    `json:"format"`
}

type EvidenceRecord struct {
	ID           string       `json:"id"`
	Claim        string       `json:"claim"`
	SourceRepo   string       `json:"source_repo"`
	SourceCommit string       `json:"source_commit"`
	SourcePath   string       `json:"source_path"`
	ContextID    string       `json:"context_id"`
	Score        float64      `json:"score"`
	Verification Verification `json:"verification"`
	MemoryType   string       `json:"memory_type"`
}

type Verification struct {
	Command      string `json:"command"`
	Result       string `json:"result"`
	EvidencePath string `json:"evidence_path"`
}

func main() {
	mode := flag.String("mode", "selftest", "selftest|run")
	root := flag.String("root", "/Users/lihongxin/.openclaw/workspace", "workspace root")
	out := flag.String("out", "", "report output")
	flag.Parse()
	abs, _ := filepath.Abs(*root)
	if *out == "" {
		*out = filepath.Join(abs, "state/apex-fusion-engine-latest.json")
	}
	report := runFusion(abs, *mode, *out)
	writeJSON(*out, report)
	fmtJSON(report)
	if report.Status != "success" {
		os.Exit(1)
	}
}

func runFusion(root, mode, out string) FusionReport {
	id := fmt.Sprintf("apex-fusion-%d", time.Now().Unix())
	rep := FusionReport{ID: id, StartedAt: time.Now().Format(time.RFC3339), Status: "success", Root: root, PhiMirror: "state/phi_v10_result.json", Format: "apex-fusion-engine-1.0"}
	state := filepath.Join(root, "state")
	gates := []struct{ name string; args []string; out string }{
		{"phasor_llm_route", []string{filepath.Join(root, "scripts/apex-phasor-llm/apex-phasor-llm"), "--mode", "route", "--task", "APEX fusion engine closes evomap evolver autoresearch superpowers openhands CLI MCP loop without virtual data", "--root", root, "--out", filepath.Join(state, "apex-phasor-llm-latest.json")}, "state/apex-phasor-llm-latest.json"},
		{"agent_dispatch_plan", []string{filepath.Join(root, "scripts/apex-agent-dispatch/apex-agent-dispatch"), "--mode", "selftest", "--root", root, "--out", filepath.Join(state, "apex-agent-dispatch-latest.json")}, "state/apex-agent-dispatch-latest.json"},
		{"evolver_cycle", []string{filepath.Join(root, "scripts/apex-evolver-core/apex-evolver-core"), "--mode", "cycle", "--root", root, "--out", filepath.Join(state, "apex-fusion-evolver.json")}, "state/apex-fusion-evolver.json"},
		{"autoresearch_plan", []string{filepath.Join(root, "scripts/apex-autoresearch-core/apex-autoresearch-core"), "--question", "APEX fusion engine closes evomap evolver autoresearch superpowers openhands CLI MCP loop without virtual data", "--out", filepath.Join(state, "apex-fusion-autoresearch.json")}, "state/apex-fusion-autoresearch.json"},
		{"harness_bridge", []string{filepath.Join(root, "scripts/apex-harness-bridge/apex-harness-bridge"), "--mode", "selftest"}, "stdout:harness_bridge"},
		{"superpowers_gate", []string{filepath.Join(root, "scripts/apex-superpowers-gate/apex-superpowers-gate"), "--mode", "full", "--input", filepath.Join(state, "apex-fusion-superpowers-input.json")}, "stdout:superpowers_gate"},
		{"dawn_gate", []string{filepath.Join(root, "scripts/apex-dawn-gate/apex-dawn-gate"), "--root", root, "--out", filepath.Join(state, "apex-dawn-gate-latest.json")}, "state/apex-dawn-gate-latest.json"},
	}
	writeSuperpowersInput(root)
	for _, g := range gates {
		gr := run(root, g.name, g.out, g.args...)
		rep.Gates = append(rep.Gates, gr)
		if gr.OK {
			rep.Evidence = append(rep.Evidence, g.out)
		} else {
			rep.Status = "failed"
		}
	}
	if rep.Status == "success" {
		rep.Next = "promote_to_phi_tracker_full_mirror"
		writeEvidence(root, rep, mode)
	} else {
		rep.Next = "repair_failed_gate"
	}
	return rep
}

func writeSuperpowersInput(root string) {
	in := map[string]any{
		"task": "APEX fusion engine closes evomap/evolver/autoresearch/superpowers/openhands/CLI/MCP loop",
		"requirements": []string{"must pass phasor route/dispatch/executor/eval/evidence/hygiene/dawn gates", "must produce evidence paths", "must reject virtual data"},
		"architecture": []string{"fusion engine orchestrates local binaries and records artifacts"},
		"tests": []string{"go build ./scripts/apex-fusion-engine", "apex-fusion-engine --mode selftest"},
		"implementation": []string{"scripts/apex-fusion-engine/main.go", "scripts/apex-fusion-engine/go.mod"},
		"verification": []string{"state/apex-fusion-engine-latest.json", "state/apex-fusion-evidence.json"},
		"evidence": []string{"state/apex-phasor-llm-latest.json", "state/apex-agent-dispatch-latest.json", "state/apex-dawn-gate-latest.json", "state/phi_v10_result.json"},
	}
	writeJSON(filepath.Join(root, "state/apex-fusion-superpowers-input.json"), in)
}

func run(root, name, out string, args ...string) GateRun {
	start := time.Now()
	cmd := exec.Command(args[0], args[1:]...)
	cmd.Dir = root
	b, err := cmd.CombinedOutput()
	gr := GateRun{Name: name, Command: fmt.Sprintf("%v", args), OutputPath: out, OK: err == nil, DurationMs: time.Since(start).Milliseconds()}
	if err != nil {
		gr.Error = err.Error() + ": " + string(b)
	}
	return gr
}

func writeEvidence(root string, rep FusionReport, mode string) {
	score := 1.0
	for _, g := range rep.Gates {
		if !g.OK {
			score = 0
		}
	}
	rec := []EvidenceRecord{{
		ID: "fusion:" + rep.ID,
		Claim: "APEX fusion engine executed phasor LLM routing, agent dispatch planning, evolver, autoresearch, superpowers, CLI/MCP harness bridge, and dawn gate with real evidence artifacts.",
		SourceRepo: "8w6hmdgchz-cmd/apex-spiral",
		SourceCommit: currentCommit(root),
		SourcePath: "scripts/apex-fusion-engine/main.go",
		ContextID: rep.ID,
		Score: score,
		Verification: Verification{Command: "scripts/apex-fusion-engine/apex-fusion-engine --mode selftest", Result: "pass", EvidencePath: "state/apex-fusion-engine-latest.json"},
		MemoryType: "Working",
	}}
	writeJSON(filepath.Join(root, "state/apex-fusion-evidence.json"), rec)
}

func currentCommit(root string) string {
	b, err := exec.Command("git", "-C", root, "rev-parse", "--short=12", "HEAD").Output()
	if err != nil { return "0000000" }
	return stringTrim(string(b))
}

func stringTrim(s string) string {
	for len(s) > 0 && (s[len(s)-1] == '\n' || s[len(s)-1] == '\r' || s[len(s)-1] == ' ') { s = s[:len(s)-1] }
	return s
}

func writeJSON(path string, v any) { b, _ := json.MarshalIndent(v, "", "  "); _ = os.WriteFile(path, append(b, '\n'), 0644) }
func fmtJSON(v any) { b, _ := json.MarshalIndent(v, "", "  "); fmt.Println(string(b)) }
