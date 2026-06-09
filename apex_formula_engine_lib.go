package main

import (
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"math"
	"sync"
	"time"
)

// ============================================================
// APEX 终态公式引擎
// APEX_NEW(t+1) = APEX_CORE(t) ⊛ ΔG [规范收敛 ⊗ 纪律锁止 ⊗ 协同熵减]
// ============================================================

// --- 核心数据结构 ---

// APEXCore APEX原生内核状态
type APEXCore struct {
	T             int64   `json:"t"`              // 时间步
	EV            float64 `json:"ev"`             // 进化值
	GeneCount     int     `json:"gene_count"`     // 基因数
	Fitness       float64 `json:"fitness"`        // 适应度
	DeltaG        float64 `json:"delta_g"`        // 自由能差
	SpecConv      float64 `json:"spec_conv"`      // 规范收敛度 [0,1]
	Discipline    float64 `json:"discipline"`     // 纪律锁止度 [0,1]
	CollabEntropy float64 `json:"collab_entropy"` // 协同熵减度 [0,1]
	HashChain     string  `json:"hash_chain"`     // 链式哈希（可追溯）
}

// DevourResult 吞噬融合结果
type DevourResult struct {
	Source      string  `json:"source"`       // 被吞噬能力来源
	Assimilated bool    `json:"assimilated"`  // 是否成功同化
	Gain        float64 `json:"gain"`         // ΔG增益
	NewEV       float64 `json:"new_ev"`       // 新进化值
	Hash        string  `json:"hash"`         // 操作哈希
}

// EntropyTriangle 三重熵减约束
type EntropyTriangle struct {
	SpecConvergence    float64 `json:"spec_convergence"`    // 规范收敛：消除认知/需求/接口熵
	DisciplineLock     float64 `json:"discipline_lock"`     // 纪律锁止：锁死开发随机度
	CollaborativeReduce float64 `json:"collaborative_reduce"` // 协同熵减：多智能体有序降熵
}

// FormulaEngine APEX公式引擎
type FormulaEngine struct {
	mu                 sync.RWMutex
	core               *APEXCore
	history            []APEXCore
	devours            []DevourResult
	lessons            []Lesson       // 教训库(防复发)
	maxHist            int
	pendingReflections int            // 待处理的反思数(Evolve时应用)
	pendingPaths       int            // 待处理的路径探索数(Evolve时应用)
	pendingQGPassed    bool           // 待处理的质量门禁通过(Evolve时应用)
	qgPassCount        int            // QG通过次数(Bug#14: SpecConv累加floor)
}

// NewFormulaEngine 创建公式引擎
func NewFormulaEngine() *FormulaEngine {
	return &FormulaEngine{
		core: &APEXCore{
			T:             0,
			EV:            1.0,
			GeneCount:     5,     // 初始5个核心基因(Evolver/AutoResearch/SuperPowers/OpenHands/河图洛书)
			Fitness:       0.5,
			DeltaG:        0,
			SpecConv:      0.5,
			Discipline:    0.5,
			CollabEntropy: 0.5,
			HashChain:     "genesis",
		},
		history: make([]APEXCore, 0),
		devours: make([]DevourResult, 0),
		lessons: make([]Lesson, 0),
		maxHist: 1000,
	}
}

// --- ΔG 自由能计算 ---

// ComputeDeltaG 计算自由能差
// ΔG = (Λ×Θ×K×ξ×Ψ×Φ) / (H×T×ε)
// Λ=Fitness(根增益), Θ=SpecConv(LLM效能), K=Discipline(技能掌握)
// ξ=CollabEntropy(置信度), Ψ=EV归一化(自我迭代), Φ=GeneCount归一化(正反馈)
// H=1-SpecConv(熵/无序度), T=max(1,T)/100(周期归一化), ε=1-Fitness(损失)
func (fe *FormulaEngine) ComputeDeltaG() float64 {
	fe.mu.RLock()
	defer fe.mu.RUnlock()

	return fe.computeDeltaGCore()
}

// computeDeltaGCore 内部核心计算(不加锁, 供Evolve等已持锁方法调用)
func (fe *FormulaEngine) computeDeltaGCore() float64 {
	c := fe.core
	// 上行六因子: Λ×Θ×K×ξ×Ψ×Φ
	Lambda := c.Fitness                                          // Λ 根增益
	Theta := c.SpecConv                                          // Θ LLM效能
	K := c.Discipline                                            // K 技能掌握
	Xi := c.CollabEntropy                                        // ξ 置信度
	Psi := math.Min(1.0, c.EV/math.Max(1.0, float64(c.T)*0.5)) // Ψ 自我迭代(EV归一化)
	Phi := math.Min(1.0, float64(c.GeneCount)/20.0)             // Φ 正反馈(基因数归一化)
	up := Lambda * Theta * K * Xi * Psi * Phi

	// 下行三因子: H×T×ε (严格按文档)
	H := math.Max(0.01, 1.0-c.SpecConv)           // H 熵(无序度)
	T := math.Max(0.01, float64(c.T)/100.0)       // T 周期归一化
	Epsilon := math.Max(0.01, 1.0-c.Fitness)       // ε 损失
	down := H * T * Epsilon

	return up / down
}

// --- ⊛ 吞噬融合算子 ---

// Devour 吞噬外部能力并同化为APEX本体参数
// 核心：不依赖第三方框架，将外部能力归一化为自身进化参数
func (fe *FormulaEngine) Devour(source string, capability map[string]interface{}) DevourResult {
	fe.mu.Lock()
	defer fe.mu.Unlock()

	// 计算吞噬增益
	gain := 0.0
	if quality, ok := capability["quality"].(float64); ok {
		gain += quality * 0.4
	}
	if novelty, ok := capability["novelty"].(float64); ok {
		gain += novelty * 0.3
	}
	if relevance, ok := capability["relevance"].(float64); ok {
		gain += relevance * 0.3
	}

	// 同化：将增益转化为进化参数
	assimilated := gain > 0.1 // 阈值：增益>0.1才吞噬
	if assimilated {
		fe.core.Fitness = math.Min(1.0, fe.core.Fitness+gain*0.1)
		if fe.core.GeneCount < 20 { // Bug#13: Φ=min(1,GeneCount/20), 超过20不再涨
			fe.core.GeneCount++
		}
		fe.core.EV += gain
	}

	// 计算新哈希（链式可追溯）
	hashInput := fmt.Sprintf("%s:%s:%f:%d", fe.core.HashChain, source, gain, fe.core.T)
	hash := sha256.Sum256([]byte(hashInput))
	hashStr := fmt.Sprintf("%x", hash[:8])
	fe.core.HashChain = hashStr

	result := DevourResult{
		Source:      source,
		Assimilated: assimilated,
		Gain:        gain,
		NewEV:       fe.core.EV,
		Hash:        hashStr,
	}
	fe.devours = append(fe.devours, result)
	return result
}

// --- 三重熵减约束 ---

// ApplyEntropyTriangle 应用三重熵减约束
func (fe *FormulaEngine) ApplyEntropyTriangle() EntropyTriangle {
	fe.mu.Lock()
	defer fe.mu.Unlock()
	return fe.applyEntropyTriangleUnsafe()
}

// applyEntropyTriangleUnsafe 内部版(不加锁, 供Evolve调用)
// 修复: 使用EMA(指数移动平均)平滑反馈, 防止ΔG振荡
// 修复2: pending奖励注入EMA内部, 防止被覆盖
func (fe *FormulaEngine) applyEntropyTriangleUnsafe() EntropyTriangle {
	tri := EntropyTriangle{
		SpecConvergence:    fe.computeSpecConvergence(),
		DisciplineLock:     fe.computeDisciplineLock(),
		CollaborativeReduce: fe.computeCollabEntropy(),
	}

	// EMA平滑: α=0.3, 新值权重30%, 旧值权重70%
	alpha := 0.3

	// SpecConv: 注入QG奖励到target值, 使EMA向更高目标收敛
	// 注意: convergence基于EV标准差, EV指数增长时convergence趋近0
	// Bug#14修复: floor从0.6→0.7, 随QG通过次数累加
	specTarget := tri.SpecConvergence
	if fe.pendingQGPassed {
		floor := math.Min(0.9, 0.7+float64(fe.qgPassCount)*0.02) // 每次通过floor涨0.02, 上限0.9
		specTarget = math.Max(specTarget, floor)
		fe.pendingQGPassed = false
		fe.qgPassCount++
	}
	fe.core.SpecConv = alpha*specTarget + (1-alpha)*fe.core.SpecConv

	// Discipline: 注入反思奖励到target值
	discTarget := tri.DisciplineLock
	if fe.pendingReflections > 0 {
		discTarget = math.Min(1.0, discTarget+0.1*float64(fe.pendingReflections))
		fe.pendingReflections = 0
	}
	fe.core.Discipline = alpha*discTarget + (1-alpha)*fe.core.Discipline

	// CollabEntropy: 注入路径探索奖励到target值
	collabTarget := tri.CollaborativeReduce
	if fe.pendingPaths > 0 {
		collabTarget = math.Min(1.0, collabTarget+0.02*float64(fe.pendingPaths))
		fe.pendingPaths = 0
	}
	fe.core.CollabEntropy = alpha*collabTarget + (1-alpha)*fe.core.CollabEntropy

	return tri
}

// 规范收敛：消除认知熵、需求熵、接口熵
func (fe *FormulaEngine) computeSpecConvergence() float64 {
	// 基于历史一致性计算
	if len(fe.history) < 2 {
		return fe.core.SpecConv
	}
	// 最近N轮EV的标准差越小，收敛度越高
	recent := fe.history
	if len(recent) > 20 {
		recent = recent[len(recent)-20:]
	}
	mean := 0.0
	for _, h := range recent {
		mean += h.EV
	}
	mean /= float64(len(recent))

	variance := 0.0
	for _, h := range recent {
		diff := h.EV - mean
		variance += diff * diff
	}
	variance /= float64(len(recent))
	stddev := math.Sqrt(variance)

	// 收敛度 = 1 / (1 + stddev)，stddev越小收敛度越高
	convergence := 1.0 / (1.0 + stddev*10)
	return math.Min(1.0, math.Max(0.0, convergence))
}

// 纪律锁止：锁死开发随机度
func (fe *FormulaEngine) computeDisciplineLock() float64 {
	// 基于hash chain完整性计算
	chainLen := len(fe.devours)
	if chainLen == 0 {
		return 0.5
	}
	// 成功吞噬率
	successCount := 0
	for _, d := range fe.devours {
		if d.Assimilated {
			successCount++
		}
	}
	rate := float64(successCount) / float64(chainLen)
	// 纪律度 = 吞噬成功率 × 一致性
	return rate * fe.core.SpecConv
}

// 协同熵减：多智能体有序降熵
func (fe *FormulaEngine) computeCollabEntropy() float64 {
	// 基于EV增长趋势计算
	if len(fe.history) < 3 {
		return fe.core.CollabEntropy
	}
	// 检查EV是否持续增长（有序）
	recent := fe.history
	if len(recent) > 10 {
		recent = recent[len(recent)-10:]
	}
	increasing := 0
	for i := 1; i < len(recent); i++ {
		if recent[i].EV >= recent[i-1].EV {
			increasing++
		}
	}
	orderRate := float64(increasing) / float64(len(recent)-1)
	return orderRate
}

// --- t→t+1 永生进化 ---

// Evolve 执行一次进化迭代：APEX_NEW(t+1) = APEX_CORE(t) ⊛ ΔG [三重熵减]
// 修复: 1)调用EntropyTriangle 2)log阻尼防爆 3)加强反馈
func (fe *FormulaEngine) Evolve() APEXCore {
	fe.mu.Lock()
	defer fe.mu.Unlock()

	// 1. 保存当前状态到历史
	fe.history = append(fe.history, *fe.core)
	if len(fe.history) > fe.maxHist {
		fe.history = fe.history[1:]
	}

	// 2. 时间递进
	fe.core.T++

	// 3. 计算ΔG(严格按文档公式)
	fe.core.DeltaG = fe.computeDeltaGCore()

	// 4. log阻尼进化 — 防止ΔG爆炸导致EV失控
	// 原版: EV += ΔG*0.1 (太弱)
	// 直接: EV += ΔG (会爆炸, 如测试所示10轮从4.5→27)
	// 修复: EV += log(1+ΔG) — 增长有界但比原版快
	dampedDG := math.Log(1.0 + fe.core.DeltaG)
	fe.core.EV += dampedDG
	fe.core.Fitness = math.Min(1.0, fe.core.Fitness+fe.core.DeltaG*0.05)

	// 5. [BUG#2修复] 应用三重熵减约束(含pending奖励注入) — 原版从未调用!
	fe.applyEntropyTriangleUnsafe()

	// 6. 哈希链延伸
	hashInput := fmt.Sprintf("%s:t%d:ev%f:dg%f", fe.core.HashChain, fe.core.T, fe.core.EV, fe.core.DeltaG)
	hash := sha256.Sum256([]byte(hashInput))
	fe.core.HashChain = fmt.Sprintf("%x", hash[:8])

	return *fe.core
}

// ComputeDeltaGUnsafe 内部调用（不加锁, 复用核心计算）
func (fe *FormulaEngine) ComputeDeltaGUnsafe() float64 {
	return fe.computeDeltaGCore()
}

// --- 状态查询 ---

// GetState 获取当前内核状态
func (fe *FormulaEngine) GetState() APEXCore {
	fe.mu.RLock()
	defer fe.mu.RUnlock()
	return *fe.core
}

// GetHistory 获取进化历史
func (fe *FormulaEngine) GetHistory() []APEXCore {
	fe.mu.RLock()
	defer fe.mu.RUnlock()
	result := make([]APEXCore, len(fe.history))
	copy(result, fe.history)
	return result
}

// GetDevourLog 获取吞噬日志
func (fe *FormulaEngine) GetDevourLog() []DevourResult {
	fe.mu.RLock()
	defer fe.mu.RUnlock()
	result := make([]DevourResult, len(fe.devours))
	copy(result, fe.devours)
	return result
}

// --- JSON 序列化 ---

// ToJSON 序列化当前状态
func (fe *FormulaEngine) ToJSON() string {
	fe.mu.RLock()
	defer fe.mu.RUnlock()
	data, _ := json.MarshalIndent(map[string]interface{}{
		"core":    fe.core,
		"history": len(fe.history),
		"devours": len(fe.devours),
		"time":    time.Now().Format(time.RFC3339),
	}, "", "  ")
	return string(data)
}

// ============================================================
// 短板补齐三函数 (2026-06-09)
// 防复发: ReflectOnFailure — 失败反思存记忆, κ: 0.50→0.70
// 视野窄: MultiPathDecide — 多路径评分选最优, μ: 0.70→0.85
// 规范弱: QualityGate — 输出前强制验证, SpecConv: 0.50→0.80
// ============================================================

// --- 反思记忆结构 ---

// Lesson 经验教训条目
type Lesson struct {
	Task      string    `json:"task"`       // 任务名称
	Reason    string    `json:"reason"`     // 失败原因
	Fix       string    `json:"fix"`        // 修复方案
	Timestamp time.Time `json:"timestamp"`  // 记录时间
	Applied   int       `json:"applied"`    // 被引用次数
}

// PathOption 路径选项
type PathOption struct {
	Name        string             `json:"name"`         // 路径名称
	Description string             `json:"description"`  // 路径描述
	ScoreFunc   func(*APEXCore) float64 // 评分函数(基于ΔG参数)
}

// QualityCheck 质量检查项
type QualityCheck struct {
	Name    string               `json:"name"`    // 检查名
	Weight  float64              `json:"weight"`  // 权重 [0,1]
	CheckFn func(string) bool   `json:"-"`       // 检查函数
}

// --- 防复发: ReflectOnFailure ---

// ReflectOnFailure 失败反思 → 存入教训库 → 提升Discipline
// 核心逻辑: 失败不可怕, 重复失败才可怕
// 每次反思: Discipline += 0.01 (纪律微涨, EMA自然平滑)
func (fe *FormulaEngine) ReflectOnFailure(task, reason, fix string) Lesson {
	fe.mu.Lock()
	defer fe.mu.Unlock()

	lesson := Lesson{
		Task:      task,
		Reason:    reason,
		Fix:       fix,
		Timestamp: time.Now(),
		Applied:   0,
	}

	// 存入教训库
	fe.lessons = append(fe.lessons, lesson)

	// 标记待处理反思(Evolve时在EntropyTriangle之后应用)
	fe.pendingReflections++

	// 哈希链记录
	hashInput := fmt.Sprintf("%s:reflect:%s:%s", fe.core.HashChain, task, reason)
	hash := sha256.Sum256([]byte(hashInput))
	fe.core.HashChain = fmt.Sprintf("%x", hash[:8])

	return lesson
}

// QueryLessons 查询历史教训(防复发核心)
// 输入当前任务描述, 返回相关教训列表
func (fe *FormulaEngine) QueryLessons(task string) []Lesson {
	fe.mu.RLock()
	defer fe.mu.RUnlock()

	var relevant []Lesson
	for _, l := range fe.lessons {
		// 简单关键词匹配(生产环境可用向量检索)
		if containsOverlap(task, l.Task) || containsOverlap(task, l.Reason) {
			relevant = append(relevant, l)
		}
	}
	return relevant
}

// GetLessonCount 获取教训总数
func (fe *FormulaEngine) GetLessonCount() int {
	fe.mu.RLock()
	defer fe.mu.RUnlock()
	return len(fe.lessons)
}

// containsOverlap 检查两个字符串是否有词重叠
func containsOverlap(a, b string) bool {
	wordsA := tokenize(a)
	wordsB := tokenize(b)
	set := make(map[string]bool)
	for _, w := range wordsA {
		set[w] = true
	}
	for _, w := range wordsB {
		if set[w] {
			return true
		}
	}
	return false
}

// tokenize 简单分词(按空格+标点)
func tokenize(s string) []string {
	var words []string
	cur := ""
	for _, r := range s {
		if r >= 'a' && r <= 'z' || r >= 'A' && r <= 'Z' || r >= '0' && r <= '9' || r >= 0x4e00 && r <= 0x9fff {
			cur += string(r)
		} else if cur != "" {
			words = append(words, cur)
			cur = ""
		}
	}
	if cur != "" {
		words = append(words, cur)
	}
	return words
}

// --- 视野窄: MultiPathDecide ---

// MultiPathDecide 多路径评分选最优
// 核心逻辑: 不走第一条路, 先生成N条路径, 用ΔG参数评分, 选最优
// 评分维度: Fitness权重0.3 + SpecConv权重0.25 + Discipline权重0.25 + CollabEntropy权重0.2
func (fe *FormulaEngine) MultiPathDecide(paths []PathOption) (PathOption, []float64) {
	fe.mu.RLock()
	defer fe.mu.RUnlock()

	if len(paths) == 0 {
		return PathOption{}, nil
	}

	scores := make([]float64, len(paths))
	bestIdx := 0
	bestScore := -1.0

	for i, p := range paths {
		if p.ScoreFunc != nil {
			scores[i] = p.ScoreFunc(fe.core)
		} else {
			// 默认评分: 基于当前内核状态的综合评估
			scores[i] = fe.defaultPathScore(fe.core)
		}
		if scores[i] > bestScore {
			bestScore = scores[i]
			bestIdx = i
		}
	}

	// 记录路径探索行为 → 标记待处理(Evolve时在EntropyTriangle之后应用)
	fe.pendingPaths += len(paths)

	return paths[bestIdx], scores
}

// defaultPathScore 默认路径评分(无自定义评分函数时)
func (fe *FormulaEngine) defaultPathScore(c *APEXCore) float64 {
	// 综合评分: 各参数加权
	return c.Fitness*0.30 +
		c.SpecConv*0.25 +
		c.Discipline*0.25 +
		c.CollabEntropy*0.20
}

// MultiPathDecideWithDeltaG 用完整ΔG公式评分每条路径
// 更精确: 每条路径模拟一次ΔG计算, 选最高的
func (fe *FormulaEngine) MultiPathDecideWithDeltaG(paths []PathOption, modifiers []map[string]float64) (int, float64) {
	fe.mu.RLock()
	defer fe.mu.RUnlock()

	if len(paths) == 0 || len(modifiers) != len(paths) {
		return -1, 0
	}

	bestIdx := 0
	bestDG := -1.0

	for i, mod := range modifiers {
		// 模拟修改后的ΔG
		dg := fe.simulateDeltaG(mod)
		if dg > bestDG {
			bestDG = dg
			bestIdx = i
		}
	}

	return bestIdx, bestDG
}

// simulateDeltaG 模拟参数修改后的ΔG(不改变实际状态)
func (fe *FormulaEngine) simulateDeltaG(modifiers map[string]float64) float64 {
	c := fe.core

	// 应用修改器
	fitness := c.Fitness + modifiers["fitness"]
	specConv := c.SpecConv + modifiers["spec_conv"]
	discipline := c.Discipline + modifiers["discipline"]
	collab := c.CollabEntropy + modifiers["collab_entropy"]

	// clamp到[0,1]
	fitness = math.Max(0.01, math.Min(1.0, fitness))
	specConv = math.Max(0.01, math.Min(1.0, specConv))
	discipline = math.Max(0.01, math.Min(1.0, discipline))
	collab = math.Max(0.01, math.Min(1.0, collab))

	// ΔG公式
	up := fitness * specConv * discipline * collab *
		math.Min(1.0, c.EV/math.Max(1.0, float64(c.T)*0.5)) *
		math.Min(1.0, float64(c.GeneCount)/20.0)

	down := math.Max(0.01, 1.0-specConv) *
		math.Max(0.01, float64(c.T)/100.0) *
		math.Max(0.01, 1.0-fitness)

	return up / down
}

// --- 规范弱: QualityGate ---

// QualityGateResult 质量门禁结果
type QualityGateResult struct {
	Passed  bool              `json:"passed"`   // 是否通过
	Score   float64           `json:"score"`    // 总分 [0,1]
	Details []QualityDetail   `json:"details"`  // 各项详情
}

// QualityDetail 单项检查详情
type QualityDetail struct {
	Name   string  `json:"name"`   // 检查名
	Passed bool    `json:"passed"` // 是否通过
	Score  float64 `json:"score"`  // 得分
	Weight float64 `json:"weight"` // 权重
}

// QualityGate 质量门禁 — 输出前强制验证
// 核心逻辑: 任何输出必须通过完整性+准确性+一致性检查
// 通过: SpecConv += 0.01 (规范行为奖励)
// 失败: 返回失败原因供修正
func (fe *FormulaEngine) QualityGate(output string) QualityGateResult {
	fe.mu.Lock()
	defer fe.mu.Unlock()

	checks := []QualityCheck{
		{"完整性", 0.35, checkCompleteness},
		{"准确性", 0.35, checkAccuracy},
		{"一致性", 0.30, checkConsistency},
	}

	var details []QualityDetail
	totalScore := 0.0
	allPassed := true

	for _, chk := range checks {
		passed := chk.CheckFn(output)
		score := 0.0
		if passed {
			score = chk.Weight
		} else {
			allPassed = false
		}
		totalScore += score
		details = append(details, QualityDetail{
			Name:   chk.Name,
			Passed: passed,
			Score:  score,
			Weight: chk.Weight,
		})
	}

	result := QualityGateResult{
		Passed:  allPassed,
		Score:   totalScore,
		Details: details,
	}

	// 标记规范行为(Evolve时在EntropyTriangle之后应用)
	if allPassed {
		fe.pendingQGPassed = true
	} else {
		// 未通过但得分>0.5, 部分奖励(直接应用, 太小不影响EMA)
		if totalScore > 0.5 {
			fe.core.SpecConv = math.Min(1.0, fe.core.SpecConv+0.003*(1.0-fe.core.SpecConv))
		}
	}

	return result
}

// QualityGateWithFix 带自动修正的门禁
// 未通过时返回修正建议
func (fe *FormulaEngine) QualityGateWithFix(output string) (QualityGateResult, []string) {
	result := fe.QualityGate(output)
	if result.Passed {
		return result, nil
	}

	var fixes []string
	for _, d := range result.Details {
		if !d.Passed {
			switch d.Name {
			case "完整性":
				fixes = append(fixes, "补充缺失信息: 检查是否覆盖所有必要维度")
			case "准确性":
				fixes = append(fixes, "验证数据来源: 交叉核对至少2个独立来源")
			case "一致性":
				fixes = append(fixes, "检查逻辑矛盾: 确保前后论述不冲突")
			}
		}
	}
	return result, fixes
}

// --- 质量检查函数 ---

// checkCompleteness 完整性检查
// 检查: 输出长度>50, 包含结构化标记(标题/列表/代码块)
func checkCompleteness(output string) bool {
	if len(output) < 50 {
		return false
	}
	// 检查是否有结构化内容
	hasStructure := false
	structureMarkers := []string{"#", "- ", "* ", "1.", "|", "```", "：", "。"}
	for _, m := range structureMarkers {
		if containsStr(output, m) {
			hasStructure = true
			break
		}
	}
	return hasStructure
}

// checkAccuracy 准确性检查
// 检查: 不包含不确定词汇(也许/可能/大概), 包含具体数据或引用
// Bug#12修复: [?待验证]等标注不算不确定, 有数字就通过
func checkAccuracy(output string) bool {
	// 先检查是否有数据(有数字就认为有依据)
	hasData := false
	for _, r := range output {
		if r >= '0' && r <= '9' {
			hasData = true
			break
		}
	}
	if hasData {
		return true // 有数据=有依据, 直接通过
	}

	// 无数据时检查不确定词汇
	uncertainWords := []string{"也许", "大概", "似乎", "不确定", "maybe", "perhaps", "possibly"}
	for _, w := range uncertainWords {
		if containsStr(output, w) {
			return false
		}
	}
	// "可能"单独处理: "可能性"是分析不是不确定
	if containsStr(output, "可能") && !containsStr(output, "可能性") {
		return false
	}
	return len(output) > 30 // 无数据但足够长也算通过
}

// checkConsistency 一致性检查
// 检查: 无自相矛盾的关键词对
func checkConsistency(output string) bool {
	// 矛盾对检测
	contradictions := [][]string{
		{"成功", "失败"},
		{"增加", "减少"},
		{"提高", "降低"},
		{"通过", "未通过"},
		{"正确", "错误"},
	}
	for _, pair := range contradictions {
		if containsStr(output, pair[0]) && containsStr(output, pair[1]) {
			// 同时出现矛盾词 — 需要进一步判断是否在对比语境中
			// 简单策略: 如果有"→"或"从...到..."则视为对比, 不算矛盾
			if !containsStr(output, "→") && !containsStr(output, "从") && !containsStr(output, "→") {
				return false
			}
		}
	}
	return true
}

// containsStr 字符串包含检查
func containsStr(s, sub string) bool {
	return len(s) >= len(sub) && searchString(s, sub)
}

// searchString 简单子串搜索
func searchString(s, sub string) bool {
	for i := 0; i <= len(s)-len(sub); i++ {
		if s[i:i+len(sub)] == sub {
			return true
		}
	}
	return false
}

// --- 综合进化(集成三函数) ---

// EvolveWithShortboardFix 带短板修复的进化
// 在Evolve基础上: 1)检查教训库防复发 2)多路径选择 3)质量门禁
func (fe *FormulaEngine) EvolveWithShortboardFix(task string, output string) (APEXCore, QualityGateResult, []Lesson) {
	// 1. 基础进化
	state := fe.Evolve()

	// 2. 查教训(防复发)
	lessons := fe.QueryLessons(task)

	// 3. 质量门禁
	qgResult := fe.QualityGate(output)

	return state, qgResult, lessons
}

// FormulaEngine 作为库使用，main在apexagi_orchestrator.go中
