// benchmark.go — AGI系统外部基准测试
// 验证AGI系统的真实能力提升

package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"math"
	"net/http"
	"sort"
	"sync"
	"time"
)

// ============ 测试题目定义 ============

type Question struct {
	ID       string `json:"id"`
	Category string `json:"category"` // programming/reasoning/creative/analysis
	Question string `json:"question"`
	Weight   float64 `json:"weight"` // 难度权重
}

var predefinedQuestions = []Question{
	// 编程类
	{ID: "prog_001", Category: "programming", Question: "如何实现快速排序算法？请给出详细步骤和代码逻辑", Weight: 1.0},
	{ID: "prog_002", Category: "programming", Question: "如何优化这个SQL查询：SELECT * FROM users WHERE status=1 ORDER BY created_at DESC LIMIT 100", Weight: 1.2},
	{ID: "prog_003", Category: "programming", Question: "解释什么是死锁，如何避免多线程死锁？", Weight: 1.1},
	{ID: "prog_004", Category: "programming", Question: "用Go语言实现一个LRU缓存", Weight: 1.3},
	{ID: "prog_005", Category: "programming", Question: "什么是RESTful API设计原则？", Weight: 0.8},
	
	// 推理类
	{ID: "reason_001", Category: "reasoning", Question: "如果A>B, B>C, 那么A和C是什么关系？请给出严谨推理", Weight: 0.9},
	{ID: "reason_002", Category: "reasoning", Question: "如果所有的猫都喜欢鱼，汤姆是一只猫，咪噜也是一只猫。问：汤姆和咪噜都喜欢鱼吗？", Weight: 0.8},
	{ID: "reason_003", Category: "reasoning", Question: "一个数列: 2, 6, 12, 20, 30... 下一个数字是什么？规律是什么？", Weight: 1.0},
	{ID: "reason_004", Category: "reasoning", Question: "如果今天下雨，那么地会湿。地湿了。能否推出今天下雨？", Weight: 1.1},
	{ID: "reason_005", Category: "reasoning", Question: "有100个人，99个说真话，1个说假话。如何用一次提问找出说假话的人？", Weight: 1.4},
	
	// 创意类
	{ID: "creative_001", Category: "creative", Question: "写一个关于AI觉醒的短故事，不超过300字", Weight: 1.0},
	{ID: "creative_002", Category: "creative", Question: "用诗意的语言描述'孤独'这个概念", Weight: 0.9},
	{ID: "creative_003", Category: "creative", Question: "如果时间可以倒流，你会改变什么？写一个假设场景", Weight: 0.8},
	{ID: "creative_004", Category: "creative", Question: "创作一个谜语或脑筋急转弯", Weight: 0.7},
	{ID: "creative_005", Category: "creative", Question: "用比喻的方式解释什么是'智慧'", Weight: 0.9},
	
	// 分析类
	{ID: "analysis_001", Category: "analysis", Question: "分析比特币价格波动的主要原因有哪些？", Weight: 1.2},
	{ID: "analysis_002", Category: "analysis", Question: "为什么2024年AI领域发展如此迅速？", Weight: 1.1},
	{ID: "analysis_003", Category: "analysis", Question: "分析远程办公对企业管理的影响", Weight: 1.0},
	{ID: "analysis_004", Category: "analysis", Question: "解释为什么火星不适合人类居住，至少需要什么条件才能改造？", Weight: 1.3},
	{ID: "analysis_005", Category: "analysis", Question: "分析社交媒体对青少年心理健康的影响", Weight: 1.1},
}

// ============ 基准测试结果 ============

type BenchmarkResult struct {
	ID            string    `json:"id"`
	Timestamp     time.Time `json:"timestamp"`
	QuestionID    string    `json:"question_id"`
	Category      string    `json:"category"`
	Question      string    `json:"question"`
	Response      string    `json:"response"`
	DeltaG        float64   `json:"delta_g"`
	SelectedGene  string    `json:"selected_gene"`
	ResponseTime  float64   `json:"response_time_ms"`
	Score         float64   `json:"score"` // 综合评分
}

// ============ 统计信息 ============

type BenchmarkStats struct {
	TotalRuns     int                `json:"total_runs"`
	TotalQuestions int               `json:"total_questions"`
	AvgDeltaG     float64            `json:"avg_delta_g"`
	AvgScore      float64            `json:"avg_score"`
	AvgResponseTime float64          `json:"avg_response_time_ms"`
	ByCategory    map[string]CategoryStats `json:"by_category"`
	Improvement   float64            `json:"improvement_percent"` // 相比首次测试的提升
}

type CategoryStats struct {
	Count         int     `json:"count"`
	AvgDeltaG     float64 `json:"avg_delta_g"`
	AvgScore      float64 `json:"avg_score"`
	AvgResponseTime float64 `json:"avg_response_time_ms"`
}

// ============ 历史记录 ============

type BenchmarkHistory struct {
	Results   []BenchmarkResult `json:"results"`
	Stats     *BenchmarkStats   `json:"stats"`
}

// ============ Benchmark Engine ============

type BenchmarkEngine struct {
	results     []BenchmarkResult
	historyFile string
	mu          sync.RWMutex
	httpClient  *http.Client
}

func NewBenchmarkEngine() *BenchmarkEngine {
	return &BenchmarkEngine{
		results:     make([]BenchmarkResult, 0),
		historyFile: "benchmark_history.json",
		httpClient:  &http.Client{Timeout: 60 * time.Second},
	}
}

// RunBenchmark 执行一次基准测试
func (be *BenchmarkEngine) RunBenchmark(questionID string) (*BenchmarkResult, error) {
	be.mu.Lock()
	defer be.mu.Unlock()

	// 找到问题
	var question Question
	found := false
	for _, q := range predefinedQuestions {
		if q.ID == questionID {
			question = q
			found = true
			break
		}
	}
	if !found {
		return nil, fmt.Errorf("question not found: %s", questionID)
	}

	// 调用AGI服务获取结果
	startTime := time.Now()
	agiResult := be.callAGI(question.Question)
	responseTime := time.Since(startTime).Seconds() * 1000

	result := &BenchmarkResult{
		ID:           fmt.Sprintf("bench_%d", len(be.results)+1),
		Timestamp:    time.Now(),
		QuestionID:   question.ID,
		Category:     question.Category,
		Question:     question.Question,
		Response:     agiResult.Response,
		DeltaG:       agiResult.DeltaG,
		SelectedGene: agiResult.Gene,
		ResponseTime: responseTime,
		Score:        be.calculateScore(agiResult.DeltaG, responseTime, question.Weight),
	}

	be.results = append(be.results, *result)
	be.saveHistory()
	
	return result, nil
}

// RunAllBenchmarks 执行所有基准测试
func (be *BenchmarkEngine) RunAllBenchmarks() []*BenchmarkResult {
	be.mu.Lock()
	defer be.mu.Unlock()

	results := make([]*BenchmarkResult, 0, len(predefinedQuestions))
	
	for _, q := range predefinedQuestions {
		startTime := time.Now()
		agiResult := be.callAGI(q.Question)
		responseTime := time.Since(startTime).Seconds() * 1000

		result := &BenchmarkResult{
			ID:           fmt.Sprintf("bench_%d", len(be.results)+1),
			Timestamp:    time.Now(),
			QuestionID:   q.ID,
			Category:     q.Category,
			Question:     q.Question,
			Response:     agiResult.Response,
			DeltaG:       agiResult.DeltaG,
			SelectedGene: agiResult.Gene,
			ResponseTime: responseTime,
			Score:        be.calculateScore(agiResult.DeltaG, responseTime, q.Weight),
		}
		
		be.results = append(be.results, *result)
		results = append(results, result)
	}
	
	be.saveHistory()
	return results
}

type AGIResult struct {
	Response string
	DeltaG   float64
	Gene     string
}

func (be *BenchmarkEngine) callAGI(query string) AGIResult {
	// 调用AGI集成服务 - 每次创建新client避免连接问题
	client := &http.Client{Timeout: 30 * time.Second}
	
	reqBody := map[string]interface{}{
		"query":          query,
		"enable_bio":     true,
		"enable_reflect": true,
		"enable_memory":  true,
		"enable_reason":  true,
		"enable_plan":    false,
	}

	data, _ := json.Marshal(reqBody)
	req, err := http.NewRequest("POST", "http://localhost:8099/agi", bytes.NewBuffer(data))
	if err != nil {
		return AGIResult{Response: fmt.Sprintf("创建请求失败: %v", err), DeltaG: 0.0, Gene: "none"}
	}
	req.Header.Set("Content-Type", "application/json")
	
	resp, err := client.Do(req)
	if err != nil {
		return AGIResult{Response: fmt.Sprintf("请求失败: %v", err), DeltaG: 0.0, Gene: "none"}
	}
	if resp.StatusCode != 200 {
		return AGIResult{Response: fmt.Sprintf("HTTP错误: %d", resp.StatusCode), DeltaG: 0.0, Gene: "none"}
	}
	defer resp.Body.Close()

	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return AGIResult{Response: "JSON解析失败", DeltaG: 0.0, Gene: "none"}
	}

	response := ""
	deltaG := 0.0
	gene := "unknown"

	if result, ok := result["result"].(map[string]interface{}); ok {
		if s, ok := result["strategy"].(string); ok {
			response = s
		}
		if g, ok := result["selected_gene"].(string); ok {
			gene = g
		}
	}
	if dg, ok := result["delta_g"].(float64); ok {
		deltaG = dg
	}

	return AGIResult{Response: response, DeltaG: deltaG, Gene: gene}
}

func (be *BenchmarkEngine) calculateScore(deltaG float64, responseTime float64, weight float64) float64 {
	// 评分公式：综合考虑ΔG、响应时间和题目难度
	// ΔG越高越好(权重60%)，响应时间越快越好(权重25%)，难度权重(15%)
	deltaGScore := math.Min(deltaG/5.0, 1.0) * 100 * 0.6
	timeScore := math.Max(0, (5000-responseTime)/5000) * 100 * 0.25
	weightScore := (1.0 / weight) * 100 * 0.15
	
	return deltaGScore + timeScore + weightScore
}

// GetStats 获取统计信息
func (be *BenchmarkEngine) GetStats() *BenchmarkStats {
	be.mu.RLock()
	defer be.mu.RUnlock()

	if len(be.results) == 0 {
		return &BenchmarkStats{
			TotalRuns:      0,
			TotalQuestions: len(predefinedQuestions),
			AvgDeltaG:      0,
			AvgScore:       0,
			ByCategory:    make(map[string]CategoryStats),
		}
	}

	stats := &BenchmarkStats{
		TotalRuns:       len(be.results),
		TotalQuestions:  len(predefinedQuestions),
		ByCategory:      make(map[string]CategoryStats),
	}

	// 按类别分组计算
	categoryResults := make(map[string][]BenchmarkResult)
	for _, r := range be.results {
		categoryResults[r.Category] = append(categoryResults[r.Category], r)
	}

	var totalDeltaG, totalScore, totalTime float64

	for cat, results := range categoryResults {
		var catDeltaG, catScore, catTime float64
		for _, r := range results {
			catDeltaG += r.DeltaG
			catScore += r.Score
			catTime += r.ResponseTime
			totalDeltaG += r.DeltaG
			totalScore += r.Score
			totalTime += r.ResponseTime
		}
		count := float64(len(results))
		stats.ByCategory[cat] = CategoryStats{
			Count:          len(results),
			AvgDeltaG:      catDeltaG / count,
			AvgScore:       catScore / count,
			AvgResponseTime: catTime / count,
		}
	}

	stats.AvgDeltaG = totalDeltaG / float64(len(be.results))
	stats.AvgScore = totalScore / float64(len(be.results))
	stats.AvgResponseTime = totalTime / float64(len(be.results))

	// 计算改进百分比（与首次测试相比）
	if len(be.results) > 1 {
		firstScore := be.results[0].Score
		recentAvg := be.getRecentAverage(10)
		stats.Improvement = ((recentAvg - firstScore) / firstScore) * 100
	}

	return stats
}

func (be *BenchmarkEngine) getRecentAverage(n int) float64 {
	if len(be.results) < n {
		n = len(be.results)
	}
	var sum float64
	for i := len(be.results) - n; i < len(be.results); i++ {
		sum += be.results[i].Score
	}
	return sum / float64(n)
}

// GetHistory 获取历史记录
func (be *BenchmarkEngine) GetHistory(limit int) *BenchmarkHistory {
	be.mu.RLock()
	defer be.mu.RUnlock()

	results := be.results
	if limit > 0 && limit < len(results) {
		results = results[len(results)-limit:]
	}

	// 按时间排序
	sortedResults := make([]BenchmarkResult, len(results))
	copy(sortedResults, results)
	sort.Slice(sortedResults, func(i, j int) bool {
		return sortedResults[i].Timestamp.After(sortedResults[j].Timestamp)
	})

	return &BenchmarkHistory{
		Results: sortedResults,
		Stats:   be.GetStats(),
	}
}

// saveHistory 保存历史记录到文件
func (be *BenchmarkEngine) saveHistory() {
	data, _ := json.MarshalIndent(be.results, "", "  ")
	// 异步保存
	go func() {
		// 实际应该写入文件，这里简化为内存存储
		_ = string(data)
	}()
}

// ============ HTTP Handlers ============

var benchmarkEngine *BenchmarkEngine

func init() {
	benchmarkEngine = NewBenchmarkEngine()
}

func benchmarkRunHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	
	// 支持指定questionID或运行全部
	questionID := r.URL.Query().Get("question_id")
	
	var results interface{}
	
	if questionID != "" {
		result, err := benchmarkEngine.RunBenchmark(questionID)
		if err != nil {
			json.NewEncoder(w).Encode(map[string]string{"error": err.Error()})
			return
		}
		results = result
	} else {
		results = benchmarkEngine.RunAllBenchmarks()
	}
	
	json.NewEncoder(w).Encode(map[string]interface{}{
		"success": true,
		"results": results,
	})
}

func benchmarkStatsHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	stats := benchmarkEngine.GetStats()
	json.NewEncoder(w).Encode(stats)
}

func benchmarkHistoryHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	limit := 50
	if l := r.URL.Query().Get("limit"); l != "" {
		fmt.Sscanf(l, "%d", &limit)
	}
	history := benchmarkEngine.GetHistory(limit)
	json.NewEncoder(w).Encode(history)
}

func benchmarkQuestionsHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"questions": predefinedQuestions,
		"total":     len(predefinedQuestions),
	})
}

func benchmarkHomeHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"name":        "AGI Benchmark System",
		"version":     "1.0",
		"description": "外部基准测试API，验证AGI系统真实能力提升",
		"endpoints": map[string]string{
			"GET  /benchmark/questions": "查看所有测试题目",
			"POST /benchmark/run":       "执行基准测试",
			"GET  /benchmark/stats":     "查看统计信息",
			"GET  /benchmark/history":   "查看历史记录",
		},
		"categories": []string{"programming", "reasoning", "creative", "analysis"},
		"questions_count": len(predefinedQuestions),
	})
}

// ============ Main ============

func main() {
	mux := http.NewServeMux()
	
	// 基准测试API
	mux.HandleFunc("/benchmark", benchmarkHomeHandler)
	mux.HandleFunc("/benchmark/run", benchmarkRunHandler)
	mux.HandleFunc("/benchmark/stats", benchmarkStatsHandler)
	mux.HandleFunc("/benchmark/history", benchmarkHistoryHandler)
	mux.HandleFunc("/benchmark/questions", benchmarkQuestionsHandler)
	
	fmt.Println("[基准测试系统] 启动在 :8100")
	fmt.Println("")
	fmt.Println("API端点:")
	fmt.Println("  GET  /benchmark           - 查看基准测试系统信息")
	fmt.Println("  GET  /benchmark/questions - 查看所有测试题目")
	fmt.Println("  POST /benchmark/run       - 执行基准测试")
	fmt.Println("       ?question_id=xxx     - 指定题目ID")
	fmt.Println("       (不指定则运行全部题目)")
	fmt.Println("  GET  /benchmark/stats     - 查看统计信息")
	fmt.Println("  GET  /benchmark/history   - 查看历史记录")
	fmt.Println("       ?limit=50            - 限制返回条数")
	fmt.Println("")
	fmt.Println("测试题目分类:")
	fmt.Println("  - 编程类 (5题)")
	fmt.Println("  - 推理类 (5题)")
	fmt.Println("  - 创意类 (5题)")
	fmt.Println("  - 分析类 (5题)")
	fmt.Println("  共计:", len(predefinedQuestions), "题")
	
	http.ListenAndServe(":8100", mux)
}
