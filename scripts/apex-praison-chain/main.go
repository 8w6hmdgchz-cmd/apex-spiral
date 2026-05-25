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

// APEX Praison Chain
// Distills PraisonAI-style multi-agent orchestration into a deterministic local planner.
// Core formula:
//   ApexPraisonChain = RoleAgents × TaskGraph × ProcessMode × ToolGate × VerifyLoop × MemLedger

type AgentRole struct {
	Name         string   `json:"name"`
	Role         string   `json:"role"`
	Goal         string   `json:"goal"`
	Backstory    string   `json:"backstory"`
	Tools        []string `json:"tools"`
	Guardrails   []string `json:"guardrails"`
	Deliverables []string `json:"deliverables"`
}

type TaskNode struct {
	ID       string   `json:"id"`
	Title    string   `json:"title"`
	Agent    string   `json:"agent"`
	Depends  []string `json:"depends_on"`
	Input    string   `json:"input"`
	Expected string   `json:"expected_output"`
	Verify   []string `json:"verify"`
	Status   string   `json:"status"`
}

type ChainPlan struct {
	ID          string      `json:"id"`
	CreatedAt   string      `json:"created_at"`
	Objective   string      `json:"objective"`
	ProcessMode string      `json:"process_mode"`
	APEXFormula string      `json:"apex_formula"`
	Agents      []AgentRole `json:"agents"`
	Tasks       []TaskNode  `json:"tasks"`
	Score       Score       `json:"score"`
	NextAction  string      `json:"next_action"`
}

type Score struct {
	RoleCoverage      float64 `json:"role_coverage"`
	DependencyClarity float64 `json:"dependency_clarity"`
	VerificationPower float64 `json:"verification_power"`
	ToolSafety        float64 `json:"tool_safety"`
	MemoryContinuity  float64 `json:"memory_continuity"`
	Composite         float64 `json:"composite"`
}

func main() {
	mode := flag.String("mode", "plan", "plan|roles|score|skill")
	objective := flag.String("task", "", "task/objective to transform into a Praison-style APEX chain")
	process := flag.String("process", "hierarchical", "sequential|parallel|hierarchical")
	out := flag.String("out", "", "optional JSON output path")
	flag.Parse()

	if *mode == "skill" {
		printSkillSummary()
		return
	}
	if strings.TrimSpace(*objective) == "" {
		fmt.Fprintln(os.Stderr, "Error: --task is required unless --mode skill")
		os.Exit(1)
	}

	plan := BuildChainPlan(*objective, *process)
	switch *mode {
	case "plan":
		emit(plan, *out)
	case "roles":
		emit(plan.Agents, *out)
	case "score":
		emit(plan.Score, *out)
	default:
		fmt.Fprintf(os.Stderr, "unknown mode: %s\n", *mode)
		os.Exit(1)
	}
}

func BuildChainPlan(objective, process string) ChainPlan {
	objective = strings.TrimSpace(objective)
	process = normalizeProcess(process)
	agents := deriveAgents(objective)
	tasks := deriveTasks(objective, agents, process)
	score := scorePlan(agents, tasks, process)
	id := shortHash(objective + process + time.Now().Format("20060102150405"))
	return ChainPlan{
		ID:          "apex-praison-" + id,
		CreatedAt:   time.Now().Format(time.RFC3339),
		Objective:   objective,
		ProcessMode: process,
		APEXFormula: "ApexPraisonChain = RoleAgents × TaskGraph × ProcessMode × ToolGate × VerifyLoop × MemLedger",
		Agents:      agents,
		Tasks:       tasks,
		Score:       score,
		NextAction:  nextAction(score),
	}
}

func deriveAgents(objective string) []AgentRole {
	lower := strings.ToLower(objective)
	agents := []AgentRole{
		{Name: "planner", Role: "strategy architect", Goal: "decompose the objective into a minimal dependency graph", Backstory: "PraisonAI-style coordinator: role-first planning, explicit task ownership, no hidden work.", Tools: []string{"read", "exec", "web_search"}, Guardrails: []string{"state assumptions", "avoid external writes without approval"}, Deliverables: []string{"task graph", "acceptance criteria"}},
		{Name: "builder", Role: "implementation agent", Goal: "produce concrete files, code, commands, or artifacts", Backstory: "Execution-focused agent: every claim should map to an artifact or command output.", Tools: []string{"read", "write", "edit", "exec"}, Guardrails: []string{"small reversible changes", "compile before claim"}, Deliverables: []string{"patches", "build logs", "artifact paths"}},
		{Name: "critic", Role: "verification agent", Goal: "find failure modes and verify with the smallest meaningful gate", Backstory: "Adversarial reviewer: rejects self-scoring without evidence.", Tools: []string{"exec", "read"}, Guardrails: []string{"do not rubber-stamp", "cite failing evidence"}, Deliverables: []string{"test result", "risk list", "go/no-go"}},
	}
	if containsAny(lower, []string{"paper", "research", "phn", "gene", "biomarker", "医学", "科研"}) {
		agents = append(agents, AgentRole{Name: "researcher", Role: "scientific evidence agent", Goal: "separate verified findings from hypotheses", Backstory: "Research-specialized role for literature/data/protocol critique.", Tools: []string{"web_search", "web_fetch", "exec"}, Guardrails: []string{"no fabricated results", "distinguish local data from literature"}, Deliverables: []string{"evidence ledger", "analysis protocol"}})
	}
	if containsAny(lower, []string{"github", "repo", "skill", "代码", "module", "build"}) {
		agents = append(agents, AgentRole{Name: "scavenger", Role: "repository distillation agent", Goal: "turn external repositories into local reusable patterns", Backstory: "GitHub hunter: clone shallowly, extract interfaces, distill into skills.", Tools: []string{"exec", "read", "write"}, Guardrails: []string{"use SSH when GitHub HTTPS is unreliable", "record source commit"}, Deliverables: []string{"distillation notes", "local skill"}})
	}
	return agents
}

func deriveTasks(objective string, agents []AgentRole, process string) []TaskNode {
	var tasks []TaskNode
	add := func(id, title, agent string, deps []string, expected string, verify []string) {
		tasks = append(tasks, TaskNode{ID: id, Title: title, Agent: agent, Depends: deps, Input: objective, Expected: expected, Verify: verify, Status: "pending"})
	}
	add("T1", "Clarify objective and constraints", "planner", nil, "one-page plan with acceptance criteria", []string{"objective is restated as testable outputs"})
	if process == "parallel" || process == "hierarchical" {
		add("T2", "Build or modify concrete artifact", "builder", []string{"T1"}, "files or executable artifact", []string{"git diff shows intentional changes", "build/check passes"})
		add("T3", "Distill reusable knowledge", chooseAgent(agents, "scavenger", "planner"), []string{"T1"}, "memory/skill entry with source and usage", []string{"source path/commit recorded", "future invocation documented"})
		add("T4", "Independent verification", "critic", []string{"T2", "T3"}, "go/no-go report", []string{"smallest meaningful test passed", "known limitations listed"})
	} else {
		add("T2", "Execute artifact", "builder", []string{"T1"}, "concrete artifact", []string{"build/check passes"})
		add("T3", "Verify and archive", "critic", []string{"T2"}, "verification and memory update", []string{"result reproducible", "memory updated if durable"})
	}
	return tasks
}

func scorePlan(agents []AgentRole, tasks []TaskNode, process string) Score {
	roleCoverage := min(1, float64(len(agents))/4)
	depCount := 0
	verifyCount := 0
	for _, t := range tasks {
		depCount += len(t.Depends)
		verifyCount += len(t.Verify)
	}
	depClarity := min(1, float64(depCount+1)/float64(len(tasks)+1))
	verifyPower := min(1, float64(verifyCount)/float64(len(tasks)*2))
	toolSafety := 0.85
	if process == "parallel" {
		toolSafety = 0.78
	}
	memory := 0.80
	comp := 0.25*roleCoverage + 0.20*depClarity + 0.25*verifyPower + 0.15*toolSafety + 0.15*memory
	return Score{round(roleCoverage), round(depClarity), round(verifyPower), round(toolSafety), round(memory), round(comp)}
}

func nextAction(s Score) string {
	if s.Composite >= 0.82 {
		return "execute_with_verification"
	}
	if s.VerificationPower < 0.70 {
		return "strengthen_acceptance_tests"
	}
	if s.DependencyClarity < 0.70 {
		return "split_dependencies_more_clearly"
	}
	return "execute_after_one_more_review"
}

func normalizeProcess(p string) string {
	switch strings.ToLower(strings.TrimSpace(p)) {
	case "sequential", "parallel", "hierarchical":
		return strings.ToLower(strings.TrimSpace(p))
	default:
		return "hierarchical"
	}
}

func chooseAgent(agents []AgentRole, preferred, fallback string) string {
	for _, a := range agents {
		if a.Name == preferred {
			return preferred
		}
	}
	return fallback
}

func containsAny(s string, needles []string) bool {
	for _, n := range needles {
		if strings.Contains(s, strings.ToLower(n)) || strings.Contains(s, n) {
			return true
		}
	}
	return false
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

func printSkillSummary() {
	items := []string{
		"RoleAgents: planner/builder/critic plus domain roles",
		"TaskGraph: explicit dependencies and expected outputs",
		"ProcessMode: sequential, parallel, or hierarchical",
		"ToolGate: allowed tools and guardrails per role",
		"VerifyLoop: smallest meaningful verification before claims",
		"MemLedger: distill durable lessons into memory/skills",
	}
	sort.Strings(items)
	for _, item := range items {
		fmt.Println("- " + item)
	}
}

func shortHash(s string) string {
	h := sha256.Sum256([]byte(s))
	return hex.EncodeToString(h[:])[:10]
}

func min(a, b float64) float64 {
	if a < b {
		return a
	}
	return b
}

func round(x float64) float64 {
	return float64(int(x*1000+0.5)) / 1000
}
