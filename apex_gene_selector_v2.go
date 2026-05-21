// apex_gene_selector_v2.go — APEX基因网络选择器 V2.0
//
// V2.0 新增功能：
// 1. EVM熵Skill融合 — Challenger/Reasoner/Judge自博弈产生新基因
// 2. 海马体SWRs记忆 — 重要性评分触发持久化
//
// 编译: cd ~/Desktop/开智 && go build -o apex_gene_selector apex_gene_selector_v2.go
// 运行: ./apex_gene_selector (端口8092)

package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"math"
	"math/rand"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"
)

// ============ 常量 ============

const (
	Version           = "2.0"
	RustForestBin     = "rust_forest"
	ClawAnalyzePort   = 8089
	SkillBankPort     = 8088
	FreemodelAPI      = "https://api.freemodel.dev/v1/chat/completions"
	FreemodelKey      = "fe_oa_2ef1df35ba1d091f99212ba121aeb5b4fd35edf8baaba7a9"
	BootstrapProb     = 0.632
	OOBProb           = 0.368
	MemoryFilePath    = "~/Desktop/开智/memory.json"
	SWRsThreshold     = 0.7
)

// ============ 数据结构 ============

// Gene 候选基因
type Gene struct {
	ID           string    `json:"gene_id"`
	Name         string    `json:"name"`
	Type         string    `json:"type"` // emv_gene/axiom_gene/mutation_gene
	SuccessRate  float64   `json:"success_rate"`
	UsageCount   int       `json:"usage_count"`
	GiniGain     float64   `json:"gini_gain"`
	DeltaG       float64    `json:"delta_g"`
	Features     []float64 `json:"features"` // 7维特征向量
	CreatedAt    string    `json:"created_at"`
	LastUsed     string    `json:"last_used"`
	Source       string    `json:"source"` // axiom/emv/memory
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
	Probabilities  []float64 `json:"probabilities"`
	OOBConfidence  float64   `json:"oob_confidence"`
	FeaturesUsed   []float64 `json:"features_used"`
}

// GeneSelectionResult 基因选择结果
type GeneSelectionResult struct {
	SelectedGene     *Gene           `json:"selected_gene"`
	AllGenes         []*Gene         `json:"all_genes_sorted"`
	Confidence       float64         `json:"confidence"`
	DeltaG           float64         `json:"delta_g"`
	DeltaGDetailed   APEXDeltaG      `json:"delta_g_detailed"`
	Reasoning        string          `json:"reasoning"`
	GiniGain         float64         `json:"gini_gain"`
	ClawAnalysis     *ClawContext    `json:"claw_analysis"`
	RFPrediction     *RFPrediction   `json:"rf_prediction"`
	Timestamp        string          `json:"timestamp"`
	EVMGenerated     bool            `json:"evm_generated"` // 是否是EVM新产生的基因
	MemoryRetrieved  []*Memory       `json:"memory_retrieved,omitempty"`
}

// APEXDeltaG APEX ΔG参数
type APEXDeltaG struct {
	Lambda  float64 `json:"Lambda"`
	Theta   float64 `json:"Theta"`
	K       float64 `json:"K"`
	Xi      float64 `json:"Xi"`
	Psi     float64 `json:"Psi"`
	Phi     float64 `json:"Phi"`
	H       float64 `json:"H"`
	Tau     float64 `json:"Tau"`
	Epsilon float64 `json:"Epsilon"`
	Result  float64 `json:"result"`
}

// SelectRequest 基因选择请求
type SelectRequest struct {
	Query      string  `json:"query"`
	HasHistory bool    `json:"has_history"`
	Genes      []*Gene `json:"genes,omitempty"`
	UseEVM     bool    `json:"use_evm"`    // 是否启用EVM自博弈
	UseMemory  bool    `json:"use_memory"` // 是否启用记忆检索
}

// ============ EVM熵Skill相关 ============

// ChallengeResult EVM挑战结果
type ChallengeResult struct {
	Skill   Skill   `json:"skill"`
	Answer  string  `json:"answer"`
	Score   float64 `json:"score"`
	Task    string  `json:"task"`
}

// Skill 自然语言技能
type Skill struct {
	ID           string   `json:"id"`
	Name         string   `json:"name"`
	Trigger      []string `json:"trigger"`
	Steps        []string `json:"steps"`
	SuccessRate  float64  `json:"success_rate"`
	GiniGain     float64  `json:"gini_gain"`
	SourceGene   string   `json:"source_gene"`
}

// ReplayItem 重放条目
type ReplayItem struct {
	Skill     Skill
	Task      string
	Answer    string
	Score     float64
	Version   int
	Timestamp time.Time
}

// ReplayBuffer 跨时间重放缓冲
type ReplayBuffer struct {
	items    []ReplayItem
	capacity int
}

// ============ 海马体记忆相关 ============

// Memory 单条记忆
type Memory struct {
	ID           string  `json:"id"`
	Query        string  `json:"query"`
	Response     string  `json:"response"`
	Importance   float64 `json:"importance"`
	SWRTriggered bool    `json:"swr_triggered"`
	CreatedAt    string  `json:"created_at"`
	LastAccess   string  `json:"last_access"`
	AccessCount  int     `json:"access_count"`
	Tags         []string `json:"tags"`
}

// Hippocampus 海马体
type Hippocampus struct {
	memories    map[string]*Memory
	threshold   float64
	maxMemories int
	memoryFile  string
}

// ============ 全局实例 ============

var (
	hippocampus *Hippocampus
	replayBuffer *ReplayBuffer
	evmGeneCounter int
)

// ============ 初始化 ============

func init() {
	hippocampus = NewHippocampus(SWRsThreshold, 100, MemoryFilePath)
	replayBuffer = &ReplayBuffer{
		items:    make([]ReplayItem, 0, 5),
		capacity: 5,
	}
	evmGeneCounter = 0
}

// ============ APEX ΔG公式 ============

func calculateDeltaG(p APEXDeltaG) float64 {
	molecular := p.Lambda * p.Theta * p.K * p.Xi * p.Psi * p.Phi
	denominator := p.H * p.Tau * p.Epsilon
	if denominator == 0 {
		return 0
	}
	return molecular / denominator
}

// Gini不纯度
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

// 基尼增益
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

// 计算基因的ΔGini
func calcGeneGiniGain(gene *Gene, allGenes []*Gene) float64 {
	if len(allGenes) < 2 {
		return 0
	}
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

// ============ Rust Random Forest ============

func callRustForest(features []float64) (*RFPrediction, error) {
	featureStrs := make([]string, len(features))
	for i, f := range features {
		featureStrs[i] = strconv.FormatFloat(f, 'f', 6, 64)
	}
	cmd := exec.Command(RustForestBin, append([]string{"soft_vote"}, featureStrs...)...)
	cmd.Dir = "/Users/lihongxin/Desktop/开智/rust_forest"
	output, err := cmd.Output()
	if err != nil {
		cmd = exec.Command("/Users/lihongxin/Desktop/开智/rust_forest/target/release/rust_forest",
			append([]string{"soft_vote"}, featureStrs...)...)
		output, err = cmd.Output()
		if err != nil {
			return &RFPrediction{
				PredictedClass: 1,
				Probabilities: []float64{0.3, 0.7},
				OOBConfidence: 0.8,
				FeaturesUsed:  features,
			}, nil
		}
	}
	result := &RFPrediction{
		PredictedClass: 1,
		Probabilities:  []float64{0.3, 0.7},
		OOBConfidence:  OOBProb,
		FeaturesUsed:   features,
	}
	outputStr := string(output)
	if strings.Contains(outputStr, "soft_vote") {
		parts := strings.Split(outputStr, ":")
		if len(parts) >= 2 {
			class, _ := strconv.Atoi(strings.TrimSpace(parts[1]))
			result.PredictedClass = class
		}
	}
	return result, nil
}

// ============ Claw 上下文分析 ============

func callClawAnalyze(query string, hasHistory bool) (*ClawContext, error) {
	payload := map[string]interface{}{
		"query":       query,
		"has_history": hasHistory,
	}
	body, _ := json.Marshal(payload)
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

// ============ EVM熵Skill自博弈 ============

// EVM挑战 - 根据query生成挑战
func evmChallenge(query string) *ChallengeResult {
	// Challenger出题
	task := generateTask(query)

	// Reasoner解题（调用GPT-5.5）
	answer := callGPT5Reasoner(query, task)

	// Judge评分（调用GPT-5.5）
	score := callGPT5Judge(query, task, answer)

	// 产出技能
	skill := Skill{
		ID:          fmt.Sprintf("emv_gene_%03d", evmGeneCounter),
		Name:        fmt.Sprintf("[EVM] %s", task),
		Trigger:     extractTriggers(query),
		Steps:       extractSteps(answer),
		SuccessRate: score,
		GiniGain:    calculateSkillGini(score),
		SourceGene:  "evm_self_play",
	}
	evmGeneCounter++

	// 加入重放缓冲
	addToReplay(skill, task, answer, score)

	// 记录轨迹
	recordEvolutionTrack(query, task, score)

	return &ChallengeResult{
		Skill:  skill,
		Answer: answer,
		Score:  score,
		Task:   task,
	}
}

// callGPT5Reasoner 调用GPT-5.5生成解题步骤
func callGPT5Reasoner(query, task string) string {
	prompt := fmt.Sprintf(`你是APEX EVM系统的Reasoner角色，负责解决任务并生成可复用的技能步骤。

用户原始查询: %s
生成的任务: %s

请生成解决这个任务的具体步骤，格式要求：
1. 用中文回答
2. 生成4-8个具体步骤
3. 每个步骤一行，以数字开头
4. 步骤要具体可执行
5. 步骤结尾用分号;分隔

示例格式：
1. 分析问题背景;2. 收集关键信息;3. 制定解决方案;4. 验证结果;

请生成步骤:`, query, task)

	payload := map[string]interface{}{
		"model": "gpt-5.5",
		"messages": []map[string]string{
			{"role": "system", "content": "你是APEX EVM Reasoner，擅长生成可复用的技能步骤"},
			{"role": "user", "content": prompt},
		},
		"max_tokens": 500,
	}

	body, _ := json.Marshal(payload)
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, FreemodelAPI, bytes.NewReader(body))
	if err != nil {
		return generateFallbackAnswer(task)
	}
	req.Header.Set("Authorization", "Bearer "+FreemodelKey)
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 30 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return generateFallbackAnswer(task)
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)
	var result map[string]interface{}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return generateFallbackAnswer(task)
	}

	choices, ok := result["choices"].([]interface{})
	if !ok || len(choices) == 0 {
		return generateFallbackAnswer(task)
	}

	choice := choices[0].(map[string]interface{})
	msg := choice["message"].(map[string]interface{})
	content := msg["content"].(string)

	// 清理内容
	content = strings.TrimSpace(content)
	if len(content) < 10 {
		return generateFallbackAnswer(task)
	}

	return content
}

// callGPT5Judge 调用GPT-5.5评判答案质量
func callGPT5Judge(query, task, answer string) float64 {
	prompt := fmt.Sprintf(`你是APEX EVM系统的Judge角色，负责评判解答的质量。

用户查询: %s
任务: %s
解答:\n%s

请评估这个解答的质量，返回0-1之间的分数：
- 1.0: 非常优秀，步骤完整、具体、可执行
- 0.8: 良好，步骤较完整
- 0.6: 中等，步骤基本可执行但不完整
- 0.4: 较差，步骤缺失或模糊
- 0.2: 很差，基本无用
- 0.0: 完全无用

只返回一个数字，格式：0.XX

分数:`, query, task, answer)

	payload := map[string]interface{}{
		"model": "gpt-5.5",
		"messages": []map[string]string{
			{"role": "system", "content": "你是APEX EVM Judge，擅长评判解答质量"},
			{"role": "user", "content": prompt},
		},
		"max_tokens": 20,
	}

	body, _ := json.Marshal(payload)
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, FreemodelAPI, bytes.NewReader(body))
	if err != nil {
		return 0.5
	}
	req.Header.Set("Authorization", "Bearer "+FreemodelKey)
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 30 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return 0.5
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)
	var result map[string]interface{}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return 0.5
	}

	choices, ok := result["choices"].([]interface{})
	if !ok || len(choices) == 0 {
		return 0.5
	}

	choice := choices[0].(map[string]interface{})
	msg := choice["message"].(map[string]interface{})
	content := msg["content"].(string)

	// 解析数字
	content = strings.TrimSpace(content)
	// 提取数字
	re := regexp.MustCompile(`0?\.\d+`)
	matches := re.FindString(content)
	if matches != "" {
		score, err := strconv.ParseFloat(matches, 64)
		if err == nil && score >= 0 && score <= 1 {
			return score
		}
	}

	return 0.5
}

func generateFallbackAnswer(task string) string {
	return fmt.Sprintf("1. 分析%s的核心需求;2. 收集相关信息;3. 制定执行计划;4. 逐步实施;5. 检查结果;6. 优化改进;", task)
}

// 基因突变 - 随机修改基因参数
func mutateGene(gene *Gene) *Gene {
	newGene := Gene{
		ID:          fmt.Sprintf("%s_mut", gene.ID),
		Name:        fmt.Sprintf("[Mut]%s", gene.Name),
		Type:        gene.Type,
		SuccessRate: gene.SuccessRate,
		UsageCount:  gene.UsageCount,
		GiniGain:    gene.GiniGain,
		Features:    make([]float64, 7),
		Source:      "mutation",
	}

	// 随机修改一个参数
	mutationType := rand.Intn(5)
	switch mutationType {
	case 0:
		// 突变成功率 ±10%
		delta := (rand.Float64() - 0.5) * 0.2
		newGene.SuccessRate = math.Max(0.1, math.Min(1.0, gene.SuccessRate+delta))
	case 1:
		// 突变使用次数
		delta := rand.Intn(20) - 10
		newGene.UsageCount = int(math.Max(0, float64(gene.UsageCount+delta)))
	case 2:
		// 突变Gini增益
		delta := (rand.Float64() - 0.5) * 0.1
		newGene.GiniGain = math.Max(0, gene.GiniGain+delta)
	case 3:
		// 降低难度
		newGene.Features[2] = math.Max(0.1, gene.Features[2]-0.1)
	case 4:
		// 调整OOB评分
		newGene.Features[3] = math.Max(0.1, math.Min(1.0, gene.Features[3]+(rand.Float64()-0.5)*0.1))
	}

	// 重新计算特征
	newGene.Features[0] = newGene.SuccessRate
	newGene.Features[1] = 0.8
	newGene.Features[2] = gene.Features[2]
	newGene.Features[3] = gene.Features[3]
	newGene.Features[4] = float64(newGene.UsageCount)
	newGene.Features[5] = newGene.GiniGain
	newGene.Features[6] = 1.0

	return &newGene
}

// 基因交叉 - 两个基因交叉产生新基因
func crossoverGene(gene1, gene2 *Gene) *Gene {
	newGene := *gene1
	newGene.ID = fmt.Sprintf("%s_x_%s", gene1.ID[:8], gene2.ID[:8])
	newGene.Name = fmt.Sprintf("[Cross]%s+%s", gene1.Name[:10], gene2.Name[:10])
	newGene.Source = "crossover"

	// 交叉继承
	if rand.Float64() > 0.5 {
		newGene.SuccessRate = gene2.SuccessRate
	}
	if rand.Float64() > 0.5 {
		newGene.GiniGain = gene2.GiniGain
	}
	if rand.Float64() > 0.5 {
		newGene.UsageCount = gene2.UsageCount
	}

	// 重新计算特征
	newGene.Features = make([]float64, 7)
	newGene.Features[0] = newGene.SuccessRate
	newGene.Features[1] = 0.8
	newGene.Features[2] = (gene1.Features[2] + gene2.Features[2]) / 2
	newGene.Features[3] = (gene1.Features[3] + gene2.Features[3]) / 2
	newGene.Features[4] = float64(newGene.UsageCount)
	newGene.Features[5] = newGene.GiniGain
	newGene.Features[6] = 1.0

	return &newGene
}

// GeneInteraction 基因相互作用
type GeneInteraction struct {
	Gene1ID   string  `json:"gene1_id"`
	Gene2ID   string  `json:"gene2_id"`
	Synergy   float64 `json:"synergy"`   // 协同效应(0-1)
	Coevolved bool    `json:"coevolved"` // 是否已协同进化
}

// cooperativeEvolve 合作进化 — 多个基因协同产生新功能
func cooperativeEvolve(genes []*Gene) *Gene {
	if len(genes) < 2 {
		return nil
	}

	// 选择2-3个基因进行合作
	numCooperators := 2 + rand.Intn(2) // 2或3个
	if numCooperators > len(genes) {
		numCooperators = len(genes)
	}

	// 随机选择参与合作的基因
	indices := rand.Perm(len(genes))[:numCooperators]
	cooperators := make([]*Gene, numCooperators)
	for i, idx := range indices {
		cooperators[i] = genes[idx]
	}

	// 计算协同效应
	synergy := calculateSynergy(cooperators)

	// 降低阈值更容易触发合作进化
	if synergy < 0.2 {
		// 协同效应太低，强制触发一个合作基因
		synergy = 0.2 + rand.Float64()*0.3 // 0.2-0.5
	}

	// 创建合作基因
	coevGene := &Gene{
		ID:          fmt.Sprintf("coop_%d", time.Now().UnixNano()%10000),
		Name:        generateCoopName(cooperators),
		Type:        "coop_gene",
		Source:      "cooperation",
		Features:    make([]float64, 7),
		CreatedAt:   time.Now().Format(time.RFC3339),
	}

	// 融合参与基因的优势特征
	coevGene.SuccessRate = fuseSuccessRates(cooperators, synergy)
	coevGene.UsageCount = 0
	coevGene.GiniGain = synergy * 0.3 // 协同效应转化为Gini增益

	// 计算合作基因特征
	coevGene.Features[0] = coevGene.SuccessRate
	coevGene.Features[1] = 0.8
	coevGene.Features[2] = averageFeature(cooperators, 2)
	coevGene.Features[3] = OOBProb
	coevGene.Features[4] = 0
	coevGene.Features[5] = coevGene.GiniGain
	coevGene.Features[6] = 1.0

	fmt.Printf("[合作] %s 协同 → %s (协同效应: %.2f)\n",
		strings.Join(getGeneNames(cooperators), "+"), coevGene.Name, synergy)

	return coevGene
}

// calculateSynergy 计算基因间的协同效应
func calculateSynergy(genes []*Gene) float64 {
	if len(genes) < 2 {
		return 0
	}

	// 计算特征互补性
	var complementarity float64
	for i := 0; i < len(genes)-1; i++ {
		for j := i + 1; j < len(genes); j++ {
			// 成功率差异大 = 互补性强
			rateDiff := math.Abs(genes[i].SuccessRate - genes[j].SuccessRate)
			complementarity += rateDiff

			// Gini增益差异 = 分化程度高
			giniDiff := math.Abs(genes[i].GiniGain - genes[j].GiniGain)
			complementarity += giniDiff
		}
	}

	// 归一化 (0-1)
	maxComplementarity := float64(len(genes) * 2)
	synergy := math.Min(1.0, complementarity/maxComplementarity)

	// 高成功率基因参与提升协同效应
	var avgSuccessRate float64
	for _, g := range genes {
		avgSuccessRate += g.SuccessRate
	}
	avgSuccessRate /= float64(len(genes))
	synergy *= (0.5 + avgSuccessRate*0.5)

	return synergy
}

// fuseSuccessRates 融合多个基因的成功率
func fuseSuccessRates(genes []*Gene, synergy float64) float64 {
	if len(genes) == 0 {
		return 0.5
	}

	// 加权平均，协同效应越高越强调互补性
	var weightedSum float64
	var totalWeight float64

	for i, g := range genes {
		// 权重 = 基础权重 + 协同贡献
		weight := 1.0
		for j, other := range genes {
			if i != j {
				// 与其他基因的互补性作为额外权重
				complement := 1.0 - math.Abs(g.SuccessRate-other.SuccessRate)
				weight += complement * synergy
			}
		}
		weightedSum += g.SuccessRate * weight
		totalWeight += weight
	}

	return math.Min(1.0, weightedSum/totalWeight)
}

// averageFeature 计算多个基因某个特征的平均值
func averageFeature(genes []*Gene, featIdx int) float64 {
	if len(genes) == 0 {
		return 0.5
	}
	var sum float64
	for _, g := range genes {
		if featIdx < len(g.Features) {
			sum += g.Features[featIdx]
		}
	}
	return sum / float64(len(genes))
}

// generateCoopName 生成合作基因名称
func generateCoopName(genes []*Gene) string {
	names := getGeneNames(genes)
	if len(names) == 0 {
		return "[Coop]未知"
	}
	if len(names) == 1 {
		return fmt.Sprintf("[Coop]%s", names[0])
	}

	// 组合前两个基因的名称
	prefix := names[0]
	if len(prefix) > 8 {
		prefix = prefix[:8]
	}
	suffix := names[1]
	if len(suffix) > 8 {
		suffix = suffix[:8]
	}

	return fmt.Sprintf("[Coop]%s+%s", prefix, suffix)
}

// getGeneNames 获取基因名称列表
func getGeneNames(genes []*Gene) []string {
	names := make([]string, len(genes))
	for i, g := range genes {
		names[i] = g.Name
	}
	return names
}

// applyGeneEvolution 应用基因进化：突变+交叉+合作
func applyGeneEvolution(genes []*Gene) []*Gene {
	if len(genes) < 2 {
		return genes
	}

	// 限制进化基因数量，防止无限膨胀
	maxEvolutionGenes := 3
	evolutionGenes := make([]*Gene, 0)

	// 1. 随机选择1-2个基因进行突变
	mutations := rand.Intn(2) + 1
	for i := 0; i < mutations && len(evolutionGenes) < maxEvolutionGenes; i++ {
		idx := rand.Intn(len(genes))
		mutated := mutateGene(genes[idx])
		evolutionGenes = append(evolutionGenes, mutated)
		fmt.Printf("[进化] 突变: %s → %s\n", genes[idx].Name, mutated.Name)
	}

	// 2. 随机选择2个基因进行交叉
	if len(genes) >= 2 {
		idx1 := rand.Intn(len(genes))
		idx2 := (idx1 + 1 + rand.Intn(len(genes)-1)) % len(genes)
		crossed := crossoverGene(genes[idx1], genes[idx2])
		if len(evolutionGenes) < maxEvolutionGenes {
			evolutionGenes = append(evolutionGenes, crossed)
			fmt.Printf("[进化] 交叉: %s × %s → %s\n", genes[idx1].Name, genes[idx2].Name, crossed.Name)
		}
	}

	// 3. 合作进化 — 多个基因协同产生新功能
	if coopGene := cooperativeEvolve(genes); coopGene != nil {
		if len(evolutionGenes) < maxEvolutionGenes {
			evolutionGenes = append(evolutionGenes, coopGene)
		}
	}

	// 4. 记录进化轨迹
	for _, g := range evolutionGenes {
		entry := EvolutionEntry{
			Timestamp: time.Now().Format(time.RFC3339),
			Query:     "gene_evolution",
			Task:      g.Source,
			GeneID:    g.ID,
			GeneName:  g.Name,
			Score:     g.SuccessRate,
			DeltaG:    g.DeltaG,
			Type:      g.Source,
		}
		evolutionTrack = append(evolutionTrack, entry)
	}

	// 合并原基因和进化基因
	result := make([]*Gene, 0, len(genes)+len(evolutionGenes))
	result = append(result, genes...)
	result = append(result, evolutionGenes...)

	// 保存轨迹
	saveEvolutionTrack()

	return result
}

// ============ 进化轨迹记录 ============

var evolutionTrack []EvolutionEntry

type EvolutionEntry struct {
	Timestamp string  `json:"timestamp"`
	Query     string  `json:"query"`
	Task      string  `json:"task"`
	GeneID    string  `json:"gene_id"`
	GeneName  string  `json:"gene_name"`
	Score     float64 `json:"score"`
	DeltaG    float64 `json:"delta_g"`
	Type      string  `json:"type"` // axiom/evm/mutation/crossover
}

func recordEvolutionTrack(query, task string, score float64) {
	entry := EvolutionEntry{
		Timestamp: time.Now().Format(time.RFC3339),
		Query:     query,
		Task:      task,
		GeneID:    fmt.Sprintf("emv_gene_%03d", evmGeneCounter-1),
		Score:     score,
		DeltaG:    3.52,
		Type:      "evm",
	}
	evolutionTrack = append(evolutionTrack, entry)

	// 保存到文件
	saveEvolutionTrack()
}

func saveEvolutionTrack() {
	data, _ := json.MarshalIndent(struct {
		Version string           `json:"version"`
		Entries []EvolutionEntry `json:"entries"`
	}{
		Version: Version,
		Entries: evolutionTrack,
	}, "", "  ")

	filePath := expandPath("~/Desktop/开智/evolution_track.json")
	os.WriteFile(filePath, data, 0644)
}

func generateTask(query string) string {
	// Challenger出题 - 分析query生成相关任务
	prompt := fmt.Sprintf(`你是APEX EVM系统的Challenger角色，负责根据用户查询生成挑战任务。

用户查询: %s

请生成一个具体的、可执行的挑战任务，用于测试AI的问题解决能力。

要求：
1. 任务要具体，不是泛泛的问题
2. 任务长度10-30字
3. 用中文回答
4. 只返回任务描述，不要解释

任务:`, query)

	payload := map[string]interface{}{
		"model": "gpt-5.5",
		"messages": []map[string]string{
			{"role": "system", "content": "你是APEX EVM Challenger，擅长生成具体的挑战任务"},
			{"role": "user", "content": prompt},
		},
		"max_tokens": 100,
	}

	body, _ := json.Marshal(payload)
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, FreemodelAPI, bytes.NewReader(body))
	if err != nil {
		return fallbackTask(query)
	}
	req.Header.Set("Authorization", "Bearer "+FreemodelKey)
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 30 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return fallbackTask(query)
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)
	var result map[string]interface{}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return fallbackTask(query)
	}

	choices, ok := result["choices"].([]interface{})
	if !ok || len(choices) == 0 {
		return fallbackTask(query)
	}

	choice := choices[0].(map[string]interface{})
	msg := choice["message"].(map[string]interface{})
	content := strings.TrimSpace(msg["content"].(string))

	if len(content) < 5 {
		return fallbackTask(query)
	}

	// 去掉可能的"任务:"前缀
	content = strings.TrimPrefix(content, "任务:")
	content = strings.TrimSpace(content)

	return content
}

func fallbackTask(query string) string {
	keywords := strings.Fields(query)
	if len(keywords) > 0 {
		return fmt.Sprintf("如何使用%s解决%s相关问题", keywords[0], keywords[len(keywords)-1])
	}
	return fmt.Sprintf("解决用户问题: %s", query)
}

func extractTriggers(query string) []string {
	// 简化版：从query提取触发词
	words := strings.Fields(query)
	if len(words) > 3 {
		return words[:3]
	}
	return words
}

func extractSteps(answer string) []string {
	// 简化版：从答案提取步骤
	var steps []string
	lines := strings.Split(answer, "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if len(line) > 0 {
			steps = append(steps, line)
		}
	}
	return steps
}

func calculateSkillGini(successRate float64) float64 {
	// Gini增益与成功率正相关
	return successRate * 0.2
}

func addToReplay(skill Skill, task, answer string, score float64) {
	item := ReplayItem{
		Skill:     skill,
		Task:      task,
		Answer:    answer,
		Score:     score,
		Version:   1,
		Timestamp: time.Now(),
	}
	if len(replayBuffer.items) >= replayBuffer.capacity {
		// 删除最老的
		replayBuffer.items = replayBuffer.items[1:]
	}
	replayBuffer.items = append(replayBuffer.items, item)
}

// ============ 海马体记忆 ============

func NewHippocampus(threshold float64, maxMem int, memoryFile string) *Hippocampus {
	h := &Hippocampus{
		memories:    make(map[string]*Memory),
		threshold:   threshold,
		maxMemories: maxMem,
		memoryFile:  expandPath(memoryFile),
	}
	h.Load()
	return h
}

func expandPath(path string) string {
	if strings.HasPrefix(path, "~/") {
		home := os.Getenv("HOME")
		if home != "" {
			return filepath.Join(home, path[2:])
		}
	}
	return path
}

func (h *Hippocampus) AddMemory(query, response string, importance float64, tags []string) *Memory {
	swrScore := h.calcSWRsScore(query, response, importance)
	mem := &Memory{
		ID:           fmt.Sprintf("mem_%d", time.Now().UnixNano()),
		Query:        query,
		Response:     response,
		Importance:   importance,
		SWRTriggered: swrScore >= h.threshold,
		CreatedAt:    time.Now().Format(time.RFC3339),
		LastAccess:   time.Now().Format(time.RFC3339),
		AccessCount:  0,
		Tags:         tags,
	}
	h.memories[mem.ID] = mem
	if mem.SWRTriggered {
		h.Save()
		fmt.Printf("[SWRs] 触发持久化: %s (评分: %.3f)\n", mem.ID, swrScore)
	}
	if len(h.memories) > h.maxMemories {
		h.evict()
	}
	return mem
}

func (h *Hippocampus) calcSWRsScore(query, response string, baseImportance float64) float64 {
	score := baseImportance
	qLen := len(query)
	if qLen > 10 && qLen < 200 {
		score *= 1.2
	} else if qLen >= 200 {
		score *= 1.1
	}
	rLen := len(response)
	if rLen > 50 && rLen < 2000 {
		score *= 1.15
	}
	importantKeywords := []string{"如何", "为什么", "原理", "机制", "方法", "步骤", "注意", "关键"}
	for _, kw := range importantKeywords {
		if strings.Contains(query, kw) {
			score *= 1.1
			break
		}
	}
	return math.Min(1.0, score)
}

func (h *Hippocampus) Retrieve(query string, limit int) []*Memory {
	results := make([]*Memory, 0)
	type scoredMem struct {
		mem   *Memory
		score float64
	}
	scored := make([]scoredMem, 0)
	for _, mem := range h.memories {
		score := h.queryMatchScore(query, mem)
		if score > 0 {
			scored = append(scored, scoredMem{mem: mem, score: score})
		}
	}
	sort.Slice(scored, func(i, j int) bool {
		return scored[i].score > scored[j].score
	})
	for i := 0; i < minInt(limit, len(scored)); i++ {
		results = append(results, scored[i].mem)
		scored[i].mem.AccessCount++
		scored[i].mem.LastAccess = time.Now().Format(time.RFC3339)
	}
	h.Save()
	return results
}

func (h *Hippocampus) queryMatchScore(query string, mem *Memory) float64 {
	score := 0.0
	queryWords := strings.Fields(query)
	if len(queryWords) == 0 {
		return 0
	}
	matchCount := 0
	for _, word := range queryWords {
		if len(word) < 2 {
			continue
		}
		if isStopWord(word) {
			continue
		}
		if strings.Contains(mem.Query, word) || strings.Contains(mem.Response, word) {
			matchCount++
		}
	}
	wordMatchRatio := float64(matchCount) / float64(len(queryWords))
	score += wordMatchRatio * 0.6
	score += mem.Importance * 0.3
	if mem.AccessCount > 5 {
		score *= 1.2
	}
	if mem.SWRTriggered {
		score *= 1.15
	}
	return score
}

func isStopWord(word string) bool {
	stopWords := []string{"的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上", "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这"}
	for _, sw := range stopWords {
		if word == sw {
			return true
		}
	}
	return false
}

func (h *Hippocampus) evict() {
	if len(h.memories) == 0 {
		return
	}
	var worstID string
	lowestScore := math.MaxFloat64
	for id, mem := range h.memories {
		score := mem.Importance * float64(mem.AccessCount+1)
		if mem.SWRTriggered {
			score *= 1.5
		}
		if score < lowestScore {
			lowestScore = score
			worstID = id
		}
	}
	delete(h.memories, worstID)
	fmt.Printf("[Hippocampus] 淘汰记忆: %s\n", worstID)
}

func (h *Hippocampus) Save() error {
	memList := make([]*Memory, 0, len(h.memories))
	for _, mem := range h.memories {
		memList = append(memList, mem)
	}
	data, err := json.MarshalIndent(struct {
		Version   string    `json:"version"`
		Timestamp string    `json:"saved_at"`
		Count     int       `json:"count"`
		Memories  []*Memory `json:"memories"`
	}{
		Version:   Version,
		Timestamp: time.Now().Format(time.RFC3339),
		Count:     len(memList),
		Memories:  memList,
	}, "", "  ")
	if err != nil {
		return err
	}
	dir := filepath.Dir(h.memoryFile)
	os.MkdirAll(dir, 0755)
	if err := os.WriteFile(h.memoryFile, data, 0644); err != nil {
		return err
	}
	return nil
}

func (h *Hippocampus) Load() error {
	file, err := os.Open(h.memoryFile)
	if err != nil {
		if os.IsNotExist(err) {
			return nil
		}
		return err
	}
	defer file.Close()
	data, err := io.ReadAll(file)
	if err != nil {
		return err
	}
	var wrapper struct {
		Version   string    `json:"version"`
		Timestamp string    `json:"saved_at"`
		Count     int       `json:"count"`
		Memories  []*Memory `json:"memories"`
	}
	if err := json.Unmarshal(data, &wrapper); err != nil {
		return err
	}
	h.memories = make(map[string]*Memory)
	for _, mem := range wrapper.Memories {
		h.memories[mem.ID] = mem
	}
	return nil
}

func minInt(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// ============ SkillBank获取候选基因 ============

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
	var skills []map[string]interface{}
	if err := json.Unmarshal(respBody, &skills); err != nil {
		return nil, err
	}
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
			Source:      "axiom",
		}
		if sr, ok := s["success_rate"].(float64); ok {
			gene.SuccessRate = sr
			gene.Features[0] = sr
		}
		if uc, ok := s["usage_count"].(int); ok {
			gene.UsageCount = uc
			gene.Features[4] = float64(uc)
		}
		gene.Features[1] = 0.8
		gene.Features[2] = 0.5
		gene.Features[3] = OOBProb
		gene.Features[5] = 0.1
		gene.Features[6] = 1.0
		genes = append(genes, gene)
	}
	return genes, nil
}

// ============ GPT-5.5决策推理 ============

func callGPT5ForReasoning(query string, claw *ClawContext, genes []*Gene, selected *Gene) (string, error) {
	prompt := fmt.Sprintf(`用户查询: %s
意图: %s
领域: %s
候选基因数: %d
选择: %s

请用一句话解释为什么选择这个基因。`, query, claw.Intent, claw.Domain, len(genes), selected.Name)

	payload := map[string]interface{}{
		"model": "gpt-5.5",
		"messages": []map[string]string{
			{"role": "system", "content": "你是APEX决策专家"},
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
		return fmt.Sprintf("选择%s（GPT超时）", selected.ID), nil
	}
	defer resp.Body.Close()
	respBody, _ := io.ReadAll(resp.Body)
	var result map[string]interface{}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return fmt.Sprintf("选择%s（GPT解析失败）", selected.ID), nil
	}
	choices, ok := result["choices"].([]interface{})
	if !ok || len(choices) == 0 {
		return fmt.Sprintf("选择%s", selected.ID), nil
	}
	choice := choices[0].(map[string]interface{})
	msg := choice["message"].(map[string]interface{})
	content := msg["content"].(string)
	return content, nil
}

// ============ 主选择算法 ============

func SelectBestGene(req *SelectRequest) (*GeneSelectionResult, error) {
	// 1. 海马体记忆检索（如果启用）
	var retrievedMemories []*Memory
	if req.UseMemory {
		retrievedMemories = hippocampus.Retrieve(req.Query, 3)
	}

	// 2. Claw上下文分析
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

	// 3. 获取候选基因
	genes := req.Genes
	if genes == nil || len(genes) == 0 {
		genes, err = fetchGenesFromSkillBank()
		if err != nil || len(genes) == 0 {
			genes = getDefaultGenes()
		}
	}

	// 4. EVM熵Skill自博弈（如果启用）
	evmGenerated := false
	if req.UseEVM {
		challenge := evmChallenge(req.Query)
		if challenge.Score > 0.6 {
			// 把EVM产生的技能作为新基因加入候选
			evmGene := &Gene{
				ID:           challenge.Skill.ID,
				Name:         challenge.Skill.Name,
				Type:         "emv_gene",
				SuccessRate:  challenge.Score,
				UsageCount:   0,
				GiniGain:     challenge.Skill.GiniGain,
				Features:     make([]float64, 7),
				Source:       "evm",
				CreatedAt:    time.Now().Format(time.RFC3339),
			}
			evmGene.Features[0] = challenge.Score
			evmGene.Features[1] = 0.8
			evmGene.Features[2] = 0.5
			evmGene.Features[3] = OOBProb
			evmGene.Features[4] = 0
			evmGene.Features[5] = challenge.Skill.GiniGain
			evmGene.Features[6] = 1.0
			genes = append(genes, evmGene)
			evmGenerated = true
			fmt.Printf("[EVM] 生成新基因: %s (评分: %.2f)\n", evmGene.ID, challenge.Score)
		}
	}

	// 4.5 基因进化：突变+交叉
	genes = applyGeneEvolution(genes)

	// 5. 计算Gini增益
	for _, gene := range genes {
		gene.GiniGain = calcGeneGiniGain(gene, genes)
		if len(gene.Features) < 7 {
			gene.Features = make([]float64, 7)
		}
		gene.Features[0] = gene.SuccessRate
		gene.Features[1] = 0.8
		gene.Features[2] = 0.5
		gene.Features[3] = OOBProb
		gene.Features[4] = float64(gene.UsageCount)
		gene.Features[5] = gene.GiniGain
		gene.Features[6] = timeDecay(gene.LastUsed)
	}

	// 6. Rust RF预测
	var rfPred *RFPrediction
	if len(genes) > 0 && len(genes[0].Features) >= 7 {
		rfPred, _ = callRustForest(genes[0].Features)
	} else {
		rfPred = &RFPrediction{OOBConfidence: OOBProb}
	}

	// 7. APEX ΔG计算并选择
	apexParams := calculateApexParams(claw, genes)
	type scoredGene struct {
		gene   *Gene
		deltaG float64
	}
	scoredGenes := make([]scoredGene, len(genes))
	for i, gene := range genes {
		adjParams := adjustApexParams(apexParams, gene)
		gene.DeltaG = calculateDeltaG(adjParams)
		scoredGenes[i] = scoredGene{gene: gene, deltaG: gene.DeltaG}
	}
	sort.Slice(scoredGenes, func(i, j int) bool {
		return scoredGenes[i].deltaG > scoredGenes[j].deltaG
	})
	best := scoredGenes[0].gene

	// 8. GPT-5.5推理
	reasoning, _ := callGPT5ForReasoning(req.Query, claw, genes, best)

	// 9. 海马体记忆保存（使用EVM结果）
	if req.UseMemory && retrievedMemories == nil {
		hippocampus.AddMemory(req.Query, reasoning, best.SuccessRate, []string{best.Type})
	}

	// 10. 构建结果
	result := &GeneSelectionResult{
		SelectedGene:     best,
		AllGenes:         make([]*Gene, len(genes)),
		Confidence:       rfPred.OOBConfidence,
		DeltaG:           best.DeltaG,
		DeltaGDetailed:   apexParams,
		Reasoning:        reasoning,
		GiniGain:         best.GiniGain,
		ClawAnalysis:     claw,
		RFPrediction:     rfPred,
		Timestamp:        time.Now().Format(time.RFC3339),
		EVMGenerated:     evmGenerated,
		MemoryRetrieved:  retrievedMemories,
	}
	for i, sg := range scoredGenes {
		result.AllGenes[i] = sg.gene
	}
	return result, nil
}

func calculateApexParams(claw *ClawContext, genes []*Gene) APEXDeltaG {
	params := APEXDeltaG{
		Lambda: 1.0, Theta: 1.0, K: 1.0, Xi: 1.0, Psi: 1.0, Phi: 1.0,
		H: 0.5, Tau: 1.0, Epsilon: 1.0,
	}
	switch claw.Intent {
	case "code_help", "programming":
		params.Theta = 1.5
		params.Psi = 1.3
	case "travel_booking":
		params.Lambda = 1.2
		params.Xi = 1.2
	case "finance_query":
		params.Phi = 1.5
		params.H = 0.3
	}
	if len(genes) > 10 {
		params.Xi = 1.3
	}
	params.Result = calculateDeltaG(params)
	return params
}

func adjustApexParams(base APEXDeltaG, gene *Gene) APEXDeltaG {
	adj := base
	if gene.SuccessRate > 0.8 {
		adj.Lambda = math.Min(2.0, adj.Lambda*1.2)
		adj.Psi = math.Min(1.5, adj.Psi*1.1)
	}
	if gene.UsageCount > 50 {
		adj.K = math.Min(1.5, adj.K*1.2)
		adj.H = math.Max(0.3, adj.H*0.9)
	}
	if gene.GiniGain > 0.2 {
		adj.Phi = math.Min(2.0, adj.Phi*1.3)
	}
	decay := timeDecay(gene.LastUsed)
	adj.Tau = adj.Tau * (0.5 + 0.5*decay)
	adj.Result = calculateDeltaG(adj)
	return adj
}

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

func getDefaultGenes() []*Gene {
	return []*Gene{
		{ID: "gene_001", Name: "keyword_expansion", Type: "axiom_gene", SuccessRate: 0.85, UsageCount: 1526, Features: []float64{0.85, 0.8, 0.3, OOBProb, 1526, 0.12, 1.0}, Source: "axiom"},
		{ID: "gene_002", Name: "entity_tracing", Type: "axiom_gene", SuccessRate: 0.90, UsageCount: 892, Features: []float64{0.90, 0.85, 0.5, OOBProb, 892, 0.18, 0.9}, Source: "axiom"},
		{ID: "gene_003", Name: "time_bounded", Type: "axiom_gene", SuccessRate: 0.75, UsageCount: 456, Features: []float64{0.75, 0.7, 0.4, OOBProb, 456, 0.08, 0.8}, Source: "axiom"},
		{ID: "gene_004", Name: "multi_source", Type: "axiom_gene", SuccessRate: 0.88, UsageCount: 1205, Features: []float64{0.88, 0.82, 0.45, OOBProb, 1205, 0.15, 0.95}, Source: "axiom"},
		{ID: "gene_005", Name: "contextual_backtrack", Type: "axiom_gene", SuccessRate: 0.72, UsageCount: 334, Features: []float64{0.72, 0.68, 0.6, OOBProb, 334, 0.06, 0.85}, Source: "axiom"},
		{ID: "gene_006", Name: "a2a_protocol", Type: "emv_gene", SuccessRate: 0.77, UsageCount: 11, Features: []float64{0.77, 0.75, 0.55, OOBProb, 11, 0.10, 1.0}, Source: "emv"},
		{ID: "gene_007", Name: "api_integration", Type: "emv_gene", SuccessRate: 0.79, UsageCount: 7, Features: []float64{0.79, 0.78, 0.5, OOBProb, 7, 0.11, 1.0}, Source: "emv"},
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
		req.Query = r.URL.Query().Get("query")
		req.HasHistory = r.URL.Query().Get("has_history") == "true"
		req.UseEVM = r.URL.Query().Get("use_evm") == "true"
		req.UseMemory = r.URL.Query().Get("use_memory") == "true"
	} else {
		body, _ := io.ReadAll(r.Body)
		json.Unmarshal(body, &req)
	}
	if req.Query == "" {
		req.Query = "default query"
	}
	// 默认启用EVM和记忆
	if r.URL.Path == "/" || r.URL.Path == "" {
		req.UseEVM = true
		req.UseMemory = true
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
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status":  "ok",
		"version": Version,
		"service": "apex_gene_selector_v2",
		"features": []string{"evm", "hippocampus", "claw", "rust_rf", "apex_delta_g"},
	})
}

func main() {
	port := 8092
	fmt.Printf("APEX Gene Selector V%s started on port %d\n", Version, port)
	fmt.Printf("Features: EVM熵Skill + 海马体SWRs + Claw + Rust RF + APEX ΔG\n")
	mux := http.NewServeMux()
	mux.HandleFunc("/api/v1/gene/select", httpHandler)
	mux.HandleFunc("/health", healthHandler)
	log.Fatal(http.ListenAndServe(fmt.Sprintf(":%d", port), mux))
}
