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
	mu        sync.RWMutex
	core      *APEXCore
	history   []APEXCore
	devours   []DevourResult
	maxHist   int
}

// NewFormulaEngine 创建公式引擎
func NewFormulaEngine() *FormulaEngine {
	return &FormulaEngine{
		core: &APEXCore{
			T:             0,
			EV:            1.0,
			GeneCount:     0,
			Fitness:       0.5,
			DeltaG:        0,
			SpecConv:      0.5,
			Discipline:    0.5,
			CollabEntropy: 0.5,
			HashChain:     "genesis",
		},
		history: make([]APEXCore, 0),
		devours: make([]DevourResult, 0),
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
		fe.core.GeneCount++
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
func (fe *FormulaEngine) applyEntropyTriangleUnsafe() EntropyTriangle {
	tri := EntropyTriangle{
		SpecConvergence:    fe.computeSpecConvergence(),
		DisciplineLock:     fe.computeDisciplineLock(),
		CollaborativeReduce: fe.computeCollabEntropy(),
	}

	// EMA平滑: α=0.3, 新值权重30%, 旧值权重70%
	// 防止奇偶轮ΔG剧烈振荡(原版直接覆写→振荡)
	alpha := 0.3
	fe.core.SpecConv = alpha*tri.SpecConvergence + (1-alpha)*fe.core.SpecConv
	fe.core.Discipline = alpha*tri.DisciplineLock + (1-alpha)*fe.core.Discipline
	fe.core.CollabEntropy = alpha*tri.CollaborativeReduce + (1-alpha)*fe.core.CollabEntropy

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

	// 5. [BUG#2修复] 应用三重熵减约束 — 原版从未调用!
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

// FormulaEngine 作为库使用，main在apexagi_orchestrator.go中
