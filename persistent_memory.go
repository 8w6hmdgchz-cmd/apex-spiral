// persistent_memory.go — 长期记忆持久化
// 简单可靠的持久化方案: JSON文件存储

package main

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
)

// MemoryEntry 记忆条目
type MemoryEntry struct {
	ID        string    `json:"id"`
	Content   string    `json:"content"`
	Timestamp time.Time `json:"timestamp"`
	Tags      []string  `json:"tags"`
	Importance float64  `json:"importance"` // 0-1
	AccessCount int     `json:"access_count"`
	LastAccess time.Time `json:"last_access"`
}

// MemoryStore 记忆存储
type MemoryStore struct {
	Dir     string
	Entries map[string]*MemoryEntry
	mu      sync.RWMutex
}

// NewMemoryStore 创建记忆存储
func NewMemoryStore(dir string) *MemoryStore {
	if dir == "" {
		dir = "/Users/lihongxin/Desktop/开智/memory"
	}
	os.MkdirAll(dir, 0755)
	return &MemoryStore{
		Dir: dir,
		Entries: make(map[string]*MemoryEntry),
	}
}

// Load 从磁盘加载
func (ms *MemoryStore) Load() error {
	ms.mu.Lock()
	defer ms.mu.Unlock()

	indexFile := filepath.Join(ms.Dir, "index.json")
	data, err := ioutil.ReadFile(indexFile)
	if err != nil {
		return err
	}

	var ids []string
	if err := json.Unmarshal(data, &ids); err != nil {
		return err
	}

	for _, id := range ids {
		file := filepath.Join(ms.Dir, id+".json")
		entryData, err := ioutil.ReadFile(file)
		if err != nil {
			continue
		}
		var entry MemoryEntry
		if err := json.Unmarshal(entryData, &entry); err != nil {
			continue
		}
		ms.Entries[entry.ID] = &entry
	}

	fmt.Printf("[记忆库] 加载了 %d 条记忆\n", len(ms.Entries))
	return nil
}

// Save 保存到磁盘
func (ms *MemoryStore) Save() error {
	ms.mu.Lock()
	defer ms.mu.Unlock()

	// 保存索引
	ids := make([]string, 0, len(ms.Entries))
	for id := range ms.Entries {
		ids = append(ids, id)
	}
	indexData, err := json.MarshalIndent(ids, "", "  ")
	if err != nil {
		return err
	}
	ioutil.WriteFile(filepath.Join(ms.Dir, "index.json"), indexData, 0644)

	// 保存每个记忆
	for id, entry := range ms.Entries {
		file := filepath.Join(ms.Dir, id+".json")
		data, err := json.MarshalIndent(entry, "", "  ")
		if err != nil {
			continue
		}
		ioutil.WriteFile(file, data, 0644)
	}

	fmt.Printf("[记忆库] 保存了 %d 条记忆\n", len(ms.Entries))
	return nil
}

// Add 添加记忆
func (ms *MemoryStore) Add(content string, tags []string, importance float64) *MemoryEntry {
	ms.mu.Lock()
	defer ms.mu.Unlock()

	id := fmt.Sprintf("mem_%d", time.Now().UnixNano())
	entry := &MemoryEntry{
		ID:         id,
		Content:    content,
		Timestamp:  time.Now(),
		Tags:       tags,
		Importance: importance,
		AccessCount: 0,
		LastAccess: time.Now(),
	}
	ms.Entries[id] = entry

	// 异步保存
	go func() {
		ms.Save()
	}()

	return entry
}

// Search 搜索记忆
func (ms *MemoryStore) Search(query string, limit int) []*MemoryEntry {
	ms.mu.RLock()
	defer ms.mu.RUnlock()

	var results []*MemoryEntry
	queryLower := strings.ToLower(query)

	for _, entry := range ms.Entries {
		// 文本匹配
		if strings.Contains(strings.ToLower(entry.Content), queryLower) {
			results = append(results, entry)
			continue
		}
		// 标签匹配
		for _, tag := range entry.Tags {
			if strings.Contains(strings.ToLower(tag), queryLower) {
				results = append(results, entry)
				break
			}
		}
	}

	// 按重要性排序
	for i := 0; i < len(results); i++ {
		for j := i + 1; j < len(results); j++ {
			if results[j].Importance > results[i].Importance {
				results[i], results[j] = results[j], results[i]
			}
		}
	}

	if limit > 0 && len(results) > limit {
		results = results[:limit]
	}

	// 更新访问计数
	for _, r := range results {
		r.AccessCount++
		r.LastAccess = time.Now()
	}

	return results
}

// Get 获取单条记忆
func (ms *MemoryStore) Get(id string) *MemoryEntry {
	ms.mu.RLock()
	defer ms.mu.RUnlock()

	if entry, ok := ms.Entries[id]; ok {
		entry.AccessCount++
		entry.LastAccess = time.Now()
		return entry
	}
	return nil
}

// Delete 删除记忆
func (ms *MemoryStore) Delete(id string) bool {
	ms.mu.Lock()
	defer ms.mu.Unlock()

	if _, ok := ms.Entries[id]; ok {
		delete(ms.Entries, id)
		os.Remove(filepath.Join(ms.Dir, id+".json"))
		go ms.Save()
		return true
	}
	return false
}

// Stats 获取统计
func (ms *MemoryStore) Stats() map[string]interface{} {
	ms.mu.RLock()
	defer ms.mu.RUnlock()

	total := len(ms.Entries)
	var totalImportance float64
	var totalAccess int

	for _, e := range ms.Entries {
		totalImportance += e.Importance
		totalAccess += e.AccessCount
	}

	return map[string]interface{}{
		"total_memories": total,
		"avg_importance": totalImportance / float64(max(1, total)),
		"total_accesses": totalAccess,
	}
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}

// ============ API ============

var memoryStore *MemoryStore

func init() {
	memoryStore = NewMemoryStore("")
	memoryStore.Load()
}

func memoryAddHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req struct {
		Content    string   `json:"content"`
		Tags       []string `json:"tags"`
		Importance float64  `json:"importance"`
	}
	json.NewDecoder(r.Body).Decode(&req)

	if req.Content == "" {
		json.NewEncoder(w).Encode(map[string]string{"error": "content required"})
		return
	}
	if req.Importance == 0 {
		req.Importance = 0.5
	}

	entry := memoryStore.Add(req.Content, req.Tags, req.Importance)
	json.NewEncoder(w).Encode(entry)
}

func memorySearchHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	query := r.URL.Query().Get("q")
	limit := 10

	results := memoryStore.Search(query, limit)
	json.NewEncoder(w).Encode(map[string]interface{}{
		"results": results,
		"count": len(results),
	})
}

func memoryGetHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	id := r.URL.Query().Get("id")
	entry := memoryStore.Get(id)
	if entry == nil {
		json.NewEncoder(w).Encode(map[string]string{"error": "not found"})
		return
	}
	json.NewEncoder(w).Encode(entry)
}

func memoryDeleteHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	id := r.URL.Query().Get("id")
	ok := memoryStore.Delete(id)
	json.NewEncoder(w).Encode(map[string]bool{"deleted": ok})
}

func memoryStatsHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(memoryStore.Stats())
}

func memoryHealthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status": "ok",
		"service": "persistent_memory",
	})
}

func mainMemoryServer() {
	memoryStore = NewMemoryStore("")
	memoryStore.Load()

	mux := http.NewServeMux()
	mux.HandleFunc("/add", memoryAddHandler)
	mux.HandleFunc("/search", memorySearchHandler)
	mux.HandleFunc("/get", memoryGetHandler)
	mux.HandleFunc("/delete", memoryDeleteHandler)
	mux.HandleFunc("/stats", memoryStatsHandler)
	mux.HandleFunc("/health", memoryHealthHandler)

	fmt.Println("[持久记忆] 服务启动在 :8096")
	fmt.Println("  /add     - 添加记忆")
	fmt.Println("  /search  - 搜索记忆")
	fmt.Println("  /get     - 获取记忆")
	fmt.Println("  /delete  - 删除记忆")
	fmt.Println("  /stats   - 记忆统计")
	http.ListenAndServe(":8096", mux)
}

func main() {
	mainMemoryServer()
}
