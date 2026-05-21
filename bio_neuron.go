// bio_neuron.go — 类生物智能系统
// 基于生物神经元原理的智能个体模拟
//
// 生物机制映射:
//   神经元         → Gene (基因)
//   膜电位         → MembranePotential (激活阈值)
//   ATP能量        → ATP (运行能量)
//   神经递质       → Neurotransmitter (信号分子)
//   突触           → Synapse (基因连接)
//   细胞凋亡       → GeneApoptosis (基因死亡)
//   神经发生       → NeuroGenesis (新基因产生)
//   遗传变异       → GeneticVariation (DNA变异)

package main

import (
	"encoding/json"
	"fmt"
	"math"
	"math/rand"
	"net/http"
	"sort"
	"time"
)

// ============ 生物神经元基础结构 ============

// Neurotransmitter 神经递质类型
type Neurotransmitter string

const (
	Dopamine     Neurotransmitter = "dopamine"     // 多巴胺: 奖励/动机
	Serotonin    Neurotransmitter = "serotonin"    // 血清素: 情绪/压力
	Acetylcholine Neurotransmitter = "acetylcholine" // 乙酰胆碱: 学习/注意
	GABA         Neurotransmitter = "gaba"         // GABA: 抑制/平静
	Glutamate    Neurotransmitter = "glutamate"    // 谷氨酸: 兴奋/学习
	Norepinephrine Neurotransmitter = "norepinephrine" // 去甲肾上腺素: 警觉/应激
)

// MembranePotential 膜电位状态
type MembranePotential struct {
	Resting float64 // 静息电位 (-70mV 基准)
	Current float64 // 当前电位
	Threshold float64 // 阈值电位 (-55mV)
	Firing bool // 是否发放
}

// ATPEnergy ATP能量系统
type ATPEnergy struct {
	Current float64 // 当前ATP水平
	Max float64 // 最大ATP容量
	ConsumptionRate float64 // 消耗率
	RecoveryRate float64 // 恢复率
}

// BioGene 生物神经元版基因
type BioGene struct {
	ID string `json:"id"`
	Name string `json:"name"`

	// 膜电位系统
	MembranePotential *MembranePotential `json:"membrane_potential"`

	// ATP能量系统
	ATP *ATPEnergy `json:"atp"`

	// 神经递质敏感度
	NeurotransmitterSensitivity map[Neurotransmitter]float64 `json:"neurotransmitter_sensitivity"`

	// 树突 (输入连接)
	Dendrites []*Synapse `json:"dendrites"`

	// 轴突 (输出连接)
	AxonTerminals []*Synapse `json:"axon_terminals"`

	// 突触强度
	SynapticStrength float64 `json:"synaptic_strength"`

	// 基因信息
	GeneType string `json:"gene_type"` // axiom/emv/mutation/fusion
	SuccessRate float64 `json:"success_rate"`
	UsageCount int `json:"usage_count"`
	DeltaG float64 `json:"delta_g"`

	// 生命周期
	Age int `json:"age"` // 生存周期数
	Lifespan int `json:"lifespan"` // 寿命上限
	IsAlive bool `json:"is_alive"` // 是否存活

	// 遗传物质
	DNA *GeneDNA `json:"dna"` // 基因DNA

	// 状态
	 Inhibition float64 `json:"inhibition"` // 抑制程度
}

// GeneDNA 基因DNA — 存储基因的核心"遗传信息"
type GeneDNA struct {
	Chromosomes []string `json:"chromosomes"` // 染色体片段
	Sequence string `json:"sequence"` // 序列
	Mutations int `json:"mutations"` // 突变次数
}

// Synapse 突触 — 神经元间的连接
type Synapse struct {
	SourceID string `json:"source_id"` // 源基因ID
	TargetID string `json:"target_id"` // 目标基因ID
	Weight float64 `json:"weight"` // 突触权重
	Neurotransmitter Neurotransmitter `json:"neurotransmitter"` // 神经递质类型
	LastFired time.Time `json:"last_fired"`
	Plasticity float64 `json:"plasticity"` // 可塑性 (LTP/LTD)
}

// NeuralCircuit 神经回路 — 一组相连的基因
type NeuralCircuit struct {
	ID string `json:"id"`
	Name string `json:"name"`
	Genes []*BioGene `json:"genes"`
	Synapses []*Synapse `json:"synapses"`
	Type string `json:"type"` // sensory/motor/cognitive
}

// NeuralMicrocircuit 神经微回路 — 多个回路组成的系统
type NeuralMicrocircuit struct {
	Circuits map[string]*NeuralCircuit `json:"circuits"`
	GlobalATP *ATPEnergy `json:"global_atp"`
	TotalGenes int `json:"total_genes"`
}

// ============ 全局生物智能系统 ============

var (
	biocircuit *NeuralMicrocircuit
	evolutionGeneration int
	energyBudget float64
)

// initBioSystem 初始化生物智能系统
func initBioSystem() {
	biocircuit = &NeuralMicrocircuit{
		Circuits: make(map[string]*NeuralCircuit),
		GlobalATP: &ATPEnergy{
			Current: 1000,
			Max: 1000,
			ConsumptionRate: 1.0,
			RecoveryRate: 0.5,
		},
		TotalGenes: 0,
	}

	// 创建三个基础神经回路
	createCircuit("sensory", "感觉回路", 3)
	createCircuit("motor", "运动回路", 2)
	createCircuit("cognitive", "认知回路", 5)

	// 初始化axiom基因作为基础神经元
	initAxiomNeurons()

	energyBudget = 1000
	fmt.Println("[生物智能系统] 初始化完成")
	fmt.Printf("  - 神经回路: 3个 (sensory/motor/cognitive)\n")
	fmt.Printf("  - 基础神经元: axiom基因\n")
	fmt.Printf("  - 全局ATP: %.0f/%.0f\n", biocircuit.GlobalATP.Current, biocircuit.GlobalATP.Max)
}

// createCircuit 创建神经回路
func createCircuit(id, name string, geneCount int) {
	circuit := &NeuralCircuit{
		ID: id,
		Name: name,
		Genes: make([]*BioGene, 0),
		Synapses: make([]*Synapse, 0),
		Type: id,
	}
	biocircuit.Circuits[id] = circuit
}

// initAxiomNeurons 初始化axiom基因为神经元
func initAxiomNeurons() {
	axiomGenes := []struct{
		name string
		dna string
	}{
		{"keyword_expansion", "ATCGATCGATCG"},
		{"entity_tracing", "GCTAGCTAGCTA"},
		{"context_analysis", "TTAATTAATTAA"},
		{"pattern_recognition", "CGCGCGCGCGCA"},
		{"abstract_reasoning", "ATATATATATAT"},
	}

	for _, ag := range axiomGenes {
		gene := createBioGene(ag.name, "axiom", ag.dna)
		// axiom基因有更高的静息电位和ATP
		gene.MembranePotential.Resting = -70
		gene.MembranePotential.Threshold = -55
		gene.ATP.Max = 100
		gene.ATP.Current = 100
		gene.Lifespan = 10000
		gene.IsAlive = true

		// 添加到认知回路
		if circuit, ok := biocircuit.Circuits["cognitive"]; ok {
			circuit.Genes = append(circuit.Genes, gene)
		}
		biocircuit.TotalGenes++
	}
}

// createBioGene 创建生物神经元版基因
func createBioGene(name, geneType, dnaSequence string) *BioGene {
	id := fmt.Sprintf("neuron_%d_%s", time.Now().UnixNano(), name[:5])

	return &BioGene{
		ID: id,
		Name: name,
		MembranePotential: &MembranePotential{
			Resting: -70,
			Current: -70,
			Threshold: -55,
			Firing: false,
		},
		ATP: &ATPEnergy{
			Current: 50,
			Max: 50,
			ConsumptionRate: 0.5,
			RecoveryRate: 0.1,
		},
		NeurotransmitterSensitivity: map[Neurotransmitter]float64{
			Dopamine: 0.5,
			Serotonin: 0.5,
			Acetylcholine: 0.6,
			GABA: 0.3,
			Glutamate: 0.7,
			Norepinephrine: 0.4,
		},
		Dendrites: make([]*Synapse, 0),
		AxonTerminals: make([]*Synapse, 0),
		SynapticStrength: 0.5,
		GeneType: geneType,
		SuccessRate: 0.5,
		UsageCount: 0,
		DeltaG: 0,
		Age: 0,
		Lifespan: 1000,
		IsAlive: true,
		DNA: &GeneDNA{
			Chromosomes: []string{dnaSequence[:4], dnaSequence[4:8], dnaSequence[8:12]},
			Sequence: dnaSequence,
			Mutations: 0,
		},
	}
}

// ============ 膜电位系统 ============

// fireNeuron 神经元发放 — 类似于APEX的基因选择
func (g *BioGene) fireNeuron() bool {
	// 检查是否能发放
	if !g.IsAlive || g.ATP.Current <= 0 {
		return false
	}

	// 检查是否达到阈值
	if g.MembranePotential.Current >= g.MembranePotential.Threshold {
		g.MembranePotential.Firing = true

		// 消耗ATP
		g.ATP.Current -= g.ATP.ConsumptionRate * 10

		// 重置膜电位
		g.MembranePotential.Current = g.MembranePotential.Resting

		return true
	}

	g.MembranePotential.Firing = false
	return false
}

// receiveSignal 接收信号 — 类似于基因间协同
func (g *BioGene) receiveSignal(signal float64, nt Neurotransmitter) {
	if !g.IsAlive {
		return
	}

	// 神经递质调节
	sensitivity := g.NeurotransmitterSensitivity[nt]

	// 应用信号
	delta := signal * sensitivity
	g.MembranePotential.Current += delta

	// 衰减
	g.MembranePotential.Current *= 0.95

	// GABA抑制
	if nt == GABA {
		g.MembranePotential.Current -= 5
	}

	// 谷氨酸兴奋
	if nt == Glutamate {
		g.MembranePotential.Current += 3
	}
}

// updateMembranePotential 更新膜电位
func (g *BioGene) updateMembranePotential() {
	if !g.IsAlive {
		return
	}

	// 恢复到静息电位
	g.MembranePotential.Current += (g.MembranePotential.Resting - g.MembranePotential.Current) * 0.1

	// ATP不足时降低膜电位
	if g.ATP.Current < g.ATP.Max*0.2 {
		g.MembranePotential.Current -= 1
	}

	g.Age++
}

// ============ ATP能量代谢系统 ============

// consumeATP 消耗ATP
func (g *BioGene) consumeATP(amount float64) {
	g.ATP.Current = math.Max(0, g.ATP.Current-amount)
}

// recoverATP 恢复ATP
func (g *BioGene) recoverATP() {
	if !g.IsAlive {
		return
	}

	// 基础恢复
	g.ATP.Current = math.Min(g.ATP.Max, g.ATP.Current+g.ATP.RecoveryRate)

	// 过度使用惩罚
	if g.UsageCount > 100 {
		g.ATP.RecoveryRate *= 0.99
	}
}

// checkATPDepletion ATP耗竭检测
func (g *BioGene) checkATPDepletion() bool {
	return g.ATP.Current <= 0
}

// ============ 突触传导系统 ============

// createSynapse 创建突触连接
func createSynapse(source, target *BioGene, nt Neurotransmitter) *Synapse {
	synapse := &Synapse{
		SourceID: source.ID,
		TargetID: target.ID,
		Weight: 0.5,
		Neurotransmitter: nt,
		LastFired: time.Now(),
		Plasticity: 0.5,
	}

	// 添加到源和目标
	source.AxonTerminals = append(source.AxonTerminals, synapse)
	target.Dendrites = append(target.Dendrites, synapse)

	return synapse
}

// fireSynapse 突触传递信号
func fireSynapse(synapse *Synapse) float64 {
	// 计算信号强度
	signal := synapse.Weight * synapse.Plasticity

	// LTP: 如果最近频繁激活，增强突触
	if time.Since(synapse.LastFired) < time.Second*10 {
		synapse.Plasticity = math.Min(1.0, synapse.Plasticity*1.1)
	} else {
		// LTD: 长期不激活，减弱突触
		synapse.Plasticity *= 0.99
	}

	synapse.LastFired = time.Now()

	return signal
}

// applyHebbianLearning Hebb学习 — 一起激活的神经元连接加强
func applyHebbianLearning(gene1, gene2 *BioGene) {
	// 查找之间的突触
	for _, syn := range gene1.AxonTerminals {
		if syn.TargetID == gene2.ID {
			// 加强权重
			syn.Weight = math.Min(1.0, syn.Weight*1.1)
			fmt.Printf("[Hebb学习] %s → %s 权重: %.3f\n", gene1.Name, gene2.Name, syn.Weight)
			return
		}
	}

	// 如果没有突触，创建新的
	synapse := createSynapse(gene1, gene2, Glutamate)
	synapse.Weight = 0.6
}

// ============ 细胞凋亡与新生 ============

// apoptosis 细胞凋亡 — 基因死亡
func (g *BioGene) apoptosis() {
	if !g.IsAlive {
		return
	}

	// 凋亡条件
	shouldDie := false
	reason := ""

	if g.ATP.Current <= 0 {
		shouldDie = true
		reason = "ATP耗竭"
	} else if g.Age >= g.Lifespan {
		shouldDie = true
		reason = "寿命终结"
	} else if g.UsageCount > 1000 && g.SuccessRate < 0.3 {
		shouldDie = true
		reason = "低适应度淘汰"
	}

	if shouldDie {
		g.IsAlive = false
		g.MembranePotential.Firing = false
		fmt.Printf("[细胞凋亡] %s 死亡: %s (Age:%d, ATP:%.1f, SR:%.2f)\n",
			g.Name, reason, g.Age, g.ATP.Current, g.SuccessRate)
	}
}

// neuroGenesis 神经发生 — 新基因从现有基因分裂产生
func neuroGenesis(parent *BioGene, mutationRate float64) *BioGene {
	if !parent.IsAlive || parent.ATP.Current < parent.ATP.Max*0.5 {
		return nil
	}

	// 消耗能量
	parent.ATP.Current -= 20

	// 创建子基因
	childID := fmt.Sprintf("offspring_%d_%s", time.Now().UnixNano(), parent.Name[:3])
	child := &BioGene{
		ID: childID,
		Name: parent.Name + "_offspring",
		MembranePotential: &MembranePotential{
			Resting: parent.MembranePotential.Resting,
			Current: parent.MembranePotential.Resting,
			Threshold: parent.MembranePotential.Threshold,
			Firing: false,
		},
		ATP: &ATPEnergy{
			Current: parent.ATP.Current * 0.5,
			Max: parent.ATP.Max * 0.8,
			ConsumptionRate: parent.ATP.ConsumptionRate,
			RecoveryRate: parent.ATP.RecoveryRate,
		},
		NeurotransmitterSensitivity: parent.NeurotransmitterSensitivity,
		Dendrites: make([]*Synapse, 0),
		AxonTerminals: make([]*Synapse, 0),
		SynapticStrength: parent.SynapticStrength * 0.9,
		GeneType: "offspring",
		SuccessRate: parent.SuccessRate,
		UsageCount: 0,
		DeltaG: parent.DeltaG,
		Age: 0,
		Lifespan: parent.Lifespan,
		IsAlive: true,
	}

	// DNA复制
	child.DNA = &GeneDNA{
		Chromosomes: make([]string, len(parent.DNA.Chromosomes)),
		Sequence: parent.DNA.Sequence,
		Mutations: 0,
	}
	copy(child.DNA.Chromosomes, parent.DNA.Chromosomes)

	// 遗传变异
	if mutationRate > 0 && rand.Float64() < mutationRate {
		mutateDNA(child)
	}

	// 与父基因建立突触连接
	createSynapse(parent, child, Dopamine)

	fmt.Printf("[神经发生] %s → %s (ATP消耗:20)\n", parent.Name, child.Name)

	return child
}

// ============ 遗传变异系统 ============

// mutateDNA DNA突变
func mutateDNA(gene *BioGene) {
	if gene.DNA == nil {
		return
	}

	// 点突变
	dnaBytes := []byte(gene.DNA.Sequence)
	pos := rand.Intn(len(dnaBytes))
	original := dnaBytes[pos]

	// 随机替换
	newBase := []byte{'A', 'T', 'C', 'G'}[rand.Intn(4)]
	if newBase != original {
		dnaBytes[pos] = newBase
		gene.DNA.Sequence = string(dnaBytes)
		gene.DNA.Mutations++

		// 影响基因特性
		gene.SuccessRate = math.Max(0.1, math.Min(1.0, gene.SuccessRate+float64(rand.Intn(20)-10)/100))
		gene.SynapticStrength = math.Max(0.1, gene.SynapticStrength*0.95)

		fmt.Printf("[DNA突变] %s pos:%d %c→%c SR:%.3f\n", gene.Name, pos, original, newBase, gene.SuccessRate)
	}
}

// crossoverDNA DNA交叉
func crossoverDNA(parent1, parent2 *BioGene) (string, string) {
	if parent1.DNA == nil || parent2.DNA == nil {
		return parent1.DNA.Sequence, parent2.DNA.Sequence
	}

	seq1 := parent1.DNA.Sequence
	seq2 := parent2.DNA.Sequence

	// 单点交叉
	pos := len(seq1) / 2
	child1Seq := seq1[:pos] + seq2[pos:]
	child2Seq := seq2[:pos] + seq1[pos:]

	return child1Seq, child2Seq
}

// genePoolSelection 基因池选择 — 类比自然选择
func genePoolSelection(circuit *NeuralCircuit, survivalRate float64) {
	if len(circuit.Genes) <= 2 {
		return
	}

	// 按适应度排序
	sorted := make([]*BioGene, len(circuit.Genes))
	copy(sorted, circuit.Genes)
	sort.Slice(sorted, func(i, j int) bool {
		return sorted[i].SuccessRate > sorted[j].SuccessRate
	})

	// 淘汰低适应度
	survivorCount := int(float64(len(sorted)) * survivalRate)
	for i := survivorCount; i < len(sorted); i++ {
		sorted[i].apoptosis()
	}

	// 更新回路
	alive := make([]*BioGene, 0)
	for _, g := range sorted {
		if g.IsAlive {
			alive = append(alive, g)
		}
	}
	circuit.Genes = alive
}

// ============ 神经回路运行 ============

// runCircuit 运行神经回路一次迭代
func runCircuit(circuit *NeuralCircuit, stimulus float64) []*BioGene {
	if len(circuit.Genes) == 0 {
		return nil
	}

	// 1. 接收刺激
	for _, gene := range circuit.Genes {
		if !gene.IsAlive {
			continue
		}
		gene.receiveSignal(stimulus, Glutamate)
	}

	// 2. 神经回路内的信号传播
	for _, gene := range circuit.Genes {
		if !gene.IsAlive {
			continue
		}

		// 从树突接收信号
		for _, dendrite := range gene.Dendrites {
			signal := fireSynapse(dendrite)
			gene.receiveSignal(signal, dendrite.Neurotransmitter)
		}

		// 检查发放
		if gene.fireNeuron() {
			// 发放信号到轴突
			for _, axon := range gene.AxonTerminals {
				fireSynapse(axon)
			}

			// Hebb学习
			for _, other := range circuit.Genes {
				if other.ID != gene.ID && other.IsAlive {
					applyHebbianLearning(gene, other)
				}
			}

			gene.UsageCount++
		}
	}

	// 3. 更新状态
	for _, gene := range circuit.Genes {
		gene.updateMembranePotential()
		gene.recoverATP()
		gene.checkATPDepletion()
		gene.apoptosis()
	}

	// 4. 选择发放的神经元
	fired := make([]*BioGene, 0)
	for _, gene := range circuit.Genes {
		if gene.IsAlive && gene.MembranePotential.Firing {
			fired = append(fired, gene)
		}
	}

	return fired
}

// ============ 主循环 ============

// runBioEvolutionStep 运行一次生物进化步骤
func runBioEvolutionStep() {
	evolutionGeneration++
	fmt.Printf("\n========== 生物进化代: %d ==========\n", evolutionGeneration)

	// 全局能量恢复
	biocircuit.GlobalATP.Current = math.Min(
		biocircuit.GlobalATP.Max,
		biocircuit.GlobalATP.Current+biocircuit.GlobalATP.RecoveryRate*10,
	)

	fmt.Printf("全局ATP: %.1f/%.1f\n", biocircuit.GlobalATP.Current, biocircuit.GlobalATP.Max)

	// 运行每个回路
	totalFired := 0
	for id, circuit := range biocircuit.Circuits {
		// 刺激信号
		stimulus := 30.0 + rand.Float64()*20

		fired := runCircuit(circuit, stimulus)
		totalFired += len(fired)

		if len(fired) > 0 {
			fmt.Printf("[%s回路] 发放神经元: %d\n", id, len(fired))
			for _, g := range fired {
				fmt.Printf("  - %s (MP:%.1f, ATP:%.1f)\n", g.Name, g.MembranePotential.Current, g.ATP.Current)
			}
		}
	}

	// 神经发生 (如果能量充足)
	if biocircuit.GlobalATP.Current > 500 && len(biocircuit.Circuits["cognitive"].Genes) < 20 {
		if parent := biocircuit.Circuits["cognitive"].Genes[rand.Intn(len(biocircuit.Circuits["cognitive"].Genes))]; parent != nil {
			if child := neuroGenesis(parent, 0.1); child != nil {
				biocircuit.Circuits["cognitive"].Genes = append(biocircuit.Circuits["cognitive"].Genes, child)
				biocircuit.TotalGenes++
			}
		}
	}

	// 基因池选择
	for _, circuit := range biocircuit.Circuits {
		genePoolSelection(circuit, 0.7)
	}

	fmt.Printf("总神经元: %d\n", biocircuit.TotalGenes)
}

// ============ API接口 ============

type BioSelectRequest struct {
	Query string `json:"query"`
	UseBio bool `json:"use_bio"`
}

type BioSelectResponse struct {
	Winner *BioGene `json:"winner"`
	Circuit string `json:"circuit"`
	Generation int `json:"generation"`
	GlobalATP float64 `json:"global_atp"`
	AllNeurons int `json:"all_neurons"`
}

// bioSelectHandler 生物选择API
func bioSelectHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req BioSelectRequest
	json.NewDecoder(r.Body).Decode(&req)

	// 运行一次进化
	runBioEvolutionStep()

	// 选择胜者
	var winner *BioGene
	var circuitName string

	for id, circuit := range biocircuit.Circuits {
		for _, gene := range circuit.Genes {
			if gene.IsAlive && gene.MembranePotential.Firing {
				winner = gene
				circuitName = id
				break
			}
		}
		if winner != nil {
			break
		}
	}

	if winner == nil && len(biocircuit.Circuits["cognitive"].Genes) > 0 {
		// 选择适应度最高的
		best := biocircuit.Circuits["cognitive"].Genes[0]
		for _, g := range biocircuit.Circuits["cognitive"].Genes {
			if g.IsAlive && g.SuccessRate > best.SuccessRate {
				best = g
			}
		}
		winner = best
		circuitName = "cognitive"
	}

	resp := BioSelectResponse{
		Winner: winner,
		Circuit: circuitName,
		Generation: evolutionGeneration,
		GlobalATP: biocircuit.GlobalATP.Current,
		AllNeurons: biocircuit.TotalGenes,
	}

	json.NewEncoder(w).Encode(resp)
}

// bioStatsHandler 生物系统统计API
func bioStatsHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	stats := map[string]interface{}{
		"generation": evolutionGeneration,
		"global_atp": biocircuit.GlobalATP.Current,
		"total_neurons": biocircuit.TotalGenes,
		"circuits": map[string]int{},
	}

	for id, circuit := range biocircuit.Circuits {
		count := 0
		for _, g := range circuit.Genes {
			if g.IsAlive {
				count++
			}
		}
		stats["circuits"].(map[string]int)[id] = count
	}

	json.NewEncoder(w).Encode(stats)
}

// bioHealthHandler 健康检查
func bioHealthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status": "ok",
		"service": "bio_neuron",
		"generation": evolutionGeneration,
		"features": []string{
			"membrane_potential",
			"atp_energy",
			"neurotransmitter",
			"synaptic_plasticity",
			"apoptosis",
			"neurogenesis",
			"dna_mutation",
		},
	})
}

var bioMux *http.ServeMux

func mainBioServer() {
	initBioSystem()

	bioMux = http.NewServeMux()
	bioMux.HandleFunc("/bio/select", bioSelectHandler)
	bioMux.HandleFunc("/bio/stats", bioStatsHandler)
	bioMux.HandleFunc("/health", bioHealthHandler)

	fmt.Println("[生物智能系统] 服务启动在 :8093")
	http.ListenAndServe(":8093", bioMux)
}

// 生物智能系统入口
func main() {
	mainBioServer()
}
