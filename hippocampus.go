// hippocampus.go — 海马体SWRs记忆机制 V1.0
//
// 大脑机制类比：
// 1. 经验输入 → 海马体编码
// 2. SWRs (Sharp-Wave Ripples) 选择重要经验
// 3. 回放巩固到新皮层
// 4. 形成稳定记忆
//
// AI实现：
// 1. 对话输入 → 临时记忆存入Hippocampus
// 2. SWRs评分 > 阈值触发持久化到长期记忆
// 3. 保存到memory.json
// 4. 下次查询优先从记忆检索
//
// 编译: cd ~/Desktop/开智 && go build -o hippocampus hippocampus.go

package main

import (
	"encoding/json"
	"fmt"
	"io"
	"math"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

// ============ 常量 ============

const (
	Version           = "1.0"
	MemoryFilePath    = "~/Desktop/开智/memory.json"
	DefaultThreshold  = 0.7  // SWRs触发阈值
	DefaultMaxMemories = 100 // 最大记忆数
)

// ============ 数据结构 ============

// Memory 单条记忆
type Memory struct {
	ID           string  `json:"id"`
	Query        string  `json:"query"`
	Response     string  `json:"response"`
	Importance   float64 `json:"importance"`   // SWRs重要性评分 (0-1)
	SWRTriggered bool    `json:"swr_triggered"` // 是否被SWRs选中
	CreatedAt    string  `json:"created_at"`
	LastAccess   string  `json:"last_access"`
	AccessCount  int     `json:"access_count"`
	Tags         []string `json:"tags"`        // 标签分类
}

// Hippocampus 海马体
type Hippocampus struct {
	memories     map[string]*Memory
	threshold    float64
	maxMemories  int
	memoryFile   string
}

// SWRsEvent SWR事件记录
type SWRsEvent struct {
	Timestamp   string  `json:"timestamp"`
	MemoryID    string  `json:"memory_id"`
	Score       float64 `json:"score"`
	Triggered   bool    `json:"triggered"`
}

// ============ 海马体核心算法 ============

// NewHippocampus 创建海马体
func NewHippocampus(threshold float64, maxMem int, memoryFile string) *Hippocampus {
	h := &Hippocampus{
		memories:    make(map[string]*Memory),
		threshold:   threshold,
		maxMemories: maxMem,
		memoryFile:  expandPath(memoryFile),
	}
	// 加载已有记忆
	h.Load()
	return h
}

// expandPath 展开~路径
func expandPath(path string) string {
	if strings.HasPrefix(path, "~/") {
		home := os.Getenv("HOME")
		if home != "" {
			return filepath.Join(home, path[2:])
		}
	}
	return path
}

// AddMemory 添加记忆
func (h *Hippocampus) AddMemory(query, response string, importance float64, tags []string) *Memory {
	// 计算SWRs评分
	swrScore := h.calcSWRsScore(query, response, importance)

	// 创建记忆
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

	// 添加到记忆库
	h.memories[mem.ID] = mem

	// 如果SWRs触发，写入持久化
	if mem.SWRTriggered {
		h.Save()
		fmt.Printf("[SWRs] 触发持久化: %s (评分: %.3f)\n", mem.ID, swrScore)
	}

	// 如果超过最大记忆数，删除最不重要的
	if len(h.memories) > h.maxMemories {
		h.evict()
	}

	return mem
}

// calcSWRsScore 计算SWRs评分
// 类比大脑SWRs：高频放电选择重要经验
func (h *Hippocampus) calcSWRsScore(query, response string, baseImportance float64) float64 {
	score := baseImportance

	// 查询长度因子（中等长度更有信息量）
	qLen := len(query)
	if qLen > 10 && qLen < 200 {
		score *= 1.2 // 适中长度加分
	} else if qLen >= 200 {
		score *= 1.1 // 长查询可能有更多上下文
	}

	// 响应长度因子
	rLen := len(response)
	if rLen > 50 && rLen < 2000 {
		score *= 1.15
	}

	// 包含特定关键词（暗示重要）
	importantKeywords := []string{"如何", "为什么", "原理", "机制", "方法", "步骤", "注意", "关键"}
	for _, kw := range importantKeywords {
		if strings.Contains(query, kw) {
			score *= 1.1
			break
		}
	}

	return math.Min(1.0, score)
}

// min 最小值
func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// Retrieve 检索记忆
func (h *Hippocampus) Retrieve(query string, limit int) []*Memory {
	results := make([]*Memory, 0)

	// 计算每个记忆与查询的相关性
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

	// 按相关性排序
	sort.Slice(scored, func(i, j int) bool {
		return scored[i].score > scored[j].score
	})

	// 取前limit个
	for i := 0; i < min(limit, len(scored)); i++ {
		results = append(results, scored[i].mem)
		// 更新访问记录
		scored[i].mem.AccessCount++
		scored[i].mem.LastAccess = time.Now().Format(time.RFC3339)
	}

	h.Save() // 保存访问更新
	return results
}

// queryMatchScore 计算查询与记忆的匹配分数
func (h *Hippocampus) queryMatchScore(query string, mem *Memory) float64 {
	score := 0.0

	// 查询词在记忆中出现的比例
	queryWords := strings.Fields(query)
	if len(queryWords) == 0 {
		return 0
	}

	matchCount := 0
	for _, word := range queryWords {
		if len(word) < 2 {
			continue
		}
		// 忽略停用词
		if isStopWord(word) {
			continue
		}
		if strings.Contains(mem.Query, word) || strings.Contains(mem.Response, word) {
			matchCount++
		}
	}

	wordMatchRatio := float64(matchCount) / float64(len(queryWords))
	score += wordMatchRatio * 0.6

	// 重要性加权
	score += mem.Importance * 0.3

	// 访问频率加权（常用记忆优先）
	if mem.AccessCount > 5 {
		score *= 1.2
	} else if mem.AccessCount > 10 {
		score *= 1.3
	}

	// SWRs触发的记忆优先
	if mem.SWRTriggered {
		score *= 1.15
	}

	return score
}

// isStopWord 判断是否为停用词
func isStopWord(word string) bool {
	stopWords := []string{"的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上", "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这"}
	for _, sw := range stopWords {
		if word == sw {
			return true
		}
	}
	return false
}

// evict 淘汰最不重要的记忆
func (h *Hippocampus) evict() {
	if len(h.memories) == 0 {
		return
	}

	// 找出评分最低的记忆
	var worstID string
	lowestScore := math.MaxFloat64

	for id, mem := range h.memories {
		score := mem.Importance * float64(mem.AccessCount+1)
		if mem.SWRTriggered {
			score *= 1.5 // SWRs触发的记忆不容易被淘汰
		}
		if score < lowestScore {
			lowestScore = score
			worstID = id
		}
	}

	delete(h.memories, worstID)
	fmt.Printf("[Hippocampus] 淘汰记忆: %s (评分: %.3f)\n", worstID, lowestScore)
}

// Save 持久化到文件
func (h *Hippocampus) Save() error {
	// 转换map到slice
	memList := make([]*Memory, 0, len(h.memories))
	for _, mem := range h.memories {
		memList = append(memList, mem)
	}

	// JSON格式化
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

	// 确保目录存在
	dir := filepath.Dir(h.memoryFile)
	os.MkdirAll(dir, 0755)

	// 写入文件
	if err := os.WriteFile(h.memoryFile, data, 0644); err != nil {
		return err
	}

	fmt.Printf("[Hippocampus] 已保存 %d 条记忆到 %s\n", len(memList), h.memoryFile)
	return nil
}

// Load 从文件加载记忆
func (h *Hippocampus) Load() error {
	file, err := os.Open(h.memoryFile)
	if err != nil {
		if os.IsNotExist(err) {
			fmt.Println("[Hippocampus] 记忆文件不存在，创建新海马体")
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

	fmt.Printf("[Hippocampus] 加载了 %d 条记忆\n", len(h.memories))
	return nil
}

// Stats 返回统计信息
func (h *Hippocampus) Stats() map[string]interface{} {
	swrTriggered := 0
	totalAccess := 0
	for _, mem := range h.memories {
		if mem.SWRTriggered {
			swrTriggered++
		}
		totalAccess += mem.AccessCount
	}

	return map[string]interface{}{
		"total_memories":   len(h.memories),
		"max_memories":     h.maxMemories,
		"swr_triggered":    swrTriggered,
		"total_accesses":   totalAccess,
		"threshold":        h.threshold,
		"version":          Version,
	}
}

// ============ 主函数（测试用）==========

func main() {
	fmt.Println("=== 海马体SWRs记忆机制 V" + Version + " ===")
	fmt.Println()

	// 创建海马体
	h := NewHippocampus(DefaultThreshold, DefaultMaxMemories, MemoryFilePath)

	// 添加一些测试记忆
	testMemories := []struct {
		query     string
		response  string
		importance float64
		tags      []string
	}{
		{"如何学习Rust", "Rust学习路径：1.所有权 2.生命周期 3.trait 4.并发", 0.9, []string{"编程", "Rust"}},
		{"什么是APEX", "APEX是一个自进化AI框架，包含ΔG公式和基因网络", 0.85, []string{"AI", "APEX"}},
		{"Gini增益公式", "ΔGini = Gini父 - (NL/N×Gini左 + NR/N×Gini右)", 0.95, []string{"数学", "ML"}},
	}

	for _, tm := range testMemories {
		mem := h.AddMemory(tm.query, tm.response, tm.importance, tm.tags)
		fmt.Printf("添加记忆: %s (SWR触发: %v)\n", mem.ID, mem.SWRTriggered)
	}

	fmt.Println()

	// 检索测试
	query := "Rust怎么学"
	results := h.Retrieve(query, 3)
	fmt.Printf("\n检索'%s'，找到%d条记忆:\n", query, len(results))
	for i, mem := range results {
		fmt.Printf("  %d. [%s] %s (匹配度: %.2f)\n", i+1, mem.ID, mem.Query, mem.Importance)
	}

	fmt.Println()

	// 统计信息
	stats := h.Stats()
	fmt.Println("=== 海马体统计 ===")
	for k, v := range stats {
		fmt.Printf("  %s: %v\n", k, v)
	}

	// 手动保存
	h.Save()
}
