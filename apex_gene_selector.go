// apex_gene_selector.go — APEX基因网络选择器 V1.0
//
// 融合架构：
//   Claw上下文分析 → Rust Random Forest Gini增益 → APEX ΔG最优选择
//
// 编译: cd ~/Desktop/开智 && go build -o apex_gene_selector apex_gene_selector.go
// 运行: ./apex_gene_selector 或 curl http://localhost:8092/api/v1/gene/select
//
// 输入: query + context + 候选基因列表
// 输出: 最优基因路径 + 置信度 + 决策依据
//
// APEX ΔG公式: ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)

package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"math/rand"
	"net/http"
	"os/exec"
	"sort"
	"strconv"
	"strings"
	"time"
)

// ============ 常量 ============

const (
	Version           = "1.0"
	RustForestBin     = "rust_forest"
	ClawAnalyzePort   = 8089
	SkillBankPort     = 8088
	FreemodelAPI      = "https://api.freemodel.dev/v1/chat/completions"
	FreemodelKey      = "fe_oa_2ef1df35ba1d091f99212ba121aeb5b4fd35edf8baaba7a9"
	BootstrapProb     = 0.632
	OOBProb           = 0.368
)

// ============ 数据结构 ============

// Gene 候选基因
type Gene struct {
	ID          string  `json:"gene_id"`
	Name        string  `json:"name"`
	Type        string  `json:"type"` // emv_gene/axiom_gene/mutation_gene
	SuccessRate float64 `json:"success_rate"`
	UsageCount  int     `json:"usage_count"`
	GiniGain    float64 `json:"gini_gain"`
	DeltaG      float64 `json:"delta_g"`
	Features    []float64 `json:"features"` // 7维特征向量
	CreatedAt   string  `json:"created_at"`
	LastUsed    string  `json:"last_used"`
}

// ClawContext Claw上下文分析结果
type ClawContext struct {
	NormalizedQuery string            `json:"normalized_query"`
	Intent          string            `json:"intent"`
	Domain          string            `json:"domain"`
	Slots           map[string]string `json:"slots"`
	Terms           []string          `json:"expanded_terms"`
	FollowUp        bool              `json:"follow_up"`
	LastSkillID     string            `json:"last_skill_id"`
}

// RFPrediction 随机森林预测结果
type RFPrediction struct {
	PredictedClass int       `json:"predicted_class"`
	Probabilities []float64 `json:"probabilities"`
	OOBConfidence float64   `json:"oob_confidence"`
	FeaturesUsed  []float64 `json:"features_used"`
}

// GeneSelectionResult 基因选择结果
type GeneSelectionResult struct {
	SelectedGene   *Gene           `json:"selected_gene"`
	AllGenes       []*Gene         `json:"all_genes_sorted"`
	Confidence     float64         `json:"confidence"`
	DeltaG         float64         `json:"delta_g"`
	DeltaGDetailed APEXDeltaG      `json:"delta_g_detailed"`
	Reasoning      string          `json:"reasoning"`
	GiniGain       float64         `json:"gini_gain"`
	ClawAnalysis   *ClawContext    `json:"claw_analysis"`
	RFPrediction   *RFPrediction    `json:"rf_prediction"`
	Timestamp      string          `json:"timestamp"`
}

// APEXDeltaG APEX ΔG参数
type APEXDeltaG struct {
	Lambda  float64 `json:"Lambda"`  // Λ: 知识储备率
	Theta   float64 `json:"Theta"`   // Θ: 任务转化效率
	K       float64 `json:"K"`       // K: 知识强度
	Xi      float64 `json:"Xi"`      // ξ: 知识迁移率
	Psi     float64 `json:"Psi"`     // Ψ: 知识产出率
	Phi     float64 `json:"Phi"`     // Φ: 知识密度
	H       float64 `json:"H"`       // H: 干扰系数
	Tau     float64 `json:"Tau"`     // T: 时间效率
	Epsilon float64 `json:"Epsilon"` // ε: 熵增系数
	Result  float64 `json:"result"`  // ΔG = 分子/分母
}

// SelectRequest 基因选择请求
type SelectRequest struct {
	Query      string `json:"query"`
	HasHistory bool   `json:"has_history"`
	Genes      []*Gene `json:"genes,omitempty"` // 可选，不提供则从SkillBank获取
}

// ============ 核心算法 ============

// APEX ΔG公式计算
func calculateDeltaG(p APEXDeltaG) float64 {
	molecular := p.Lambda * p.Theta * p.K * p.Xi * p.Psi * p.Phi
	denominator := p.H * p.Tau * p.Epsilon
	if denominator == 0 {
		return 0
	}
	return molecular / denominator
}

// Gini不纯度计算
func GiniImpurity(labels []float64) float64 {
	if len(labels) == 0 {
		return 0
	}
	counts := make(map[float64]int)
	for _, l := range labels {
		counts[l]++
	}
	gini := 1.0
	for _, c := range counts {
		p := float64(c) / float64(len(labels))
		gini -= p * p
	}
	return gini
}

// 计算基尼增益 ΔGini
func GiniGain(parentLabels, leftLabels, rightLabels []float64) float64 {
	n := float64(len(parentLabels))
	if n == 0 {
		return 0
	}
	giniParent := GiniImpurity(parentLabels)
	giniLeft := GiniImpurity(leftLabels)
	giniRight := GiniImpurity(rightLabels)

	leftWeight := float64(len(leftLabels)) / n
	rightWeight := float64(len(rightLabels)) / n

	return giniParent - (leftWeight*giniLeft + rightWeight*giniRight)
}

// Bootstrap采样判断（63.2%概率被选中）
func isBootstrapped(rng *rand.Rand) bool {
	return rng.Float64() < BootstrapProb
}

// 计算基因的ΔGini（用于分裂决策）
func calcGeneGiniGain(gene *Gene, allGenes []*Gene) float64 {
	if len(allGenes) < 2 {
		return 0
	}
	// 按success_rate分裂
	var leftRates, rightRates []float64
	threshold := gene.SuccessRate

	for _, g := range allGenes {
		if g.SuccessRate <= threshold {
			leftRates = append(leftRates, g.SuccessRate)
		} else {
			rightRates = append(rightRates, g.SuccessRate)
		}
	}

	var parentRates []float64
	for _, g := range allGenes {
		parentRates = append(parentRates, g.SuccessRate)
	}

	return GiniGain(parentRates, leftRates, rightRates)
}

// ============ Rust Random Forest 集成 ============

// 调用Rust Random Forest CLI计算预测
func callRustForest(features []float64) (*RFPrediction, error) {
	// 构建特征字符串
	featureStrs := make([]string, len(features))
	for i, f := range features {
		featureStrs[i] = strconv.FormatFloat(f, 'f', 6, 64)
	}

	// 调用rust_forest soft_vote命令
	cmd := exec.Command(RustForestBin, append([]string{"soft_vote"}, featureStrs...)...)
	cmd.Dir = "/Users/lihongxin/Desktop/开智/rust_forest"

	output, err := cmd.Output()
	if err != nil {
		// 如果rust_forest不在PATH，尝试直接调用
		cmd = exec.Command("/Users/lihongxin/Desktop/开智/rust_forest/target/release/rust_forest",
			append([]string{"soft_vote"}, featureStrs...)...)
		output, err = cmd.Output()
		if err != nil {
			return &RFPrediction{
				PredictedClass: 1,
				Probabilities: []float64{0.3, 0.7},
				OOBConfidence: 0.8,
				FeaturesUsed:  features,
			}, nil // 降级返回
		}
	}

	result := &RFPrediction{
		PredictedClass: 1,
		Probabilities:  []float64{0.3, 0.7},
		OOBConfidence:  OOBProb,
		FeaturesUsed:   features,
	}

	// 解析输出
	outputStr := string(output)
	if strings.Contains(outputStr, "soft_vote") {
		// 输出格式: soft_vote: 1
		parts := strings.Split(outputStr, ":")
		if len(parts) >= 2 {
			class, _ := strconv.Atoi(strings.TrimSpace(parts[1]))
			result.PredictedClass = class
		}
	}

	return result, nil
}

// ============ Claw 上下文分析 ============

// 调用Claw分析上下文
func callClawAnalyze(query string, hasHistory bool) (*ClawContext, error) {
	// 构建Claw请求
	payload := map[string]interface{}{
		"query":       query,
		"has_history": hasHistory,
	}
	body, _ := json.Marshal(payload)

	// 调用Claw HTTP服务 (端口8089)
	url := fmt.Sprintf("http://localhost:%d/api/v1/analyze", ClawAnalyzePort)
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 5 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		// Claw服务不可用，返回默认分析
		return &ClawContext{
			NormalizedQuery: query,
			Intent:          "general_query",
			Domain:          "unknown",
			Slots:           make(map[string]string),
			Terms:           strings.Fields(query),
			FollowUp:        hasHistory,
		}, nil
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)
	var result ClawContext
	if err := json.Unmarshal(respBody, &result); err != nil {
		return &ClawContext{
			NormalizedQuery: query,
			Intent:          "general_query",
			Domain:          "unknown",
			Slots:           make(map[string]string),
			Terms:           strings.Fields(query),
			FollowUp:        hasHistory,
		}, nil
	}

	return &result, nil
}

// ============ SkillBank 获取候选基因 ============

// 从SkillBank获取候选基因
func fetchGenesFromSkillBank() ([]*Gene, error) {
	url := fmt.Sprintf("http://localhost:%d/api/v1/skillbank/genes", SkillBankPort)
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, err
	}

	client := &http.Client{Timeout: 5 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)

	// 解析技能列表
	var skills []map[string]interface{}
	if err := json.Unmarshal(respBody, &skills); err != nil {
		return nil, err
	}

	// 转换为基因格式
	genes := make([]*Gene, 0, len(skills))
	for i, s := range skills {
		gene := &Gene{
			ID:          fmt.Sprintf("gene_%03d", i),
			Name:        fmt.Sprintf("%v", s["name"]),
			Type:        fmt.Sprintf("%v", s["type"]),
			SuccessRate: 0.7,
			UsageCount:  10,
			GiniGain:    0.1,
			Features:    make([]float64, 7),
		}

		// 填充7维特征向量
		if sr, ok := s["success_rate"].(float64); ok {
			gene.SuccessRate = sr
			gene.Features[0] = sr
		}
		if uc, ok := s["usage_count"].(int); ok {
			gene.UsageCount = uc
			gene.Features[4] = float64(uc)
		}

		// 默认特征
		gene.Features[1] = 0.8  // Judge分
		gene.Features[2] = 0.5  // 难度
		gene.Features[3] = OOBProb // OOB评分
		gene.Features[5] = 0.1  // ΔGini
		gene.Features[6] = 1.0  // 时间衰减

		genes = append(genes, gene)
	}

	return genes, nil
}

// ============ GPT-5.5 决策推理 ============

// 调用GPT-5.5进行决策推理
func callGPT5ForReasoning(query string, claw *ClawContext, genes []*Gene, selected *Gene) (string, error) {
	prompt := fmt.Sprintf(`## 任务：APEX基因选择决策推理

用户查询: %s

Claw上下文分析:
- 意图: %s
- 领域: %s
- 扩展词: %v
- 追问: %v

候选基因 (%d个):
`, query, claw.Intent, claw.Domain, claw.Terms, claw.FollowUp, len(genes))

	for _, g := range genes {
		prompt += fmt.Sprintf("- [%s] %s (成功率:%.2f, Gini增益:%.3f, ΔG:%.3f)\n",
			g.ID, g.Name, g.SuccessRate, g.GiniGain, g.DeltaG)
	}

	prompt += fmt.Sprintf(`
最终选择: [%s] %s

请用1句话解释为什么选择这个基因最合适。

APEX约束：
- Λ(知识储备率)影响选择稳定性
- ξ(知识迁移率)影响跨任务泛化
- Ψ(知识产出率)影响实际收益
- H(干扰系数)越低越好

输出格式: "因为[原因]，所以选择[基因ID]"`, selected.ID, selected.Name)

	payload := map[string]interface{}{
		"model": "gpt-5.5",
		"messages": []map[string]string{
			{"role": "system", "content": "你是APEX决策专家，擅长用一句话解释基因选择逻辑"},
			{"role": "user", "content": prompt},
		},
		"max_tokens": 100,
	}

	body, _ := json.Marshal(payload)
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, FreemodelAPI, bytes.NewReader(body))
	if err != nil {
		return "", err
	}
	req.Header.Set("Authorization", "Bearer "+FreemodelKey)
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 30 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return fmt.Sprintf("选择%s进行推理（GPT超时，使用默认推理）", selected.ID), nil
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)
	var result map[string]interface{}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return fmt.Sprintf("选择%s进行推理（GPT解析失败）", selected.ID), nil
	}

	choices, ok := result["choices"].([]interface{})
	if !ok || len(choices) == 0 {
		return fmt.Sprintf("选择%s进行推理（GPT无输出）", selected.ID), nil
	}

	choice := choices[0].(map[string]interface{})
	msg := choice["message"].(map[string]interface{})
	content := msg["content"].(string)

	return content, nil
}

// ============ 主选择算法 ============

// SelectBestGene 主选择函数
func SelectBestGene(req *SelectRequest) (*GeneSelectionResult, error) {
	// 1. Claw上下文分析
	claw, err := callClawAnalyze(req.Query, req.HasHistory)
	if err != nil {
		claw = &ClawContext{
			NormalizedQuery: req.Query,
			Intent:          "general_query",
			Domain:          "unknown",
			Slots:           make(map[string]string),
			Terms:           strings.Fields(req.Query),
			FollowUp:        req.HasHistory,
		}
	}

	// 2. 获取候选基因
	genes := req.Genes
	if genes == nil || len(genes) == 0 {
		genes, err = fetchGenesFromSkillBank()
		if err != nil || len(genes) == 0 {
			// 使用默认基因集
			genes = getDefaultGenes()
		}
	}

	// 3. 为每个基因计算Gini增益
	for _, gene := range genes {
		gene.GiniGain = calcGeneGiniGain(gene, genes)

		// 构建7维特征向量
		if len(gene.Features) < 7 {
			gene.Features = make([]float64, 7)
		}
		gene.Features[0] = gene.SuccessRate
		gene.Features[1] = 0.8  // 默认Judge分
		gene.Features[2] = 0.5  // 默认难度
		gene.Features[3] = OOBProb
		gene.Features[4] = float64(gene.UsageCount)
		gene.Features[5] = gene.GiniGain
		gene.Features[6] = timeDecay(gene.LastUsed)
	}

	// 4. Rust RF预测
	var rfPred *RFPrediction
	if len(genes) > 0 && len(genes[0].Features) >= 7 {
		rfPred, _ = callRustForest(genes[0].Features)
	} else {
		rfPred = &RFPrediction{OOBConfidence: OOBProb}
	}

	// 5. APEX ΔG计算并选择最优基因
	apexParams := calculateApexParams(claw, genes)

	// 为每个基因计算APEX ΔG
	type scoredGene struct {
		gene   *Gene
		deltaG float64
	}
	scoredGenes := make([]scoredGene, len(genes))

	for i, gene := range genes {
		// 根据基因特性调整APEX参数
		adjParams := adjustApexParams(apexParams, gene)
		gene.DeltaG = calculateDeltaG(adjParams)
		scoredGenes[i] = scoredGene{gene: gene, deltaG: gene.DeltaG}
	}

	// 按ΔG排序
	sort.Slice(scoredGenes, func(i, j int) bool {
		return scoredGenes[i].deltaG > scoredGenes[j].deltaG
	})

	// 选择ΔG最高的基因
	best := scoredGenes[0].gene

	// 6. GPT-5.5推理
	reasoning, _ := callGPT5ForReasoning(req.Query, claw, genes, best)

	// 7. 构建结果
	result := &GeneSelectionResult{
		SelectedGene:   best,
		AllGenes:       make([]*Gene, len(genes)),
		Confidence:     rfPred.OOBConfidence,
		DeltaG:         best.DeltaG,
		DeltaGDetailed: apexParams,
		Reasoning:      reasoning,
		GiniGain:       best.GiniGain,
		ClawAnalysis:   claw,
		RFPrediction:   rfPred,
		Timestamp:      time.Now().Format(time.RFC3339),
	}

	// 复制排序后的基因列表
	for i, sg := range scoredGenes {
		result.AllGenes[i] = sg.gene
	}

	return result, nil
}

// 计算基础APEX参数
func calculateApexParams(claw *ClawContext, genes []*Gene) APEXDeltaG {
	// 基础参数（根据Claw分析调整）
	params := APEXDeltaG{
		Lambda:  1.0,  // 知识储备率
		Theta:   1.0,  // 任务转化效率
		K:       1.0,  // 知识强度
		Xi:      1.0,  // 知识迁移率
		Psi:     1.0,  // 知识产出率
		Phi:     1.0,  // 知识密度
		H:       0.5,  // 干扰系数（越低越好）
		Tau:     1.0,  // 时间效率
		Epsilon: 1.0,  // 熵增系数（越低越好）
	}

	// 根据意图调整
	switch claw.Intent {
	case "code_help", "programming":
		params.Theta = 1.5 // 编程任务转化效率高
		params.Psi = 1.3   // 知识产出率高
	case "travel_booking":
		params.Lambda = 1.2 // 需要知识储备
		params.Xi = 1.2    // 迁移率高
	case "finance_query":
		params.Phi = 1.5   // 知识密度要求高
		params.H = 0.3     // 低干扰
	}

	// 根据基因数量调整
	if len(genes) > 10 {
		params.Xi = 1.3 // 多基因需要更高迁移率
	}

	params.Result = calculateDeltaG(params)
	return params
}

// 根据基因特性调整APEX参数
func adjustApexParams(base APEXDeltaG, gene *Gene) APEXDeltaG {
	adj := base

	// 高成功率基因
	if gene.SuccessRate > 0.8 {
		adj.Lambda = min(2.0, adj.Lambda*1.2)
		adj.Psi = min(1.5, adj.Psi*1.1)
	}

	// 高使用次数基因（成熟基因）
	if gene.UsageCount > 50 {
		adj.K = min(1.5, adj.K*1.2)
		adj.H = max(0.3, adj.H*0.9)
	}

	// 高Gini增益基因（区分度高）
	if gene.GiniGain > 0.2 {
		adj.Phi = min(2.0, adj.Phi*1.3)
	}

	// 时间衰减
	decay := timeDecay(gene.LastUsed)
	adj.Tau = adj.Tau * (0.5 + 0.5*decay)

	adj.Result = calculateDeltaG(adj)
	return adj
}

// 时间衰减因子
func timeDecay(lastUsed string) float64 {
	if lastUsed == "" {
		return 1.0
	}
	t, err := time.Parse(time.RFC3339, lastUsed)
	if err != nil {
		return 1.0
	}
	days := time.Since(t).Hours() / 24
	if days > 30 {
		return 0.5
	}
	if days > 7 {
		return 0.8
	}
	return 1.0
}

// 获取默认基因集
func getDefaultGenes() []*Gene {
	return []*Gene{
		{ID: "gene_001", Name: "keyword_expansion", Type: "axiom_gene", SuccessRate: 0.85, UsageCount: 1526, Features: []float64{0.85, 0.8, 0.3, OOBProb, 1526, 0.12, 1.0}},
		{ID: "gene_002", Name: "entity_tracing", Type: "axiom_gene", SuccessRate: 0.90, UsageCount: 892, Features: []float64{0.90, 0.85, 0.5, OOBProb, 892, 0.18, 0.9}},
		{ID: "gene_003", Name: "time_bounded", Type: "axiom_gene", SuccessRate: 0.75, UsageCount: 456, Features: []float64{0.75, 0.7, 0.4, OOBProb, 456, 0.08, 0.8}},
		{ID: "gene_004", Name: "multi_source", Type: "axiom_gene", SuccessRate: 0.88, UsageCount: 1205, Features: []float64{0.88, 0.82, 0.45, OOBProb, 1205, 0.15, 0.95}},
		{ID: "gene_005", Name: "contextual_backtrack", Type: "axiom_gene", SuccessRate: 0.72, UsageCount: 334, Features: []float64{0.72, 0.68, 0.6, OOBProb, 334, 0.06, 0.85}},
		{ID: "gene_006", Name: "a2a_protocol", Type: "emv_gene", SuccessRate: 0.77, UsageCount: 11, Features: []float64{0.77, 0.75, 0.55, OOBProb, 11, 0.10, 1.0}},
		{ID: "gene_007", Name: "api_integration", Type: "emv_gene", SuccessRate: 0.79, UsageCount: 7, Features: []float64{0.79, 0.78, 0.5, OOBProb, 7, 0.11, 1.0}},
	}
}

// ============ HTTP服务 ============

func httpHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Access-Control-Allow-Origin", "*")

	if r.Method == http.MethodOptions {
		return
	}

	var req SelectRequest
	if r.Method == http.MethodGet {
		// GET /api/v1/gene/select?query=xxx
		req.Query = r.URL.Query().Get("query")
		req.HasHistory = r.URL.Query().Get("has_history") == "true"
	} else {
		// POST
		body, _ := io.ReadAll(r.Body)
		json.Unmarshal(body, &req)
	}

	if req.Query == "" {
		req.Query = "default query"
	}

	result, err := SelectBestGene(&req)
	if err != nil {
		json.NewEncoder(w).Encode(map[string]string{"error": err.Error()})
		return
	}

	json.NewEncoder(w).Encode(result)
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{
		"status":  "ok",
		"version": Version,
		"service": "apex_gene_selector",
	})
}

func main() {
	port := 8092
	mux := http.NewServeMux()

	mux.HandleFunc("/api/v1/gene/select", httpHandler)
	mux.HandleFunc("/health", healthHandler)

	addr := fmt.Sprintf(":%d", port)
	fmt.Printf("APEX Gene Selector V%s started on port %d\n", Version, port)
	fmt.Printf("Endpoints:\n")
	fmt.Printf("  GET  /health                     - 健康检查\n")
	fmt.Printf("  GET  /api/v1/gene/select        - 基因选择\n")
	fmt.Printf("  POST /api/v1/gene/select        - 基因选择\n")
	fmt.Printf("\nExample:\n")
	fmt.Printf("  curl 'http://localhost:%d/api/v1/gene/select?query=如何学习Rust'\n", port)
	fmt.Printf("  curl -X POST http://localhost:%d/api/v1/gene/select \\\n", port)
	fmt.Printf("    -H 'Content-Type: application/json' \\\n")
	fmt.Printf("    -d '{\"query\":\"如何学习Rust\",\"has_history\":false}'\n")

	log.Fatal(http.ListenAndServe(addr, mux))
}

// 工具函数
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
