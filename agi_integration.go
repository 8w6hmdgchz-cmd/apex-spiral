// agi_integration.go — AGI整合服务
// 整合所有7个服务形成完整的AGI闭环

package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

// ============ 服务客户端 ============

const (
	APEX_URL     = "http://localhost:8092"
	BIO_URL      = "http://localhost:8093"
	REFLECT_URL  = "http://localhost:8094"
	MODIFY_URL   = "http://localhost:8095"
	MEMORY_URL   = "http://localhost:8096"
	REASON_URL   = "http://localhost:8097"
	PLANNER_URL  = "http://localhost:8098"
)

// AGI整合请求
type AGIRequest struct {
	Query        string `json:"query"`         // 用户query
	EnableBio    bool   `json:"enable_bio"`    // 启用Bio Neuron
	EnableReflect bool  `json:"enable_reflect"` // 启用自我反思
	EnableMemory bool   `json:"enable_memory"` // 启用记忆
	EnableReason bool   `json:"enable_reason"`  // 启用推理链
	EnablePlan   bool   `json:"enable_plan"`    // 启用规划
}

// AGI整合响应
type AGIResponse struct {
	Result       *AGIResult   `json:"result"`
	MemoryUsed   []*MemoryUse `json:"memory_used,omitempty"`
	Reflection   string       `json:"reflection,omitempty"`
	Reasoning    string       `json:"reasoning,omitempty"`
	Plan         *PlanSummary `json:"plan,omitempty"`
	Insights     []string     `json:"insights"`
	DeltaG       float64      `json:"delta_g"`
}

type AGIResult struct {
	Gene        string `json:"selected_gene"`
	Strategy    string `json:"strategy"`
	ExecutedBy  string `json:"executed_by"` // apex/bio/both
}

type MemoryUse struct {
	ID      string `json:"id"`
	Content string `json:"content"`
}

type PlanSummary struct {
	Goal       string `json:"goal"`
	TaskCount  int    `json:"task_count"`
	Completed  int    `json:"completed"`
}

// ============ AGI整合引擎 ============

type AGIEngine struct {
	httpClient *http.Client
}

func NewAGIEngine() *AGIEngine {
	return &AGIEngine{
		httpClient: &http.Client{Timeout: 30 * time.Second},
	}
}

// Process 处理AGI请求 — 完整闭环
func (e *AGIEngine) Process(req *AGIRequest) *AGIResponse {
	resp := &AGIResponse{
		Insights: make([]string, 0),
	}

	// Step 1: 记忆检索
	if req.EnableMemory {
		memories := e.searchMemory(req.Query)
		if len(memories) > 0 {
			resp.MemoryUsed = memories
			resp.Insights = append(resp.Insights, fmt.Sprintf("从记忆库检索到%d条相关记忆", len(memories)))
		}
	}

	// Step 2: 自我反思 — 问自己"我知道什么"
	if req.EnableReflect {
		reflection := e.selfReflect(req.Query)
		resp.Reflection = reflection
		resp.Insights = append(resp.Insights, "完成自我反思")
	}

	// Step 3: 推理链 — 多步推理
	if req.EnableReason {
		reasoning := e.reason(req.Query)
		resp.Reasoning = reasoning
		resp.Insights = append(resp.Insights, "完成多步推理")
	}

	// Step 4: 规划 — 分解复杂任务
	if req.EnablePlan {
		plan := e.plan(req.Query)
		resp.Plan = plan
		resp.Insights = append(resp.Insights, fmt.Sprintf("生成%d个任务步骤", plan.TaskCount))
	}

	// Step 5: Bio Neuron 处理
	var bioResult *BioResult
	if req.EnableBio {
		bioResult = e.callBioNeuron(req.Query)
		resp.Insights = append(resp.Insights, fmt.Sprintf("Bio神经元激活: %s", bioResult.NeuronID))
	}

	// Step 6: APEX 基因选择
	apexResult := e.callAPEX(req.Query)
	resp.DeltaG = apexResult.DeltaG
	resp.Insights = append(resp.Insights, fmt.Sprintf("APEX选择: %s (ΔG=%.3f)", apexResult.Gene, apexResult.DeltaG))

	// Step 7: 综合决策
	finalResult := e.synthesize(apexResult, bioResult, resp)
	resp.Result = finalResult

	// Step 8: 记录经验到记忆
	if req.EnableMemory {
		e.storeMemory(req.Query, finalResult.Strategy, apexResult.DeltaG)
	}

	// Step 9: 自我修改 — 记录性能
	e.logPerformance(finalResult.Gene, apexResult.DeltaG, req.Query)

	return resp
}

// ============ 各服务调用 ============

func (e *AGIEngine) callAPEX(query string) *ApexResult {
	reqBody := map[string]interface{}{
		"query":    query,
		"use_bio":  false,
		"use_evm":  true,
	}

	data, _ := json.Marshal(reqBody)
	resp, err := e.httpClient.Post(APEX_URL+"/api/v1/gene/select", "application/json", bytes.NewBuffer(data))
	if err != nil {
		return &ApexResult{Gene: "default", DeltaG: 3.0}
	}
	defer resp.Body.Close()

	var result map[string]interface{}
	json.NewDecoder(resp.Body).Decode(&result)

	gene := "default"
	deltaG := 3.0

	if sg, ok := result["selected_gene"].(map[string]interface{}); ok {
		if n, ok := sg["name"].(string); ok {
			gene = n
		}
	}
	if dg, ok := result["delta_g"].(float64); ok {
		deltaG = dg
	}

	return &ApexResult{Gene: gene, DeltaG: deltaG}
}

type ApexResult struct {
	Gene  string
	DeltaG float64
}

func (e *AGIEngine) callBioNeuron(query string) *BioResult {
	reqBody := map[string]interface{}{
		"query":     query,
		"use_bio":   true,
	}

	data, _ := json.Marshal(reqBody)
	resp, err := e.httpClient.Post(BIO_URL+"/bio/select", "application/json", bytes.NewBuffer(data))
	if err != nil {
		return &BioResult{NeuronID: "none", Active: false}
	}
	defer resp.Body.Close()

	var result map[string]interface{}
	json.NewDecoder(resp.Body).Decode(&result)

	bio := result["bio_neuron"].(map[string]interface{})
	return &BioResult{
		NeuronID: bio["winner_id"].(string),
		Active:   true,
	}
}

type BioResult struct {
	NeuronID string
	Active   bool
}

func (e *AGIEngine) selfReflect(query string) string {
	reqBody := map[string]string{"question": fmt.Sprintf("关于'%s'，我的能力边界在哪里？", query)}

	data, _ := json.Marshal(reqBody)
	resp, err := e.httpClient.Post(REFLECT_URL+"/ask", "application/json", bytes.NewBuffer(data))
	if err != nil {
		return "反思服务不可用"
	}
	defer resp.Body.Close()

	var result map[string]interface{}
	json.NewDecoder(resp.Body).Decode(&result)

	if ans, ok := result["answer"].(string); ok {
		return ans
	}
	return "反思完成"
}

func (e *AGIEngine) searchMemory(query string) []*MemoryUse {
	resp, err := e.httpClient.Get(MEMORY_URL + "/search?q=" + query)
	if err != nil {
		return nil
	}
	defer resp.Body.Close()

	var result map[string]interface{}
	json.NewDecoder(resp.Body).Decode(&result)

	var memories []*MemoryUse
	if results, ok := result["results"].([]interface{}); ok {
		for _, r := range results {
			if m, ok := r.(map[string]interface{}); ok {
				memories = append(memories, &MemoryUse{
					ID:      m["id"].(string),
					Content: m["content"].(string),
				})
			}
		}
	}
	return memories
}

func (e *AGIEngine) reason(query string) string {
	reqBody := map[string]interface{}{
		"query":     query,
		"max_depth": 5,
	}

	data, _ := json.Marshal(reqBody)
	resp, err := e.httpClient.Post(REASON_URL+"/build", "application/json", bytes.NewBuffer(data))
	if err != nil {
		return "推理服务不可用"
	}
	defer resp.Body.Close()

	var result map[string]interface{}
	json.NewDecoder(resp.Body).Decode(&result)

	return fmt.Sprintf("推理链ID: %s, 置信度: %.2f",
		result["id"].(string), result["confidence"].(float64))
}

func (e *AGIEngine) plan(query string) *PlanSummary {
	reqBody := map[string]string{"goal": query}

	data, _ := json.Marshal(reqBody)
	resp, err := e.httpClient.Post(PLANNER_URL+"/decompose", "application/json", bytes.NewBuffer(data))
	if err != nil {
		return &PlanSummary{Goal: query, TaskCount: 0}
	}
	defer resp.Body.Close()

	var result map[string]interface{}
	json.NewDecoder(resp.Body).Decode(&result)

	tasks := result["tasks"].([]interface{})
	return &PlanSummary{
		Goal:      query,
		TaskCount: len(tasks),
		Completed: 0,
	}
}

func (e *AGIEngine) storeMemory(query, strategy string, deltaG float64) {
	reqBody := map[string]interface{}{
		"content":    fmt.Sprintf("Query: %s -> Strategy: %s (ΔG=%.3f)", query, strategy, deltaG),
		"tags":       []string{"experience", "strategy"},
		"importance": deltaG / 10.0,
	}

	data, _ := json.Marshal(reqBody)
	e.httpClient.Post(MEMORY_URL+"/add", "application/json", bytes.NewBuffer(data))
}

func (e *AGIEngine) logPerformance(geneID string, deltaG float64, query string) {
	reqBody := map[string]interface{}{
		"gene_id": geneID,
		"success": deltaG > 3.5,
		"delta_g": deltaG,
		"query":   query,
	}

	data, _ := json.Marshal(reqBody)
	e.httpClient.Post(MODIFY_URL+"/modify/log", "application/json", bytes.NewBuffer(data))
}

func (e *AGIEngine) synthesize(apex *ApexResult, bio *BioResult, resp *AGIResponse) *AGIResult {
	// 综合APEX和Bio的结果
	strategy := apex.Gene
	executedBy := "apex"

	if bio != nil && bio.Active {
		if apex.DeltaG < 4.0 {
			// 如果APEX置信度低，参考Bio
			strategy = fmt.Sprintf("%s + Bio:%s", apex.Gene, bio.NeuronID)
			executedBy = "both"
		}
	}

	// 参考记忆
	if len(resp.MemoryUsed) > 0 {
		strategy = fmt.Sprintf("%s (基于%d条记忆)", strategy, len(resp.MemoryUsed))
	}

	// 参考规划
	if resp.Plan != nil && resp.Plan.TaskCount > 0 {
		strategy = fmt.Sprintf("%s [计划:%d步]", strategy, resp.Plan.TaskCount)
	}

	return &AGIResult{
		Gene:       apex.Gene,
		Strategy:   strategy,
		ExecutedBy: executedBy,
	}
}

// ============ HTTP API ============

var agiEngine *AGIEngine

func init() {
	agiEngine = NewAGIEngine()
}

func agiHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req AGIRequest
	json.NewDecoder(r.Body).Decode(&req)

	// 设置默认值
	if req.Query == "" {
		req.Query = r.URL.Query().Get("query")
	}
	if !req.EnableMemory {
		req.EnableMemory = true
	}
	if !req.EnableReflect {
		req.EnableReflect = true
	}

	resp := agiEngine.Process(&req)
	json.NewEncoder(w).Encode(resp)
}

func agiHealthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	// 检查所有服务
	services := []struct {
		name string
		url  string
	}{
		{"APEX", APEX_URL + "/health"},
		{"Bio", BIO_URL + "/health"},
		{"Reflect", REFLECT_URL + "/health"},
		{"Modify", MODIFY_URL + "/health"},
		{"Memory", MEMORY_URL + "/health"},
		{"Reason", REASON_URL + "/health"},
		{"Planner", PLANNER_URL + "/health"},
	}

	status := make(map[string]string)
	allOk := true

	for _, s := range services {
		resp, err := http.Get(s.url)
		if err != nil || resp.StatusCode != 200 {
			status[s.name] = "down"
			allOk = false
		} else {
			status[s.name] = "ok"
		}
	}

	json.NewEncoder(w).Encode(map[string]interface{}{
		"status":   map[bool]string{true: "ok", false: "degraded"}[allOk],
		"services": status,
	})
}

func main() {
	mux := http.NewServeMux()
	mux.HandleFunc("/agi", agiHandler)
	mux.HandleFunc("/health", agiHealthHandler)

	fmt.Println("[AGI整合引擎] 服务启动在 :8099")
	fmt.Println("  /agi     - AGI完整处理")
	fmt.Println("  /health  - 服务状态")
	fmt.Println("")
	fmt.Println("整合服务:")
	fmt.Println("  :8092 APEX Gene Selector")
	fmt.Println("  :8093 Bio Neuron")
	fmt.Println("  :8094 Self Reflection")
	fmt.Println("  :8095 Self Modifier")
	fmt.Println("  :8096 Persistent Memory")
	fmt.Println("  :8097 Reasoning Chain")
	fmt.Println("  :8098 Planner")

	http.ListenAndServe(":8099", mux)
}
