// apex_gini.go - Gini 基尼增益选择器
// 基于基尼不纯度和信息熵的决策树选择机制
// 璇玑帝国 · OpenClaw Native Integration
package main

import (
	"encoding/json"
	"fmt"
	"math"
	"os"
)

// ============ Gini 核心算法 ============

// Gini 不纯度: Gini = 1 - Σp_k²
func GiniImpurity(counts []float64) float64 {
	total := sum(counts)
	if total <= 0 {
		return 0
	}
	var gini float64
	for _, c := range counts {
		if c <= 0 {
			continue
		}
		p := c / total
		gini += p * p
	}
	return 1.0 - gini
}

// ΔGini = Gini父 - (N_L/N × Gini_L + N_R/N × Gini_R)
func GiniGain(parentCounts, leftCounts, rightCounts []float64) float64 {
	parentGini := GiniImpurity(parentCounts)
	total := sum(parentCounts)
	if total <= 0 {
		return 0
	}

	leftTotal := sum(leftCounts)
	rightTotal := sum(rightCounts)
	leftWeight := leftTotal / total
	rightWeight := rightTotal / total

	return parentGini - (leftWeight*GiniImpurity(leftCounts) + rightWeight*GiniImpurity(rightCounts))
}

// 信息熵: H = -Σp_k × log₂(p_k)
func Entropy(counts []float64) float64 {
	total := sum(counts)
	if total <= 0 {
		return 0
	}
	var h float64
	for _, c := range counts {
		if c <= 0 {
			continue
		}
		p := c / total
		h += p * math.Log2(p)
	}
	return -h
}

// IG = H父 - Σ(N_v/N × H_v)
func InformationGain(parentCounts []float64, childGroups [][]float64) float64 {
	parentEnt := Entropy(parentCounts)
	total := sum(parentCounts)
	if total <= 0 {
		return 0
	}

	var weightedChildEnt float64
	for _, child := range childGroups {
		weight := sum(child) / total
		weightedChildEnt += weight * Entropy(child)
	}

	return parentEnt - weightedChildEnt
}

// ============ 软投票 ============

// SoftVote 软投票概率预测
func SoftVote(predictions []map[string]float64) map[string]float64 {
	if len(predictions) == 0 {
		return make(map[string]float64)
	}

	// 收集所有 key
	keys := make(map[string]bool)
	for _, p := range predictions {
		for k := range p {
			keys[k] = true
		}
	}

	// 对每类概率取平均
	result := make(map[string]float64)
	n := float64(len(predictions))
	for k := range keys {
		var sum float64
		for _, p := range predictions {
			sum += p[k]
		}
		result[k] = sum / n
	}
	return result
}

// ============ Gini 选择器 ============

// PathScore 推理路径评分
type PathScore struct {
	Path   string  `json:"path"`
	Score  float64 `json:"score"`
	Gini   float64 `json:"gini"`
	Votes  int     `json:"votes"`
}

// GiniResult Gini 选择结果
type GiniResult struct {
	BestPath    string       `json:"best_path"`
	BestGain    float64      `json:"best_gain"`
	AllPaths    []PathScore  `json:"all_paths"`
	SoftVoteMap map[string]float64 `json:"soft_vote_map"`
	Timestamp   string       `json:"timestamp"`
}

// GiniSelect 多路径推理 Gini 增益选择
func GiniSelect(paths []string) *GiniResult {
	if len(paths) == 0 {
		return &GiniResult{}
	}

	// 为每个路径分配初始计数（成功/失败）
	counts := make([][]float64, len(paths))
	predictions := make([]map[string]float64, len(paths))

	// 简单模型：用路径索引作为"类别"，构造多类计数
	// 实际使用中，paths 应包含真实推理结果
	for i, path := range paths {
		// 模拟：基于路径长度和内容生成评分
		score := pathScore(path)
		predictions[i] = map[string]float64{
			"quality": score,
			"path_id": float64(i),
		}
		// 构造二分类计数（高质量/低质量）
		if score > 0.5 {
			counts[i] = []float64{score, 1 - score}
		} else {
			counts[i] = []float64{score, 1 - score}
		}
	}

	// 计算每个路径的 Gini 增益
	var allPaths []PathScore
	var bestGain float64
	var bestPath string

	// 父节点 Gini
	parentGini := GiniImpurity([]float64{float64(len(paths))})

	for i, path := range paths {
		gini := GiniImpurity(counts[i])
		gain := parentGini - gini

		ps := PathScore{
			Path:   truncate(path, 60),
			Score:  predictions[i]["quality"],
			Gini:   gini,
			Votes:  0,
		}
		allPaths = append(allPaths, ps)

		if gain > bestGain {
			bestGain = gain
			bestPath = path
		}
	}

	// 软投票
	svm := SoftVote(predictions)

	return &GiniResult{
		BestPath:    truncate(bestPath, 60),
		BestGain:    bestGain,
		AllPaths:    allPaths,
		SoftVoteMap: svm,
		Timestamp:   fmt.Sprintf("%d", nowSec()),
	}
}

// ============ 辅助函数 ============

func sum(vals []float64) float64 {
	var s float64
	for _, v := range vals {
		s += v
	}
	return s
}

func nowSec() int64 {
	return int64(nowUnix())
}

func nowUnix() float64 {
	return float64(os.Getpid()) // placeholder
}

// pathScore 简单路径评分（实际应接 LLM）
func pathScore(path string) float64 {
	// 基于路径长度和信息量估算
	l := float64(len(path))
	if l <= 0 {
		return 0.3
	}
	// 归一化到 [0.3, 0.9]
	score := 0.3 + math.Min(0.6, l/200.0)
	return math.Min(0.9, score)
}

func truncate(s string, max int) string {
	if len(s) <= max {
		return s
	}
	return s[:max] + "..."
}

// ============ CLI ============

func main() {
	if len(os.Args) < 2 {
		// 尝试从 stdin 读取
		data, _ := os.ReadFile("/dev/stdin")
		if len(data) > 0 {
			var paths []string
			if err := json.Unmarshal(data, &paths); err == nil {
				result := GiniSelect(paths)
				enc := json.NewEncoder(os.Stdout)
				enc.SetEscapeHTML(false)
				enc.Encode(result)
				return
			}
		}
		fmt.Println("APEX Gini Selector")
		fmt.Println("用法: apex_gini '[\"路径1\",\"路径2\"]'")
		os.Exit(1)
	}

	var paths []string
	var err error

	err = json.Unmarshal([]byte(os.Args[1]), &paths)
	if err != nil {
		fmt.Fprintf(os.Stderr, "JSON解析失败: %v\n", err)
		os.Exit(1)
	}

	result := GiniSelect(paths)

	enc := json.NewEncoder(os.Stdout)
	enc.SetEscapeHTML(false)
	enc.Encode(result)
}
