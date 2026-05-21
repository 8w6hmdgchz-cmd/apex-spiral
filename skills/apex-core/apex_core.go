// apex_core.go - APEX 核心库
// APEX 自进化推理引擎 - Go 实现
// 璇玑帝国 · OpenClaw Native Integration
package main

import (
	"encoding/json"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"strings"
	"sync"
)

// ============ APEX 主公式 ============
// ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)

// APEX 主公式参数
type APEXState struct {
	Lambda   float64 // Λ 根增益
	Theta    float64 // Θ LLM效能
	Kappa    float64 // K 技能掌握
	Xi       float64 // ξ 置信度
	Psi      float64 // Ψ 自我迭代
	Phi      float64 // Φ 正反馈强化
	H        float64 // H 熵(混乱)
	T        float64 // T 时间(周期)
	Epsilon  float64 // ε 损失
}

// EvalDeltaG 计算 APEX ΔG
func EvalDeltaG(s *APEXState) float64 {
	numerator := s.Lambda * s.Theta * s.Kappa * s.Xi * s.Psi * s.Phi
	denominator := s.H * s.T * s.Epsilon
	if denominator == 0 {
		return 0
	}
	return numerator / denominator
}

// GradeDeltaG 评估 ΔG 等级
func GradeDeltaG(dg float64) string {
	switch {
	case dg >= 1.618:
		return "S"
	case dg >= 1.0:
		return "A"
	case dg >= 0.5:
		return "B"
	case dg >= 0.1:
		return "C"
	case dg >= 0.01:
		return "D"
	default:
		return "F"
	}
}

// EvalResult APEX 评估结果
type EvalResult struct {
	DeltaG       float64  `json:"delta_g"`
	Grade        string   `json:"grade"`
	Psi          float64  `json:"psi"`
	Xi           float64  `json:"xi"`
	Phi          float64  `json:"phi"`
	Epsilon      float64  `json:"epsilon"`
	H            float64  `json:"h"`
	T            float64  `json:"t"`
	Bottlenecks  []string `json:"bottlenecks"`
	Suggestions  []string `json:"suggestions"`
}

// EvaluateState 评估当前 APEX 状态
func EvaluateState(s *APEXState) *EvalResult {
	dg := EvalDeltaG(s)
	grade := GradeDeltaG(dg)

	bottlenecks := []string{}
	suggestions := []string{}

	// 短板检测
	if s.H > 0.5 {
		bottlenecks = append(bottlenecks, "高熵(混乱度)")
		suggestions = append(suggestions, "减少熵增，保持有序")
	}
	if s.T > 0.3 {
		bottlenecks = append(bottlenecks, "周期过长")
		suggestions = append(suggestions, "缩短反馈周期")
	}
	if s.Epsilon > 0.2 {
		bottlenecks = append(bottlenecks, "损失过高")
		suggestions = append(suggestions, "优化损失函数")
	}
	if s.Xi < 0.7 {
		bottlenecks = append(bottlenecks, "置信度不足")
		suggestions = append(suggestions, "增强自我验证")
	}
	if s.Psi < 0.5 {
		bottlenecks = append(bottlenecks, "自我迭代弱")
		suggestions = append(suggestions, "增强反思闭环")
	}
	if s.Phi < 0.5 {
		bottlenecks = append(bottlenecks, "正反馈不足")
		suggestions = append(suggestions, "建立强化机制")
	}

	return &EvalResult{
		DeltaG:      dg,
		Grade:       grade,
		Psi:         s.Psi,
		Xi:          s.Xi,
		Phi:         s.Phi,
		Epsilon:     s.Epsilon,
		H:           s.H,
		T:           s.T,
		Bottlenecks: bottlenecks,
		Suggestions: suggestions,
	}
}

// ============ APEX 公式代入 ============

// SubstitutionInput 代入输入
type SubstitutionInput struct {
	Task       string  `json:"task"`
	Capability float64 `json:"capability"` // [0,1] 任务所需能力
	History    float64 `json:"history"`    // [0,1] 历史表现
	Resource   float64 `json:"resource"`   // [0,1] 资源可用性
}

// SubstitutionOutput 代入输出
type SubstitutionOutput struct {
	Psi      float64 `json:"psi"`       // 自我迭代能力差距
	Xi       float64 `json:"xi"`        // 置信度
	Phi      float64 `json:"phi"`       // 正反馈强度
	Bottleneck string `json:"bottleneck"` // 最大短板
	Grade    string  `json:"grade"`      // A/B/C/D
}

// Substitute APEX 公式代入
func Substitute(in *SubstitutionInput) *SubstitutionOutput {
	// Ψ = 能力差距修正
	psi := 1.0 - math.Min(1.0, math.Abs(in.Capability-in.History)/0.5)

	// ξ = 资源调整置信度
	xi := math.Min(1.0, in.Resource*0.5 + in.History*0.5)

	// Φ = 正反馈基于历史表现
	phi := in.History * 0.8

	// 找最大短板
	bottleneck := ""
	if in.Capability > in.History+0.3 {
		bottleneck = "能力缺口"
	} else if in.Resource < 0.5 {
		bottleneck = "资源不足"
	} else {
		bottleneck = "自我迭代"
	}

	// 评分
	grade := "D"
	if psi > 0.8 && xi > 0.8 && phi > 0.8 {
		grade = "A"
	} else if psi > 0.6 && xi > 0.6 && phi > 0.6 {
		grade = "B"
	} else if psi > 0.4 && xi > 0.4 && phi > 0.4 {
		grade = "C"
	}

	return &SubstitutionOutput{
		Psi:       psi,
		Xi:        xi,
		Phi:       phi,
		Bottleneck: bottleneck,
		Grade:     grade,
	}
}

// ============ SWRs 海马体重放 ============

// Gene 技能基因
type Gene struct {
	ID          string  `json:"gene_id"`
	Name        string  `json:"name"`
	Fitness     float64 `json:"fitness"`
	SuccessRate float64 `json:"success_rate"`
	Uses        int     `json:"uses"`
}

// SWRResult SWR 触发结果
type SWRResult struct {
	Consolidated bool    `json:"consolidated"`
	BufferSize   int     `json:"buffer_size"`
	SkillBankLen int     `json:"skillbank_len"`
	Fitness      float64 `json:"fitness"`
}

const SWR_THRESHOLD = 0.7

var swrMutex sync.Mutex
var swrBuffer []Gene

// AddExperience 高 fitness 经验入缓冲
func AddExperience(gene *Gene) *SWRResult {
	swrMutex.Lock()
	defer swrMutex.Unlock()

	result := &SWRResult{Fitness: gene.Fitness}

	if gene.Fitness < SWR_THRESHOLD {
		// 过滤低 fitness
		result.BufferSize = len(swrBuffer)
		result.SkillBankLen = len(swrBuffer)
		return result
	}

	swrBuffer = append(swrBuffer, *gene)
	if len(swrBuffer) > 100 {
		swrBuffer = swrBuffer[1:]
	}

	result.Consolidated = true
	result.BufferSize = len(swrBuffer)

	return result
}

// GetSkillBank 获取当前技能库
func GetSkillBank() []*Gene {
	swrMutex.Lock()
	defer swrMutex.Unlock()

	skills := make([]*Gene, len(swrBuffer))
	for i := range swrBuffer {
		skills[i] = &swrBuffer[i]
	}
	return skills
}

// LoadSkillBank 从文件加载技能库
func LoadSkillBank(path string) error {
	swrMutex.Lock()
	defer swrMutex.Unlock()

	data, err := os.ReadFile(path)
	if err != nil {
		return err
	}

	var genes []Gene
	if err := json.Unmarshal(data, &genes); err != nil {
		return err
	}

	swrBuffer = genes
	return nil
}

// SaveSkillBank 保存技能库到文件
func SaveSkillBank(path string) error {
	swrMutex.Lock()
	defer swrMutex.Unlock()

	// 确保目录存在
	dir := filepath.Dir(path)
	os.MkdirAll(dir, 0755)

	data, err := json.MarshalIndent(swrBuffer, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(path, data, 0644)
}

// ============ 工具函数 ============

// JSONResponse 统一 JSON 响应
func JSONResponse(w interface{}) {
	enc := json.NewEncoder(os.Stdout)
	enc.SetEscapeHTML(false)
	enc.Encode(w)
}

// FatalJSON 错误时输出 JSON 并退出
func FatalJSON(err error) {
	fmt.Fprintf(os.Stderr, "Error: %v\n", err)
	os.Exit(1)
}

// ============ 主入口 ============

func main() {
	if len(os.Args) < 2 {
		fmt.Println("APEX Core CLI - APEX 自进化推理引擎")
		fmt.Println("用法:")
		fmt.Println("  apex_core substitute -t <task> -c <cap> -h <hist> -r <res>")
		fmt.Println("  apex_core eval -d <delta_g>")
		fmt.Println("  apex_core swr -add <gene_id> -f <fitness>")
		fmt.Println("  apex_core skillbank -load <path> -save <path>")
		os.Exit(1)
	}

	cmd := os.Args[1]

	switch cmd {
	case "substitute":
		// 解析参数
		args := parseArgs(os.Args[2:])
		task := getArg(args, "-t", "APEX任务")
		cap := getFloatArg(args, "-c", 0.5)
		hist := getFloatArg(args, "-h", 0.5)
		res := getFloatArg(args, "-r", 0.5)

		in := &SubstitutionInput{
			Task:       task,
			Capability: cap,
			History:    hist,
			Resource:   res,
		}
		out := Substitute(in)
		JSONResponse(out)

	case "eval":
		args := parseArgs(os.Args[2:])
		lambda := getFloatArg(args, "-l", 0.9)
		theta := getFloatArg(args, "-t", 0.8)
		kappa := getFloatArg(args, "-k", 0.7)
		xi := getFloatArg(args, "-x", 0.8)
		psi := getFloatArg(args, "-p", 0.6)
		phi := getFloatArg(args, "-f", 0.6)
		h := getFloatArg(args, "-he", 0.3)
		t := getFloatArg(args, "-ti", 0.2)
		eps := getFloatArg(args, "-e", 0.1)

		s := &APEXState{
			Lambda:  lambda,
			Theta:   theta,
			Kappa:   kappa,
			Xi:      xi,
			Psi:     psi,
			Phi:     phi,
			H:       h,
			T:       t,
			Epsilon: eps,
		}
		result := EvaluateState(s)
		JSONResponse(result)

	case "swr":
		args := parseArgs(os.Args[2:])
		if getArg(args, "-add", "") != "" {
			geneID := getArg(args, "-add", "")
			fitness := getFloatArg(args, "-f", 0.5)
			name := getArg(args, "-n", geneID)
			gene := &Gene{ID: geneID, Name: name, Fitness: fitness}
			result := AddExperience(gene)
			JSONResponse(result)
		}

	case "skillbank":
		args := parseArgs(os.Args[2:])
		if path := getArg(args, "-load", ""); path != "" {
			if err := LoadSkillBank(path); err != nil {
				FatalJSON(err)
			}
			fmt.Printf("Loaded %d skills\n", len(swrBuffer))
		}
		if path := getArg(args, "-save", ""); path != "" {
			if err := SaveSkillBank(path); err != nil {
				FatalJSON(err)
			}
			fmt.Printf("Saved %d skills\n", len(swrBuffer))
		}

	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", cmd)
		os.Exit(1)
	}
}

// ============ 参数解析 ============

type argsMap map[string]string

func parseArgs(raw []string) argsMap {
	m := make(argsMap)
	for i := 0; i < len(raw)-1; i++ {
		if strings.HasPrefix(raw[i], "-") {
			m[raw[i]] = raw[i+1]
			i++
		}
	}
	return m
}

func getArg(m argsMap, key, def string) string {
	if v, ok := m[key]; ok {
		return v
	}
	return def
}

func getFloatArg(m argsMap, key string, def float64) float64 {
	if v, ok := m[key]; ok {
		var f float64
		fmt.Sscanf(v, "%f", &f)
		return f
	}
	return def
}
