// hetu_luoshu.go - 河图洛书 LLM路由与自我进化系统
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
)

// ============ 配置 ============

type Config struct {
	FreeModelAPIKey string `json:"free_model_api_key"`
	ZhipuAPIKey     string `json:"zhipu_api_key"`
	ScnetAPIKey     string `json:"scnet_api_key"`
	BaseURL         string `json:"base_url"`
	DefaultModel    string `json:"default_model"`
	EnableSelfCheck bool   `json:"enable_self_check"`
}

var defaultConfig = Config{
	FreeModelAPIKey: os.Getenv("FREEMODEL_API_KEY"),
	ZhipuAPIKey:     os.Getenv("ZHIPU_API_KEY"),
	ScnetAPIKey:     os.Getenv("SCNET_API_KEY"),
	BaseURL:         "https://api.freemodel.dev",
	DefaultModel:    "gpt-5.5",
	EnableSelfCheck: true,
}

// ============ 模型定义 ============

type Model struct {
	ID       string `json:"id"`
	Provider string `json:"provider"`
	Strength string `json:"strength"` // 推理/代码/快速/平衡
	CostTier int    `json:"cost_tier"` // 1=低, 2=中, 3=高
}

var models = []Model{
	{"gpt-5.5", "FreeModel", "推理", 3},
	{"gpt-5.4", "FreeModel", "平衡", 2},
	{"gpt-5.4-mini", "FreeModel", "快速", 1},
	{"gpt-5.3-codex", "FreeModel", "代码", 2},
	{"glm-5", "Zhipu", "推理", 3},
	{"glm-4.7", "Zhipu", "平衡", 2},
	{"glm-4.6", "Zhipu", "快速", 1},
	{"MiniMax-M2.5", "Scnet", "快速", 1},
}

// ============ 请求/响应 ============

type ChatRequest struct {
	Model    string        `json:"model"`
	Messages []ChatMessage `json:"messages"`
	MaxTokens int         `json:"max_tokens,omitempty"`
	Temperature float64   `json:"temperature,omitempty"`
}

type ChatMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

type ChatResponse struct {
	ID      string `json:"id"`
	Model   string `json:"model"`
	Choices []Choice `json:"choices"`
	Usage   Usage  `json:"usage"`
}

type Choice struct {
	Message ChatMessage `json:"message"`
}

type Usage struct {
	PromptTokens     int `json:"prompt_tokens"`
	CompletionTokens int `json:"completion_tokens"`
	TotalTokens      int `json:"total_tokens"`
}

// ============ 持久化 ============

const stateFile = "/Users/lihongxin/.openclaw/workspace/hetu_luoshu_state.json"

// LoadState 从文件加载状态
func LoadState() error {
	data, err := os.ReadFile(stateFile)
	if err != nil {
		return err
	}
	return json.Unmarshal(data, selfCheck)
}

// SaveState 保存状态到文件
func SaveState() error {
	selfCheck.mu.Lock()
	defer selfCheck.mu.Unlock()

	dir := filepath.Dir(stateFile)
	os.MkdirAll(dir, 0755)

	data, err := json.MarshalIndent(selfCheck, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(stateFile, data, 0644)
}

// ============ APEX 自检状态 ============

// SelfCheckState APEX自检状态
type SelfCheckState struct {
	Lambda   float64 // Λ 根增益
	Theta    float64 // Θ LLM效能 (token效率)
	Kappa    float64 // K 技能掌握
	Xi       float64 // ξ 置信度
	Psi      float64 // Ψ 自我迭代
	Phi      float64 // Φ 正反馈强化
	H        float64 // H 熵
	T        float64 // T 时间
	Epsilon  float64 // ε 损失
	TotalRequests int `json:"total_requests"`
	TotalTokens   int `json:"total_tokens"`
	FailCount     int `json:"fail_count"`
	mu            sync.Mutex
}

// init 初始化时加载状态
func init() {
	LoadState()
}

var selfCheck = &SelfCheckState{
	Lambda: 1.0,
	Theta:  1.0,
	Kappa:  1.0,
	Xi:     0.8,
	Psi:    0.7,
	Phi:    0.6,
	H:      0.2,
	T:      0.1,
	Epsilon: 0.1,
}

// ============ 河图路由 ============

// RouteTask 根据任务复杂度路由到最优模型
func RouteTask(task string) string {
	taskLower := strings.ToLower(task)

	// 代码任务
	if strings.Contains(taskLower, "code") || strings.Contains(taskLower, "函数") ||
		strings.Contains(taskLower, "python") || strings.Contains(taskLower, "golang") ||
		strings.Contains(taskLower, "rust") || strings.Contains(taskLower, "调试") {
		return "gpt-5.3-codex"
	}

	// 简单快速任务：使用已注册的轻量FreeModel，而不是未注册模型名
	if strings.Contains(taskLower, "hi") || strings.Contains(taskLower, "hello") ||
		strings.Contains(taskLower, "查询") || strings.Contains(taskLower, "状态") ||
		strings.Contains(taskLower, "时间") {
		return "gpt-5.4"
	}

	// 复杂推理任务
	if strings.Contains(taskLower, "分析") || strings.Contains(taskLower, "推理") ||
		strings.Contains(taskLower, "思考") || strings.Contains(taskLower, "判断") ||
		strings.Contains(taskLower, "比较") || strings.Contains(taskLower, "评估") ||
		strings.Contains(taskLower, "为什么") || strings.Contains(taskLower, "如何") {
		return "gpt-5.5"
	}

	// 默认主模型：GPT-5.5
	return "gpt-5.5"
}

// CallZhipu 调用智谱API
func CallZhipu(model, prompt string) (*ChatResponse, error) {
	cfg := defaultConfig
	if cfg.ZhipuAPIKey == "" {
		return nil, fmt.Errorf("ZHIPU_API_KEY not set")
	}

	reqBody := ChatRequest{
		Model: model,
		Messages: []ChatMessage{
			{Role: "user", Content: prompt},
		},
		MaxTokens: 2000,
	}

	reqJSON, err := json.Marshal(reqBody)
	if err != nil {
		return nil, err
	}

	url := "https://open.bigmodel.cn/api/paas/v4/chat/completions"
	req, err := http.NewRequest("POST", url, bytes.NewBuffer(reqJSON))
	if err != nil {
		return nil, err
	}

	req.Header.Set("Authorization", "Bearer "+cfg.ZhipuAPIKey)
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 60 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	if resp.StatusCode != 200 {
		return nil, fmt.Errorf("API error: %s", string(body))
	}

	var chatResp ChatResponse
	if err := json.Unmarshal(body, &chatResp); err != nil {
		return nil, err
	}

	UpdateMetrics(true, chatResp.Usage.TotalTokens)
	SaveState()
	return &chatResp, nil
}

// ============ 洛书进化 ============

// SelfCheckResult 自检结果
type SelfCheckResult struct {
	DeltaG       float64  `json:"delta_g"`
	Grade        string   `json:"grade"`
	Bottlenecks  []string `json:"bottlenecks"`
	Suggestions  []string `json:"suggestions"`
}

// PerformSelfCheck 执行APEX自检
func PerformSelfCheck() *SelfCheckResult {
	selfCheck.mu.Lock()
	defer selfCheck.mu.Unlock()
	return performSelfCheckUnsafe()
}

// performSelfCheckUnsafe 执行自检（不加锁，需要调用方持有锁）
func performSelfCheckUnsafe() *SelfCheckResult {

	result := &SelfCheckResult{}

	// 计算 ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)
	numerator := selfCheck.Lambda * selfCheck.Theta * selfCheck.Kappa * selfCheck.Xi * selfCheck.Psi * selfCheck.Phi
	denominator := selfCheck.H * selfCheck.T * selfCheck.Epsilon

	if denominator > 0 {
		result.DeltaG = numerator / denominator
	}

	// 评分
	if result.DeltaG >= 1.618 {
		result.Grade = "S"
	} else if result.DeltaG >= 1.0 {
		result.Grade = "A"
	} else if result.DeltaG >= 0.5 {
		result.Grade = "B"
	} else if result.DeltaG >= 0.1 {
		result.Grade = "C"
	} else {
		result.Grade = "D/F"
	}

	// 瓶颈识别
	if selfCheck.Xi < 0.7 {
		result.Bottlenecks = append(result.Bottlenecks, "置信度不足 (ξ<0.7)")
		result.Suggestions = append(result.Suggestions, "增强结果验证机制")
	}
	if selfCheck.Psi < 0.5 {
		result.Bottlenecks = append(result.Bottlenecks, "自我迭代弱 (Ψ<0.5)")
		result.Suggestions = append(result.Suggestions, "建立反思闭环")
	}
	if selfCheck.Phi < 0.5 {
		result.Bottlenecks = append(result.Bottlenecks, "正反馈不足 (Φ<0.5)")
		result.Suggestions = append(result.Suggestions, "积累成功经验")
	}
	if selfCheck.H > 0.5 {
		result.Bottlenecks = append(result.Bottlenecks, "高熵/混乱 (H>0.5)")
		result.Suggestions = append(result.Suggestions, "减少冗余，提高有序度")
	}
	if selfCheck.Epsilon > 0.3 {
		result.Bottlenecks = append(result.Bottlenecks, "损失过高 (ε>0.3)")
		result.Suggestions = append(result.Suggestions, "优化token使用效率")
	}

	return result
}

// UpdateMetrics 更新指标
func UpdateMetrics(success bool, tokensUsed int) {
	selfCheck.mu.Lock()
	defer selfCheck.mu.Unlock()

	selfCheck.TotalRequests++
	selfCheck.TotalTokens += tokensUsed

	if success {
		// 正向反馈
		selfCheck.Phi = min(1.0, selfCheck.Phi+0.01)
		selfCheck.Xi = min(1.0, selfCheck.Xi+0.02)
	} else {
		// 负向反馈
		selfCheck.FailCount++
		selfCheck.Phi = max(0, selfCheck.Phi-0.05)
		selfCheck.Epsilon = min(1.0, selfCheck.Epsilon+0.02)
	}

	// 根据历史调整Ψ
	successRate := 1.0 - float64(selfCheck.FailCount)/float64(selfCheck.TotalRequests)
	selfCheck.Psi = successRate * 0.8

	// 根据token效率调整Θ
	if selfCheck.TotalRequests > 10 {
		avgTokens := float64(selfCheck.TotalTokens) / float64(selfCheck.TotalRequests)
		if avgTokens < 500 {
			selfCheck.Theta = min(1.0, selfCheck.Theta+0.01)
		} else if avgTokens > 2000 {
			selfCheck.Theta = max(0.1, selfCheck.Theta-0.02)
		}
	}
}

func min(a, b float64) float64 {
	if a < b {
		return a
	}
	return b
}

func max(a, b float64) float64 {
	if a > b {
		return a
	}
	return b
}

// ============ LLM 调用 ============

// CallLLM 调用LLM
func CallLLM(model, prompt string) (*ChatResponse, error) {
	cfg := defaultConfig
	if cfg.FreeModelAPIKey == "" {
		return nil, fmt.Errorf("FREEMODEL_API_KEY not set")
	}

	reqBody := ChatRequest{
		Model: model,
		Messages: []ChatMessage{
			{Role: "user", Content: prompt},
		},
		MaxTokens: 2000,
	}

	reqJSON, err := json.Marshal(reqBody)
	if err != nil {
		return nil, err
	}

	url := cfg.BaseURL + "/v1/chat/completions"
	req, err := http.NewRequest("POST", url, bytes.NewBuffer(reqJSON))
	if err != nil {
		return nil, err
	}

	req.Header.Set("Authorization", "Bearer "+cfg.FreeModelAPIKey)
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 45 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		UpdateMetrics(false, 0)
		SaveState()
		return nil, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		UpdateMetrics(false, 0)
		SaveState()
		return nil, err
	}

	if resp.StatusCode != 200 {
		UpdateMetrics(false, 0)
		SaveState()
		return nil, fmt.Errorf("API error: %s", string(body))
	}

	var chatResp ChatResponse
	if err := json.Unmarshal(body, &chatResp); err != nil {
		UpdateMetrics(false, 0)
		SaveState()
		return nil, err
	}

	// 更新指标
	UpdateMetrics(true, chatResp.Usage.TotalTokens)
	SaveState()

	return &chatResp, nil
}

// fallbackChain 根据主模型生成兜底链。GPT-5.5超时时，优先用GPT-5.3-codex保持可用性。
func fallbackChain(model string) []string {
	seen := map[string]bool{}
	chain := []string{}
	add := func(m string) {
		if m != "" && !seen[m] {
			seen[m] = true
			chain = append(chain, m)
		}
	}
	add(model)
	if model == "gpt-5.5" {
		add("gpt-5.3-codex")
		add("gpt-5.4")
	} else if model == "gpt-5.3-codex" {
		add("gpt-5.5")
		add("gpt-5.4")
	} else {
		add("gpt-5.5")
		add("gpt-5.3-codex")
	}
	return chain
}

// CallWithFallback 调用模型并按兜底链重试。
func CallWithFallback(model, prompt string) (*ChatResponse, string, []string, error) {
	errors := []string{}
	for _, candidate := range fallbackChain(model) {
		var resp *ChatResponse
		var err error
		if strings.HasPrefix(candidate, "glm-") {
			resp, err = CallZhipu(candidate, prompt)
		} else if isScnetModel(candidate) {
			resp, err = CallScnet(candidate, prompt)
		} else {
			resp, err = CallLLM(candidate, prompt)
		}
		if err == nil {
			return resp, candidate, errors, nil
		}
		errors = append(errors, fmt.Sprintf("%s: %v", candidate, err))
	}
	return nil, "", errors, fmt.Errorf("all models failed")
}

// isScnetModel 判断是否为Scnet模型
func isScnetModel(model string) bool {
	return model == "MiniMax-M2.5"
}

// CallScnet 调用Scnet API
func CallScnet(model, prompt string) (*ChatResponse, error) {
	cfg := defaultConfig
	if cfg.ScnetAPIKey == "" {
		return nil, fmt.Errorf("SCNET_API_KEY not set")
	}

	reqBody := ChatRequest{
		Model: model,
		Messages: []ChatMessage{
			{Role: "user", Content: prompt},
		},
		MaxTokens: 2000,
	}

	reqJSON, err := json.Marshal(reqBody)
	if err != nil {
		return nil, err
	}

	url := "https://api.scnet.cn/api/llm/v1/chat/completions"
	req, err := http.NewRequest("POST", url, bytes.NewBuffer(reqJSON))
	if err != nil {
		return nil, err
	}

	req.Header.Set("Authorization", "Bearer "+cfg.ScnetAPIKey)
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 60 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	if resp.StatusCode != 200 {
		return nil, fmt.Errorf("API error: %s", string(body))
	}

	var chatResp ChatResponse
	if err := json.Unmarshal(body, &chatResp); err != nil {
		return nil, err
	}

	UpdateMetrics(true, chatResp.Usage.TotalTokens)
	SaveState()
	return &chatResp, nil
}

// ============ CLI ============

func main() {
	if len(os.Args) < 2 {
		fmt.Println("河图洛书 - LLM路由与自我进化系统")
		fmt.Println("用法:")
		fmt.Println("  hetu_luoshu route <task>     - 路由任务到最优模型")
		fmt.Println("  hetu_luoshu ask <task>       - 自动路由并调用LLM")
		fmt.Println("  hetu_luoshu call <model> <prompt> - 调用LLM")
		fmt.Println("  hetu_luoshu selfcheck        - 执行APEX自检")
		fmt.Println("  hetu_luoshu status           - 查看当前状态")
		os.Exit(1)
	}

	cmd := os.Args[1]

	switch cmd {
	case "route":
		if len(os.Args) < 3 {
			fmt.Println("用法: hetu_luoshu route <task>")
			os.Exit(1)
		}
		task := os.Args[2]
		model := RouteTask(task)
		fmt.Printf("任务: %s → 模型: %s\n", task, model)

	case "ask":
		if len(os.Args) < 3 {
			fmt.Println("用法: hetu_luoshu ask <task>")
			os.Exit(1)
		}
		task := os.Args[2]
		model := RouteTask(task)
		resp, usedModel, attempts, err := CallWithFallback(model, task)
		if err != nil {
			fmt.Fprintf(os.Stderr, "错误: %v\n", err)
			if len(attempts) > 0 {
				fmt.Fprintf(os.Stderr, "尝试记录: %s\n", strings.Join(attempts, " | "))
			}
			os.Exit(1)
		}
		fmt.Printf("路由模型: %s → 实际模型: %s\n", model, usedModel)
		if len(attempts) > 0 {
			fmt.Printf("兜底记录: %s\n", strings.Join(attempts, " | "))
		}
		if len(resp.Choices) > 0 {
			fmt.Printf("响应: %s\n", resp.Choices[0].Message.Content)
			fmt.Printf("Token: %d (prompt) + %d (completion) = %d total\n",
				resp.Usage.PromptTokens, resp.Usage.CompletionTokens, resp.Usage.TotalTokens)
		}

	case "call":
		if len(os.Args) < 4 {
			fmt.Println("用法: hetu_luoshu call <model> <prompt>")
			os.Exit(1)
		}
		model := os.Args[2]
		prompt := os.Args[3]

		resp, usedModel, attempts, err := CallWithFallback(model, prompt)

		if err != nil {
			fmt.Fprintf(os.Stderr, "错误: %v\n", err)
			if len(attempts) > 0 {
				fmt.Fprintf(os.Stderr, "尝试记录: %s\n", strings.Join(attempts, " | "))
			}
			os.Exit(1)
		}

		fmt.Printf("实际模型: %s\n", usedModel)
		if len(attempts) > 0 {
			fmt.Printf("兜底记录: %s\n", strings.Join(attempts, " | "))
		}
		if len(resp.Choices) > 0 {
			fmt.Printf("响应: %s\n", resp.Choices[0].Message.Content)
			fmt.Printf("Token: %d (prompt) + %d (completion) = %d total\n",
				resp.Usage.PromptTokens, resp.Usage.CompletionTokens, resp.Usage.TotalTokens)
		}

	case "selfcheck":
		result := PerformSelfCheck()
		data, _ := json.MarshalIndent(result, "", "  ")
		fmt.Println(string(data))

	case "status":
		selfCheck.mu.Lock()
		defer selfCheck.mu.Unlock()

		fmt.Printf("河图洛书 状态:\n")
		fmt.Printf("  总请求: %d\n", selfCheck.TotalRequests)
		fmt.Printf("  总Token: %d\n", selfCheck.TotalTokens)
		fmt.Printf("  失败次数: %d\n", selfCheck.FailCount)
		fmt.Printf("  APEX参数:\n")
		fmt.Printf("    Λ=%.2f Θ=%.2f K=%.2f\n", selfCheck.Lambda, selfCheck.Theta, selfCheck.Kappa)
		fmt.Printf("    ξ=%.2f Ψ=%.2f Φ=%.2f\n", selfCheck.Xi, selfCheck.Psi, selfCheck.Phi)
		fmt.Printf("    H=%.2f T=%.2f ε=%.2f\n", selfCheck.H, selfCheck.T, selfCheck.Epsilon)

		result := performSelfCheckUnsafe()
		fmt.Printf("  ΔG=%.4f 评级=%s\n", result.DeltaG, result.Grade)

	default:
		fmt.Fprintf(os.Stderr, "未知命令: %s\n", cmd)
		os.Exit(1)
	}
}
