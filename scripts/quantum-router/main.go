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
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"time"
)

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

type Route struct {
	Task              string   `json:"task"`
	Intent            string   `json:"intent"`
	Selected          string   `json:"selected"`
	Fallbacks         []string `json:"fallbacks"`
	Classes           []string `json:"classes"`
	TrajectoryHash    string   `json:"trajectory_hash"`
	TokenBudgetHint   string   `json:"token_budget_hint"`
	Reason            string   `json:"reason"`
	FullTrajectory    []string `json:"full_tool_call_trajectory"`
	ApexDeltaGFormula string   `json:"apex_delta_g_formula"`
	GeneratedAt       string   `json:"generated_at"`
}

func main() {
	mode := flag.String("mode", "route", "list|route")
	task := flag.String("task", "", "task description")
	jsonOut := flag.Bool("json", true, "json output")
	flag.Parse()

	models, err := getModels()
	if err != nil {
		fatal(err)
	}
	for i := range models {
		classify(&models[i])
	}
	sort.Slice(models, func(i, j int) bool { return models[i].Score > models[j].Score })

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

func getModels() ([]Model, error) {
	seen := map[string]Model{}
	commands := [][]string{
		{"models", "list"},
		{"models", "list", "--all", "--provider", "zai"},
		{"models", "list", "--all", "--provider", "deepseek"},
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
		if line == "" || strings.HasPrefix(line, "Model ") {
			continue
		}
		m := re.FindStringSubmatch(line)
		if len(m) == 0 {
			continue
		}
		tags := []string{}
		if strings.TrimSpace(m[6]) != "" {
			for _, t := range strings.Split(m[6], ",") {
				tags = append(tags, strings.TrimSpace(t))
			}
		}
		out = append(out, Model{Ref: m[1], Input: m[2], Ctx: m[3], Auth: m[5] == "yes", Tags: tags})
	}
	return out
}

func classify(m *Model) {
	ref := strings.ToLower(m.Ref)
	add := func(c string) {
		for _, x := range m.Classes {
			if x == c {
				return
			}
		}
		m.Classes = append(m.Classes, c)
	}
	m.Score = 1
	// A: 国内/免费/计划池优先。zai 是智谱/Z.AI，已纳入量子路由器，但按任务类型选择：
	// - glm-5.1 / glm-5-turbo: 高端推理/高速推理
	// - glm-4.7: 代码与通用开发
	// - glm-5v/glm-4.6v: 多模态理解
	if strings.Contains(ref, "xiaomimimo") || strings.Contains(ref, "freemodel") || strings.Contains(ref, "scnet/") || strings.Contains(ref, "minimax-portal-cn") || strings.Contains(ref, "zai/") {
		add("A国内/免费或计划池")
		m.FreeScore += 1.0
	}
	if strings.Contains(ref, "deepseek/deepseek-v4-pro") || strings.Contains(ref, "gpt-5.5") || strings.Contains(ref, "reasoner") || strings.Contains(ref, "minimax-m2.7") || strings.Contains(ref, "glm-5.1") || strings.Contains(ref, "glm-5-turbo") {
		add("B高端推理")
		m.Score += 3
	}
	if strings.Contains(ref, "codex") || strings.Contains(ref, "deepseek-v4") || strings.Contains(ref, "qwen") || strings.Contains(ref, "deepseek-chat") || strings.Contains(ref, "glm-4.7") || strings.Contains(ref, "glm-5") {
		add("C代码深度开发")
		m.Score += 2
	}
	if strings.Contains(m.Input, "image") || strings.Contains(ref, "glm-5v") || strings.Contains(ref, "glm-4.6v") {
		add("D多模态理解")
		m.Score += 2
	}
	if strings.Contains(ref, "highspeed") || strings.Contains(ref, "flash") || strings.Contains(ref, "turbo") || strings.Contains(ref, "mimo-2.5") {
		add("fast")
		m.Score += 1.5
	}
	if strings.Contains(ref, "pro") || strings.Contains(ref, "gpt-5.5") {
		add("strong")
		m.Score += 1
	}
	if strings.Contains(m.Ctx, "977k") {
		add("long-context")
		m.Score += 1.2
	}
	if len(m.Classes) == 0 {
		add("general")
	}
}

func chooseRoute(models []Model, task string) Route {
	intent := inferIntent(task)
	var candidates []Model
	want := map[string]bool{}
	switch intent {
	case "code":
		want["C代码深度开发"] = true
	case "reasoning":
		want["B高端推理"] = true
	case "multimodal":
		want["D多模态理解"] = true
	case "free-fast":
		want["A国内/免费或计划池"] = true
		want["fast"] = true
	default:
		want["fast"] = true
		want["A国内/免费或计划池"] = true
	}
	for _, m := range models {
		cm := m
		for _, c := range m.Classes {
			if want[c] {
				cm.Score += 4
			}
		}
		if intent == "free-fast" && cm.FreeScore > 0 {
			cm.Score += 3
		}
		candidates = append(candidates, cm)
	}
	sort.Slice(candidates, func(i, j int) bool { return candidates[i].Score > candidates[j].Score })
	selected := ""
	classes := []string{}
	fallbacks := []string{}
	if len(candidates) > 0 {
		selected = candidates[0].Ref
		classes = candidates[0].Classes
	}
	for i := 1; i < len(candidates) && len(fallbacks) < 4; i++ {
		fallbacks = append(fallbacks, candidates[i].Ref)
	}
	h := sha256.Sum256([]byte(intent + "\n" + task + "\n" + selected))
	hash := hex.EncodeToString(h[:])[:16]
	_ = writeCache(hash, task, selected, fallbacks)
	return Route{Task: task, Intent: intent, Selected: selected, Fallbacks: fallbacks, Classes: classes, TrajectoryHash: hash, TokenBudgetHint: tokenHint(intent), Reason: "按意图分类、Auth 可用性、速度/推理/代码/多模态标签和上下文长度综合打分选择。免费/计划池优先走 fast，失败后降级到同类 fallback。", FullTrajectory: trajectory(intent), ApexDeltaGFormula: "ΔG=(C_total·Λ_gene·Ω_entropy·τ_traj)/(H_info·t)", GeneratedAt: time.Now().Format(time.RFC3339)}
}

func inferIntent(task string) string {
	s := strings.ToLower(task)
	if strings.Contains(s, "代码") || strings.Contains(s, "code") || strings.Contains(s, "bug") || strings.Contains(s, "开发") || strings.Contains(s, "实现") {
		return "code"
	}
	if strings.Contains(s, "图片") || strings.Contains(s, "图像") || strings.Contains(s, "视频") || strings.Contains(s, "image") || strings.Contains(s, "video") {
		return "multimodal"
	}
	if strings.Contains(s, "推理") || strings.Contains(s, "证明") || strings.Contains(s, "复杂") || strings.Contains(s, "研究") || strings.Contains(s, "论文") {
		return "reasoning"
	}
	return "free-fast"
}

func tokenHint(intent string) string {
	switch intent {
	case "code":
		return "先压缩上下文到相关文件+错误日志；max output 中高；必要时长上下文模型。"
	case "reasoning":
		return "保留问题、约束、证据；分配高推理预算；输出前要求自检。"
	case "multimodal":
		return "仅传必要媒体；文本提示短而结构化；生成任务优先专用 image/video 工具。"
	default:
		return "免费/高速优先；短上下文；失败自动换同类模型。"
	}
}

func trajectory(intent string) []string {
	base := []string{"1. classify_intent", "2. select_model_by_score", "3. generate_full_tool_call_trajectory", "4. execute_independent_threads", "5. verify_outputs", "6. cache_successful_trajectory", "7. distill_to_skill_if_reusable"}
	if intent == "code" {
		base = append(base, "8. run_test_or_lint_gate")
	}
	if intent == "reasoning" {
		base = append(base, "8. anti_hallucination_evidence_gate")
	}
	return base
}

func writeCache(hash, task, selected string, fallbacks []string) error {
	dir := filepath.Join(".openclaw", "quantum-router")
	_ = os.MkdirAll(dir, 0755)
	b, _ := json.MarshalIndent(map[string]any{"hash": hash, "task": task, "selected": selected, "fallbacks": fallbacks, "ts": time.Now().Format(time.RFC3339)}, "", "  ")
	return os.WriteFile(filepath.Join(dir, hash+".json"), b, 0600)
}

func printJSON(v any, jsonOut bool) { b, _ := json.MarshalIndent(v, "", "  "); fmt.Println(string(b)) }
func fatal(err error)               { fmt.Fprintln(os.Stderr, "quantum-router:", err); os.Exit(1) }
