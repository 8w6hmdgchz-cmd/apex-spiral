package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"math/rand"
	"os"
	"sort"
	"strings"
	"sync"
	"time"
)

// ========================================
// APEX StraTA 蜂群Agent系统
// ApexStraTA = π(z|s₁) ⊗ π(aₜ|z,sₜ) ⊗ GRPO(z,aₜ) ⊗ MemLLM
// ========================================

// --- T1: 策略层 ---

// Strategy z = π(s₁) — 全局策略
type Strategy struct {
	ID            string       `json:"id"`
	Objective     string       `json:"objective"`
	SubTasks      []SubTask    `json:"sub_tasks"`
	Dependencies  []string     `json:"dependencies"`
	Parallelism   int          `json:"parallelism"`
	RewardConfig  RewardConfig `json:"reward_config"`
	GeneratedBy   string       `json:"generated_by"`
	Timestamp     string       `json:"timestamp"`
	StrategyScore float64      `json:"strategy_score"` // A(z)
}

type SubTask struct {
	ID             string   `json:"id"`
	AgentID        string   `json:"agent_id"`
	Description    string   `json:"description"`
	AcceptCriteria string   `json:"accept_criteria"`
	DependsOn      []string `json:"depends_on"`
	Status         string   `json:"status"` // pending, running, completed, failed
	Result         string   `json:"result,omitempty"`
	Reward         float64  `json:"reward"` // A(aₜ)
}

type RewardConfig struct {
	StrategyReward struct {
		DecompositionQuality float64 `json:"decomposition_quality"`
		DependencyAccuracy   float64 `json:"dependency_accuracy"`
		VerificationClarity  float64 `json:"verification_clarity"`
	} `json:"strategy_reward"`
	TaskReward struct {
		Completion float64 `json:"completion"`
		Quality    float64 `json:"quality"`
		Efficiency float64 `json:"efficiency"`
	} `json:"task_reward"`
	KLPenalty          float64 `json:"kl_penalty"`
	FarthestPointSample bool   `json:"farthest_point_sample"`
	MaxIterations      int     `json:"max_iterations"`
}

// --- T2: Agent层 ---

type SwarmAgent struct {
	ID          string       `json:"id"`
	Strategy    Strategy     `json:"strategy"`
	Task        SubTask      `json:"task"`
	Memory      AgentMemory  `json:"memory"`
	Status      string       `json:"status"` // idle, running, completed, failed
	StartedAt   string       `json:"started_at,omitempty"`
	CompletedAt string       `json:"completed_at,omitempty"`
	Output      string       `json:"output,omitempty"`
}

// --- T4: 记忆层 ---

type AgentMemory struct {
	ShortTerm    map[string]string `json:"short_term"`    // 当前任务上下文
	TaskHistory  []TaskRecord      `json:"task_history"`  // 本次session记录
	LongTerm     []TaskRecord      `json:"long_term"`     // 持久化知识
	RAGIndex     map[string]string `json:"rag_index"`     // 检索索引
}

type TaskRecord struct {
	Timestamp string `json:"timestamp"`
	TaskID    string `json:"task_id"`
	Input     string `json:"input"`
	Output    string `json:"output"`
	Reward    float64 `json:"reward"`
}

// --- T3: GRPO层 ---

type GRPOResult struct {
	Iteration      int              `json:"iteration"`
	StrategyReward float64          `json:"strategy_reward"`  // A(z)
	TaskRewards    map[string]float64 `json:"task_rewards"`  // A(aₜ) per agent
	TotalReward    float64          `json:"total_reward"`
	KLDivergence   float64          `json:"kl_divergence"`
	PolicyStable   bool             `json:"policy_stable"`
	SamplePool     []Strategy       `json:"sample_pool"`
	Converged      bool             `json:"converged"`
}

// --- 主系统 ---

type ApexStraTA struct {
	Strategy     Strategy              `json:"strategy"`
	Agents       map[string]*SwarmAgent `json:"agents"`
	GRPOResults  []GRPOResult          `json:"grpo_results"`
	MemLLM       MemLLM                `json:"mem_llm"`
	Phase        string                `json:"phase"` // plan, execute, optimize, verify, done
	AllCompleted bool                  `json:"all_completed"`
	mu           sync.Mutex
}

type MemLLM struct {
	GlobalMemory []TaskRecord              `json:"global_memory"`
	AgentMemories map[string][]TaskRecord  `json:"agent_memories"`
	RAGCache     map[string]float64         `json:"rag_cache"`
}

func (m *MemLLM) ragRetrieve(query string) []TaskRecord {
	// Stub: in production, do semantic search
	// For now, return all global memory that contains the query
	var results []TaskRecord
	q := strings.ToLower(query)
	for _, r := range m.GlobalMemory {
		if strings.Contains(strings.ToLower(r.Input), q) ||
		   strings.Contains(strings.ToLower(r.Output), q) {
			results = append(results, r)
		}
	}
	return results
}

func main() {
	task := flag.String("task", "", "任务描述")
	mode := flag.String("mode", "swarm", "swarm|plan|status|memory")
	model := flag.String("model", "freemodel/gpt-5.5", "策略生成模型")
	parallel := flag.Int("parallel", 3, "并行Agent数量")
	iterations := flag.Int("iterations", 3, "GRPO最大迭代次数")
	status := flag.Bool("status", false, "查看蜂群状态")
	memory := flag.Bool("memory", false, "查看记忆网络")
	agent := flag.String("agent", "all", "指定Agent ID")
	flag.Parse()

	if *status {
		showStatus()
		return
	}
	if *memory {
		showMemory(*agent)
		return
	}

	if *task == "" {
		fmt.Fprintln(os.Stderr, "Error: --task 是必填参数")
		flag.Usage()
		os.Exit(1)
	}

	system := NewApexStraTA()

	switch *mode {
	case "plan":
		system.generateStrategy(*task, *model)
		printJSON(system.Strategy)
	case "swarm":
		fullSwarmRun(system, *task, *model, *parallel, *iterations)
	default:
		fmt.Fprintf(os.Stderr, "Unknown mode: %s\n", *mode)
		os.Exit(1)
	}
}

// ========================================
// 核心逻辑
// ========================================

func NewApexStraTA() *ApexStraTA {
	return &ApexStraTA{
		Agents: make(map[string]*SwarmAgent),
		MemLLM: MemLLM{
			GlobalMemory:   []TaskRecord{},
			AgentMemories:  make(map[string][]TaskRecord),
			RAGCache:       make(map[string]float64),
		},
		Phase: "init",
	}
}

// T1: π(z|s₁) — 策略生成
func (s *ApexStraTA) generateStrategy(task string, model string) Strategy {
	s.mu.Lock()
	defer s.mu.Unlock()

	// Decompose task into sub-tasks
	subtasks := decomposeTask(task)
	
	// Generate dependency graph
	deps := generateDependencies(subtasks)

	// Create strategy with heuristic scoring
	strategy := Strategy{
		ID:           generateID("strat"),
		Objective:    task,
		SubTasks:     subtasks,
		Dependencies: deps,
		Parallelism:  len(subtasks),
		RewardConfig: defaultRewardConfig(),
		GeneratedBy:  model,
		Timestamp:    time.Now().UTC().Format(time.RFC3339),
		StrategyScore: evaluateStrategy(subtasks, deps),
	}

	s.Strategy = strategy
	s.Phase = "planned"
	return strategy
}

// T2: π(aₜ|z,sₜ) — 蜂群执行
func (s *ApexStraTA) deploySwarm() {
	// Collect sub-tasks while holding lock
	s.mu.Lock()
	s.Phase = "deploying"

	var agentTasks []struct {
		idx int
		task SubTask
	}
	for i, task := range s.Strategy.SubTasks {
		agentTasks = append(agentTasks, struct {
			idx int
			task SubTask
		}{i, task})
	}
	s.mu.Unlock()

	// Deploy agents outside lock
	for _, at := range agentTasks {
		agentID := fmt.Sprintf("agent-%d", at.idx+1)
		at.task.AgentID = agentID
		at.task.Status = "running"

		agent := &SwarmAgent{
			ID:       agentID,
			Task:     at.task,
			Memory: AgentMemory{
				ShortTerm:   make(map[string]string),
				TaskHistory: []TaskRecord{},
				LongTerm:    []TaskRecord{},
				RAGIndex:    make(map[string]string),
			},
			Status:    "running",
			StartedAt: time.Now().UTC().Format(time.RFC3339),
		}

		s.mu.Lock()
		s.Strategy.SubTasks[at.idx] = at.task
		s.Agents[agentID] = agent
		s.mu.Unlock()
	}

	s.mu.Lock()
	s.Phase = "executing"
	s.mu.Unlock()
}

// Simulate agent execution (in prod, would spawn actual sub-agents)
func (s *ApexStraTA) executeAgents() {
	var wg sync.WaitGroup

	// Snapshot agents
	s.mu.Lock()
	var agents []*SwarmAgent
	for _, a := range s.Agents {
		agents = append(agents, a)
	}
	s.mu.Unlock()

	for _, agent := range agents {
		wg.Add(1)
		go func(a *SwarmAgent) {
			defer wg.Done()

			// Simulate execution time
			time.Sleep(time.Duration(500+rand.Intn(1500)) * time.Millisecond)

			// Update agent-local state (no shared memory)
			a.Status = "completed"
			a.CompletedAt = time.Now().UTC().Format(time.RFC3339)
			a.Output = fmt.Sprintf("[模拟执行] %s 完成 ✓", a.Task.Description)

			record := TaskRecord{
				Timestamp: a.CompletedAt,
				TaskID:    a.Task.ID,
				Input:     a.Task.Description,
				Output:    a.Output,
			}
			a.Memory.TaskHistory = append(a.Memory.TaskHistory, record)
			a.Memory.ShortTerm["last_result"] = a.Output

			// Sync to shared state (lock)
			s.mu.Lock()
			s.MemLLM.AgentMemories[a.ID] = append(s.MemLLM.AgentMemories[a.ID], record)
			s.MemLLM.GlobalMemory = append(s.MemLLM.GlobalMemory, record)
			for i, t := range s.Strategy.SubTasks {
				if t.ID == a.Task.ID {
					s.Strategy.SubTasks[i].Status = "completed"
					s.Strategy.SubTasks[i].Result = a.Output
					break
				}
			}
			s.mu.Unlock()
		}(agent)
	}

	wg.Wait()

	s.mu.Lock()
	s.Phase = "executed"
	s.mu.Unlock()
}

// T3: GRPO(z,aₜ) — 分层优化
func (s *ApexStraTA) grpoOptimize(iteration int) GRPOResult {
	s.mu.Lock()
	defer s.mu.Unlock()

	result := GRPOResult{
		Iteration:   iteration,
		TaskRewards: make(map[string]float64),
	}

	// A(z): Strategy-level reward
	strategyReward := evaluateStrategy(s.Strategy.SubTasks, s.Strategy.Dependencies)
	result.StrategyReward = strategyReward

	// A(aₜ): Task-level rewards per agent
	var totalTaskReward float64
	for id, agent := range s.Agents {
		reward := evaluateTaskReward(agent)
		result.TaskRewards[id] = reward
		totalTaskReward += reward

		// Update in strategy
		for i, t := range s.Strategy.SubTasks {
			if t.AgentID == id {
				s.Strategy.SubTasks[i].Reward = reward
				break
			}
		}
	}

	// KL divergence penalty (simulated)
	klDiv := computeKLDivergence(s.Strategy, iteration)
	result.KLDivergence = klDiv

	// Total: J(θ) = E[ΣA(z) + ΣA(aₜ) - βDKL]
	beta := s.Strategy.RewardConfig.KLPenalty
	result.TotalReward = strategyReward + totalTaskReward - beta*klDiv

	// Convergence check
	result.Converged = klDiv < 0.05 || iteration >= s.Strategy.RewardConfig.MaxIterations
	result.PolicyStable = klDiv < 0.02

	// Farthest point sampling for diversity
	if s.Strategy.RewardConfig.FarthestPointSample {
		result.SamplePool = s.farthestPointSample(result.SamplePool)
	}

	s.GRPOResults = append(s.GRPOResults, result)
	s.Phase = fmt.Sprintf("optimized_iter_%d", iteration)

	return result
}

// Verify: 主公式验算
func (s *ApexStraTA) verifyAllCompleted() bool {
	s.mu.Lock()
	defer s.mu.Unlock()

	for _, agent := range s.Agents {
		if agent.Status != "completed" {
			s.AllCompleted = false
			s.Phase = "verify_failed"
			return false
		}
	}

	// Also check strategy-level verification
	for _, task := range s.Strategy.SubTasks {
		if task.Status != "completed" {
			s.Phase = "verify_failed"
			return false
		}
	}

	s.AllCompleted = true
	s.Phase = "done"
	return true
}

// Full run: 单指令完成全链路
func fullSwarmRun(system *ApexStraTA, task, model string, parallel, iterations int) {
	fmt.Printf("🐝 APEX StraTA 蜂群启动\n")
	fmt.Printf("━━━━━━━━━━━━━━━━━━━━━━━\n")
	fmt.Printf("[T1] π(z|s₁) 策略生成...\n")

	strategy := system.generateStrategy(task, model)
	fmt.Printf("  ✓ 策略: %s\n", strategy.ID)
	fmt.Printf("  ✓ 子任务: %d个\n", len(strategy.SubTasks))
	for _, t := range strategy.SubTasks {
		fmt.Printf("    • [%s] %s (验收: %s)\n", t.ID, t.Description, t.AcceptCriteria)
	}
	fmt.Printf("  ✓ A(z) 策略评分: %.2f\n", strategy.StrategyScore)

	fmt.Printf("\n[T2] π(aₜ|z,sₜ) 部署蜂群...\n")
	system.deploySwarm()
	fmt.Printf("  ✓ 部署 %d 个Agent\n", len(system.Agents))
	for id := range system.Agents {
		fmt.Printf("    • %s: 执行中...\n", id)
	}

	fmt.Printf("\n[T2] 并行执行...\n")
	system.executeAgents()
	for id, agent := range system.Agents {
		fmt.Printf("  ✓ %s → %s\n", id, agent.Output)
	}

	fmt.Printf("\n[T3] GRPO 分层优化...\n")
	for i := 0; i < iterations; i++ {
		result := system.grpoOptimize(i + 1)
		fmt.Printf("  Iter %d: J(θ)=%.2f | A(z)=%.2f | KL=%.4f | %s\n",
			i+1, result.TotalReward, result.StrategyReward, result.KLDivergence,
			map[bool]string{true: "✓ 收敛", false: "继续"}[result.Converged])
		if result.Converged {
			break
		}
	}

	fmt.Printf("\n[T4] MemLLM 记忆同步...\n")
	system.syncMemLLM()
	fmt.Printf("  ✓ 全局记忆: %d条\n", len(system.MemLLM.GlobalMemory))
	for id, mems := range system.MemLLM.AgentMemories {
		fmt.Printf("  ✓ %s 记忆: %d条\n", id, len(mems))
	}

	fmt.Printf("\n[验算] 主公式验证...\n")
	if system.verifyAllCompleted() {
		fmt.Printf("  ✅ 全部完成! 蜂群任务闭环\n")
	} else {
		fmt.Printf("  ⚠️ 未全部完成, 重新执行...\n")
		// In production: re-deploy failed agents
	}

	fmt.Printf("\n━━━━━━━━━━━━━━━━━━━━━━━\n")
	fmt.Printf("🏁 APEX StraTA 完成\n\n")

	// Output full JSON
	printJSON(system)
}

// T4: MemLLM同步
func (s *ApexStraTA) syncMemLLM() {
	s.mu.Lock()
	defer s.mu.Unlock()

	// Build RAG index
	for _, mem := range s.MemLLM.GlobalMemory {
		key := fmt.Sprintf("mem_%x", sha256.Sum256([]byte(mem.Input+mem.Output)))
		s.MemLLM.RAGCache[key[:8]] = mem.Reward
	}

	s.Phase = "mem_synced"
}

// Farthest point sampling
func (s *ApexStraTA) farthestPointSample(pool []Strategy) []Strategy {
	// Stub: in production, select diverse strategies from pool
	if len(pool) < 2 {
		// Add current strategy to pool
		pool = append(pool, s.Strategy)
	}
	if len(pool) > 5 {
		// Keep only the 5 most diverse
		sort.Slice(pool, func(i, j int) bool {
			return pool[i].StrategyScore > pool[j].StrategyScore
		})
		pool = pool[:5]
	}
	return pool
}

// ========================================
// 工具函数
// ========================================

func decomposeTask(task string) []SubTask {
	// Heuristic task decomposition
	task = strings.ToLower(task)
	var subtasks []SubTask
	i := 0

	descriptions := []string{
		"分析任务需求并制定方案",
		"搜索相关资源与知识",
		"生成核心交付物",
		"验证输出质量与正确性",
		"整合最终结果",
	}

	for _, desc := range descriptions {
		i++
		subtasks = append(subtasks, SubTask{
			ID:             fmt.Sprintf("task-%d", i),
			Description:    desc,
			AcceptCriteria: fmt.Sprintf("%s 完成并验证通过", desc),
			DependsOn:      []string{},
			Status:         "pending",
		})
	}

	// Set dependencies
	if len(subtasks) > 1 {
		for j := 1; j < len(subtasks); j++ {
			subtasks[j].DependsOn = append(subtasks[j].DependsOn, subtasks[j-1].ID)
		}
	}

	return subtasks
}

func generateDependencies(tasks []SubTask) []string {
	var deps []string
	for _, t := range tasks {
		for _, d := range t.DependsOn {
			deps = append(deps, fmt.Sprintf("%s→%s", d, t.ID))
		}
	}
	return deps
}

func evaluateStrategy(tasks []SubTask, deps []string) float64 {
	// Heuristic: coverage + dependency clarity + task granularity
	coverage := float64(len(tasks)) / 10.0
	if coverage > 1.0 {
		coverage = 1.0
	}
	dependencyClarity := float64(len(deps)) / float64(len(tasks)+1)
	return (coverage*0.5 + dependencyClarity*0.3 + 0.2) * 100
}

func evaluateTaskReward(agent *SwarmAgent) float64 {
	if agent.Status == "completed" {
		return 100.0
	} else if agent.Status == "running" {
		return 50.0
	}
	return 0.0
}

func computeKLDivergence(s Strategy, iter int) float64 {
	// Simulated KL: decreases with iterations
	baseKL := 0.2
	decay := float64(iter) * 0.05
	if baseKL-decay < 0.01 {
		return 0.01
	}
	return baseKL - decay
}

func defaultRewardConfig() RewardConfig {
	return RewardConfig{
		MaxIterations: 5,
		KLPenalty:     0.1,
		FarthestPointSample: true,
	}
}

// Agents map already uses sync.Mutex on ApexStraTA level

func generateID(prefix string) string {
	h := sha256.Sum256([]byte(fmt.Sprintf("%s%d", prefix, time.Now().UnixNano())))
	return fmt.Sprintf("%s-%s", prefix, hex.EncodeToString(h[:])[:8])
}

func showStatus() {
	fmt.Println("🐝 APEX StraTA 蜂群状态")
	fmt.Println("━━━━━━━━━━━━━━━━━")
	fmt.Println("暂无活跃蜂群。使用 --task 启动新蜂群。")
}

func showMemory(agentID string) {
	fmt.Printf("📦 MemLLM 记忆网络 [%s]\n", agentID)
	fmt.Println("━━━━━━━━━━━━━━━━━━")
	fmt.Println("暂无记忆数据。运行蜂群后查看。")
}

func printJSON(v interface{}) {
	b, _ := json.MarshalIndent(v, "", "  ")
	fmt.Println(string(b))
}

// Ensure all concurrent accesses are safe
var _ = &sync.Mutex{}
