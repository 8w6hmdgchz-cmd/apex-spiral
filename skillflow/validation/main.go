package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

type DatasetResult struct {
	DatasetID     string  `json:"dataset_id"`
	Domain        string  `json:"domain"`
	Status        string  `json:"status"` // "pass"|"fail"|"skip"
	Accuracy      float64 `json:"accuracy"`
	Trajectories  int     `json:"trajectories"`
	EvidencePath  string  `json:"evidence_path"`
}

type ValidationReport struct {
	ID        string          `json:"id"`
	StartedAt string          `json:"started_at"`
	Status    string          `json:"status"`
	Total     int             `json:"total"`
	Passed    int             `json:"passed"`
	Failed    int             `json:"failed"`
	Skipped   int             `json:"skipped"`
	Results   []DatasetResult `json:"results"`
	Format    string          `json:"format"`
}

// evalHarnessOutput represents the JSON output from apex-eval-harness --mode selftest
type EvalHarnessOutput struct {
	EvalID    string `json:"eval_id"`
	Status    string `json:"status"`
	StartedAt string `json:"started_at"`
	Samples   []struct {
		Task struct {
			ID             string `json:"id"`
			Command        string `json:"command"`
			ExpectContains string `json:"expect_contains"`
			Timeout        int    `json:"timeout"`
		} `json:"task"`
		Status    string  `json:"status"`
		Score     float64 `json:"score"`
		Output    string  `json:"output"`
		StartedAt string  `json:"started_at"`
		Duration  int64   `json:"duration_ms"`
		Trajectory string `json:"trajectory"`
	} `json:"samples"`
	Metrics struct {
		Accuracy float64 `json:"accuracy"`
	} `json:"metrics"`
	Format string `json:"format"`
}

func main() {
	root := flag.String("root", ".", "workspace root directory")
	flag.Parse()

	absRoot, err := filepath.Abs(*root)
	if err != nil {
		fmt.Fprintf(os.Stderr, "ERROR: bad root path %s: %v\n", *root, err)
		os.Exit(1)
	}

	harnessPath := filepath.Join(absRoot, "scripts", "apex-eval-harness", "apex-eval-harness")
	stateDir := filepath.Join(absRoot, "state")
	evidenceBase := filepath.Join(absRoot, "state", "skillflow-validation")

	// Ensure state directories exist
	os.MkdirAll(stateDir, 0755)
	os.MkdirAll(evidenceBase, 0755)

	startedAt := time.Now().UTC().Format(time.RFC3339)

	// 14 datasets
	datasets := []DatasetResult{
		{DatasetID: "mmlu_pro",             Domain: "reasoning", Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "humaneval_plus",       Domain: "code",      Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "math_o1",              Domain: "math",      Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "gaia_benchmark",       Domain: "agentic",   Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "arxiv_summarization",  Domain: "nlp",       Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "biology_qa",           Domain: "science",   Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "swebench",             Domain: "swe",       Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "mmlu_ethics",          Domain: "ethics",    Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "popqwa",               Domain: "reasoning", Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "realworld_agent",      Domain: "embodied",  Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "cybersecurity_ctf",    Domain: "security",  Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "financial_fraud",      Domain: "finance",   Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "medical_qa",           Domain: "biomed",    Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
		{DatasetID: "dialogue_safety",      Domain: "safety",    Status: "pending", Accuracy: 0, Trajectories: 0, EvidencePath: ""},
	}

	// Check if harness binary exists
	_, statErr := os.Stat(harnessPath)
	harnessExists := statErr == nil

	results := make([]DatasetResult, 0, len(datasets))
	passed, failed, skipped := 0, 0, 0

	for _, ds := range datasets {
		ds.EvidencePath = filepath.Join(evidenceBase, ds.DatasetID+".json")
		fmt.Printf("[%s] domain=%s → ", ds.DatasetID, ds.Domain)

		if !harnessExists {
			ds.Status = "skip"
			ds.Accuracy = 0
			ds.Trajectories = 0
			skipped++
			fmt.Println("SKIP (harness binary not found)")
			writeDatasetEvidence(ds, nil)
			results = append(results, ds)
			continue
		}

		// Run apex-eval-harness --mode selftest for this dataset
		cmd := exec.Command(harnessPath, "--mode", "selftest")
		cmd.Dir = absRoot
		cmd.Env = append(os.Environ(),
			fmt.Sprintf("APEX_EVAL_DATASET=%s", ds.DatasetID),
			fmt.Sprintf("APEX_EVAL_DOMAIN=%s", ds.Domain),
		)
		output, runErr := cmd.CombinedOutput()

		if runErr != nil {
			ds.Status = "fail"
			failed++
			fmt.Printf("FAIL (exec error: %v)\n", runErr)
			writeDatasetEvidenceRaw(ds, output)
			results = append(results, ds)
			continue
		}

		// Parse JSON output
		var harnessOutput EvalHarnessOutput
		if parseErr := json.Unmarshal(output, &harnessOutput); parseErr != nil {
			ds.Status = "fail"
			ds.Accuracy = 0
			ds.Trajectories = 0
			failed++
			fmt.Printf("FAIL (parse error: %v)\n", parseErr)
			writeDatasetEvidenceRaw(ds, output)
			results = append(results, ds)
			continue
		}

		// Count trajectories from samples
		trajCount := 0
		for _, s := range harnessOutput.Samples {
			if s.Trajectory != "" {
				trajCount++
			}
		}
		ds.Trajectories = trajCount
		ds.Accuracy = harnessOutput.Metrics.Accuracy

		// Determine status
		if harnessOutput.Status == "success" && ds.Accuracy >= 0.8 {
			ds.Status = "pass"
			passed++
			fmt.Printf("PASS (accuracy=%.2f, trajectories=%d)\n", ds.Accuracy, trajCount)
		} else if harnessOutput.Status == "success" && ds.Accuracy < 0.8 {
			ds.Status = "fail"
			failed++
			fmt.Printf("FAIL (low accuracy=%.2f, trajectories=%d)\n", ds.Accuracy, trajCount)
		} else {
			ds.Status = "fail"
			failed++
			fmt.Printf("FAIL (status=%s, accuracy=%.2f)\n", harnessOutput.Status, ds.Accuracy)
		}

		writeDatasetEvidence(ds, &harnessOutput)
		results = append(results, ds)
	}

	// Build report
	runID := fmt.Sprintf("apex-skillflow-val-%d", time.Now().UnixMilli())
	overallStatus := "pass"
	if failed > 0 {
		overallStatus = "fail"
	}
	if skipped == len(datasets) {
		overallStatus = "skip"
	}

	report := ValidationReport{
		ID:        runID,
		StartedAt: startedAt,
		Status:    overallStatus,
		Total:     len(datasets),
		Passed:    passed,
		Failed:    failed,
		Skipped:   skipped,
		Results:   results,
		Format:    "skillflow-validation-1.0",
	}

	// Write report to state dir
	reportPath := filepath.Join(stateDir, "skillflow-validation-latest.json")
	reportData, err := json.MarshalIndent(report, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "ERROR marshaling report: %v\n", err)
		os.Exit(1)
	}
	if err := os.WriteFile(reportPath, reportData, 0644); err != nil {
		fmt.Fprintf(os.Stderr, "ERROR writing report: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("\n=== Validation Report ===\n")
	fmt.Printf("  ID:       %s\n", runID)
	fmt.Printf("  Status:   %s\n", overallStatus)
	fmt.Printf("  Total:    %d\n", report.Total)
	fmt.Printf("  Passed:   %d\n", report.Passed)
	fmt.Printf("  Failed:   %d\n", report.Failed)
	fmt.Printf("  Skipped:  %d\n", report.Skipped)
	fmt.Printf("  Report:   %s\n", reportPath)
	fmt.Printf("=========================\n")
}

func writeDatasetEvidence(ds DatasetResult, harnessOutput *EvalHarnessOutput) {
	ev := map[string]interface{}{
		"dataset_id":   ds.DatasetID,
		"domain":       ds.Domain,
		"status":       ds.Status,
		"accuracy":     ds.Accuracy,
		"trajectories": ds.Trajectories,
	}
	if harnessOutput != nil {
		ev["eval_id"] = harnessOutput.EvalID
		ev["harness_status"] = harnessOutput.Status

		samples := make([]map[string]interface{}, 0)
		for _, s := range harnessOutput.Samples {
			samples = append(samples, map[string]interface{}{
				"task_id":    s.Task.ID,
				"status":     s.Status,
				"score":      s.Score,
				"trajectory": s.Trajectory,
			})
		}
		ev["samples"] = samples
	}
	data, _ := json.MarshalIndent(ev, "", "  ")
	os.WriteFile(ds.EvidencePath, data, 0644)
}

func writeDatasetEvidenceRaw(ds DatasetResult, rawOutput []byte) {
	ev := map[string]interface{}{
		"dataset_id": ds.DatasetID,
		"domain":     ds.Domain,
		"status":     ds.Status,
		"raw_output": strings.TrimSpace(string(rawOutput)),
	}
	data, _ := json.MarshalIndent(ev, "", "  ")
	os.WriteFile(ds.EvidencePath, data, 0644)
}
