package main

import (
	"crypto/sha256"
	"encoding/json"
	"flag"
	"fmt"
	"math"
	"math/rand"
	"os"
	"path/filepath"
	"sort"
	"time"
)

// ──────────────────────────────────────────────────────────────
// Core Data Types
// ──────────────────────────────────────────────────────────────

type TrajectoryStep struct {
	Step            int     `json:"step"`
	From            string  `json:"from"`
	To              string  `json:"to"`
	Reward          float64 `json:"reward"`
	Cost            float64 `json:"cost"`
	EpsilonGreedy   bool    `json:"epsilon_greedy"`
	ExplorationRate float64 `json:"exploration_rate"`
}

type Trajectory struct {
	ID          string           `json:"id"`
	Steps       []TrajectoryStep `json:"steps"`
	TotalReward float64          `json:"total_reward"`
	TotalCost   float64          `json:"total_cost"`
	PeakCount   int              `json:"peak_count"`
}

type CollapseReport struct {
	Collapsed     bool     `json:"collapsed"`
	CollapsedKeys []string `json:"collapsed_keys,omitempty"`
	Symptoms      []string `json:"symptoms,omitempty"`
	Severity      float64  `json:"severity"`
}

type MultiPeakRoute struct {
	RouteID       string   `json:"route_id"`
	PrimaryPath   []string `json:"primary_path"`
	RedundantPath []string `json:"redundant_path"`
	Diversity     float64  `json:"diversity"`
	Overlap       float64  `json:"overlap"`
}

type EvolvedSkill struct {
	SkillID        string  `json:"skill_id"`
	OldRewardBase  float64 `json:"old_reward_base"`
	NewRewardBase  float64 `json:"new_reward_base"`
	Adjustment     float64 `json:"adjustment"`
	Direction      string  `json:"direction"`
	Alpha          float64 `json:"alpha"`
	ReversePolicyR float64 `json:"reverse_policy_r"`
}

type Report struct {
	ID                string                   `json:"id"`
	DAGNodes          int                      `json:"dag_nodes"`
	DatasetsLoaded    int                      `json:"datasets_loaded"`
	RoutedTrajectories []Trajectory            `json:"routed_trajectories"`
	CreditAllocations map[string]float64       `json:"credit_allocations"`
	CollapseDetected  bool                     `json:"collapse_detected"`
	Collapse          *CollapseReport          `json:"collapse,omitempty"`
	MultiPeakRedundant []MultiPeakRoute        `json:"multi_peak_redundant"`
	EvolvedSkills     []EvolvedSkill           `json:"evolved_skills"`
	Format            string                   `json:"format"`
	StartedAt         string                   `json:"started_at"`
	DurationMs        int64                    `json:"duration_ms"`
	TrajectoryStats   map[string]interface{}   `json:"trajectory_stats,omitempty"`
}

// ──────────────────────────────────────────────────────────────
// DAG types
// ──────────────────────────────────────────────────────────────

type DAG struct {
	Format        string        `json:"format"`
	SchemaVersion string        `json:"schema_version"`
	Meta          DAGMeta       `json:"meta"`
	Nodes         []DAGNode     `json:"nodes"`
	DatasetNodes  []DatasetNode `json:"dataset_nodes"`
	Edges         []DAGEdge     `json:"edges"`
	MultiPeakNodes []string     `json:"multi_peak_nodes"`
}

type DAGMeta struct {
	GeneratedAt   string `json:"generated_at"`
	TotalSkills   int    `json:"total_skills"`
	Description   string `json:"description"`
}

type DAGNode struct {
	ID         string   `json:"id"`
	Type       string   `json:"type"`
	Upstream   []string `json:"upstream"`
	Downstream []string `json:"downstream"`
}

type DatasetNode struct {
	ID     string   `json:"id"`
	Domain string   `json:"domain"`
	Weight float64  `json:"weight"`
	Active bool     `json:"active"`
	Sources []string `json:"sources"`
}

type DAGEdge struct {
	From            string  `json:"from"`
	To              string  `json:"to"`
	Reward          float64 `json:"reward"`
	Cost            float64 `json:"cost"`
	ExplorationRate float64 `json:"exploration_rate"`
	FlowType        string  `json:"flow_type,omitempty"`
	Active          bool    `json:"active"`
}

// AdjEntry is an adjacency list entry.
type AdjEntry struct {
	To              string
	Reward          float64
	Cost            float64
	ExplorationRate float64
}

// ──────────────────────────────────────────────────────────────
// Main
// ──────────────────────────────────────────────────────────────

func main() {
	startTime := time.Now()

	rootFlag := flag.String("root", "/Users/lihongxin/.openclaw/workspace", "Workspace root path")
	outFlag := flag.String("out", "", "Output path (default: <root>/state/skillflow-orchestration-latest.json)")
	flag.Parse()

	root := *rootFlag
	outPath := *outFlag
	if outPath == "" {
		outPath = filepath.Join(root, "state", "skillflow-orchestration-latest.json")
	}

	// 1. Read DAG
	dag, err := readDAG(filepath.Join(root, "state", "skillflow-dag.json"))
	if err != nil {
		fmt.Fprintf(os.Stderr, "FATAL: cannot read DAG: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("[skillflow-orch] DAG loaded: %d nodes, %d edges, %d datasets, schema=%s\n",
		len(dag.Nodes), len(dag.Edges), len(dag.DatasetNodes), dag.SchemaVersion)

	// 2. Read schema for validation (optional)
	schemaPath := filepath.Join(root, "skillflow", "core", "schema.json")
	if _, err := os.Stat(schemaPath); err == nil {
		fmt.Printf("[skillflow-orch] schema found at %s\n", schemaPath)
	}

	// 3. Build topology
	fwd, rev := buildAdjacency(dag)
	topoOrder := topologicalSort(fwd, rev)
	fmt.Printf("[skillflow-orch] topological order: %d nodes\n", len(topoOrder))

	// 4. Load datasets
	datasets := LoadDatasets(dag)
	fmt.Printf("[skillflow-orch] datasets loaded: %d\n", len(datasets))

	// 5. Route trajectories (TTB)
	rng := rand.New(rand.NewSource(time.Now().UnixNano()))
	trajectories := RouteFlow(dag, fwd, rev, topoOrder, rng)
	fmt.Printf("[skillflow-orch] routes generated: %d\n", len(trajectories))

	// 6. Credit allocation
	creditAlloc := allocateCredit(dag, trajectories)
	fmt.Printf("[skillflow-orch] credit allocations: %d entries\n", len(creditAlloc))

	// 7. Collapse detection
	collapse := CollapseDetection(trajectories, dag)
	if collapse.Collapsed {
		fmt.Printf("[skillflow-orch] COLLAPSE DETECTED! severity=%.3f symptoms=%v\n",
			collapse.Severity, collapse.Symptoms)
	} else {
		fmt.Printf("[skillflow-orch] no collapse detected\n")
	}

	// 8. Multi-peak redundancy
	multiPeaks := MultiPeakRedundancy(dag, fwd, rev)
	fmt.Printf("[skillflow-orch] multi-peak redundant routes: %d\n", len(multiPeaks))

	// 9. Evolve skills
	evolved := EvolveSkills(dag, trajectories)
	fmt.Printf("[skillflow-orch] skills evolved: %d\n", len(evolved))

	// 10. Build report
	durationMs := time.Since(startTime).Milliseconds()
	hashInput := fmt.Sprintf("%s-%d", dag.Meta.GeneratedAt, time.Now().UnixNano())
	reportID := fmt.Sprintf("apex-skillflow-orch-%x", sha256.Sum256([]byte(hashInput)))[:44]

	report := Report{
		ID:                 reportID,
		DAGNodes:           len(dag.Nodes),
		DatasetsLoaded:     len(datasets),
		RoutedTrajectories: trajectories,
		CreditAllocations:  creditAlloc,
		CollapseDetected:   collapse.Collapsed,
		Collapse:           &collapse,
		MultiPeakRedundant: multiPeaks,
		EvolvedSkills:      evolved,
		Format:             "skillflow-orchestration-1.0",
		StartedAt:          startTime.Format(time.RFC3339),
		DurationMs:         durationMs,
		TrajectoryStats: map[string]interface{}{
			"total_trajectories":   len(trajectories),
			"avg_reward":           round3(avgTrajectoryReward(trajectories)),
			"avg_cost":             round3(avgTrajectoryCost(trajectories)),
			"peak_entropy":         round3(computePeakEntropy(trajectories, dag.Nodes)),
		},
	}

	// 11. Write report
	if err := WriteReport(report, outPath); err != nil {
		fmt.Fprintf(os.Stderr, "FATAL: cannot write report: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("[skillflow-orch] report written to %s\n", outPath)
	fmt.Printf("[skillflow-orch] done in %d ms\n", durationMs)
}

// ──────────────────────────────────────────────────────────────
// I/O
// ──────────────────────────────────────────────────────────────

func readDAG(path string) (*DAG, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var dag DAG
	if err := json.Unmarshal(data, &dag); err != nil {
		return nil, fmt.Errorf("unmarshal dag: %w", err)
	}
	return &dag, nil
}

func WriteReport(report Report, path string) error {
	data, err := json.MarshalIndent(report, "", "  ")
	if err != nil {
		return fmt.Errorf("marshal report: %w", err)
	}
	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		return fmt.Errorf("mkdir: %w", err)
	}
	return os.WriteFile(path, data, 0644)
}

// ──────────────────────────────────────────────────────────────
// LoadDAG — build adjacency from DAG
// ──────────────────────────────────────────────────────────────

func LoadDAG(dag *DAG) (map[string][]AdjEntry, map[string][]AdjEntry, []string) {
	fwd, rev := buildAdjacency(dag)
	topo := topologicalSort(fwd, rev)
	return fwd, rev, topo
}

func buildAdjacency(dag *DAG) (fwd, rev map[string][]AdjEntry) {
	fwd = make(map[string][]AdjEntry)
	rev = make(map[string][]AdjEntry)

	// Seed map with all node IDs
	for _, n := range dag.Nodes {
		if _, ok := fwd[n.ID]; !ok {
			fwd[n.ID] = nil
		}
		if _, ok := rev[n.ID]; !ok {
			rev[n.ID] = nil
		}
	}
	// Also seed dataset node prefixes
	for _, ds := range dag.DatasetNodes {
		dsID := "ds:" + ds.ID
		if _, ok := fwd[dsID]; !ok {
			fwd[dsID] = nil
		}
		if _, ok := rev[dsID]; !ok {
			rev[dsID] = nil
		}
	}

	// Process edges with reward/cost
	for _, e := range dag.Edges {
		if !e.Active {
			continue
		}
		// Default reward if zero (use from metric from upstream/downstream relation)
		r := e.Reward
		if r <= 0 {
			r = 0.75
		}
		c := e.Cost
		if c <= 0 {
			c = 0.25
		}
		er := e.ExplorationRate
		if er <= 0 {
			er = 0.10
		}

		fwd[e.From] = append(fwd[e.From], AdjEntry{
			To:              e.To,
			Reward:          r,
			Cost:            c,
			ExplorationRate: er,
		})
		rev[e.To] = append(rev[e.To], AdjEntry{
			To:              e.From,
			Reward:          r,
			Cost:            c,
			ExplorationRate: er,
		})

		if _, ok := fwd[e.To]; !ok {
			fwd[e.To] = nil
		}
		if _, ok := rev[e.From]; !ok {
			rev[e.From] = nil
		}
	}

	// Also add implicit edges from dataset_nodes to real nodes via "upstream/downstream"
	// Dataset nodes don't have explicit edges, so create them from domain-to-skill mappings
	domainSkillMap := map[string]string{
		"code":        "apex-harness-bridge",
		"math":        "apex-ecc-runtimeos",
		"reasoning":   "apex-superpowers-gate",
		"tool_use":    "apex-praison-chain",
		"search":      "search-skill",
		"planning":    "apex-strata",
		"memory":      "swrs-memory-skill",
		"rl":          "apex-evolver-core",
		"evaluation":  "apex-unified-research-engine",
		"multi_agent": "apex-skill-selector",
		"optimization":"apex-token-optimizer",
		"general":     "apex-core",
	}

	for _, ds := range dag.DatasetNodes {
		if !ds.Active {
			continue
		}
		dsID := "ds:" + ds.ID
		// Ensure presence
		if _, ok := fwd[dsID]; !ok {
			fwd[dsID] = nil
		}
		if _, ok := rev[dsID]; !ok {
			rev[dsID] = nil
		}
		// Map to skill
		target, ok := domainSkillMap[ds.Domain]
		if !ok {
			target = "apex-core"
		}
		// Check if target exists
		targetExists := false
		for _, n := range dag.Nodes {
			if n.ID == target {
				targetExists = true
				break
			}
		}
		if !targetExists {
			target = "apex-core"
		}

		edgeReward := ds.Weight * 0.95
		edgeCost := 1.0 - edgeReward
		if edgeCost < 0.05 {
			edgeCost = 0.05
		}

		fwd[dsID] = append(fwd[dsID], AdjEntry{
			To:              target,
			Reward:          edgeReward,
			Cost:            edgeCost,
			ExplorationRate: 0.10,
		})
		rev[target] = append(rev[target], AdjEntry{
			To:              dsID,
			Reward:          edgeReward,
			Cost:            edgeCost,
			ExplorationRate: 0.10,
		})
	}

	return
}

func topologicalSort(fwd, rev map[string][]AdjEntry) []string {
	inDegree := make(map[string]int)
	for node := range fwd {
		inDegree[node] = len(rev[node])
	}

	queue := make([]string, 0)
	for node, deg := range inDegree {
		if deg == 0 {
			queue = append(queue, node)
		}
	}

	order := make([]string, 0, len(fwd))
	for len(queue) > 0 {
		curr := queue[0]
		queue = queue[1:]
		order = append(order, curr)
		for _, adj := range fwd[curr] {
			inDegree[adj.To]--
			if inDegree[adj.To] == 0 {
				queue = append(queue, adj.To)
			}
		}
	}
	return order
}

// ──────────────────────────────────────────────────────────────
// LoadDatasets
// ──────────────────────────────────────────────────────────────

func LoadDatasets(dag *DAG) []DatasetNode {
	result := make([]DatasetNode, 0, len(dag.DatasetNodes))
	for _, ds := range dag.DatasetNodes {
		if ds.Active {
			result = append(result, ds)
		}
	}
	return result
}

// ──────────────────────────────────────────────────────────────
// RouteFlow — TTB routing with epsilon-greedy
// Flow(s→a) ∝ Reward
// ──────────────────────────────────────────────────────────────

func RouteFlow(dag *DAG, fwd, rev map[string][]AdjEntry, topoOrder []string, rng *rand.Rand) []Trajectory {
	// Source nodes: dataset nodes with "ds:" prefix
	sourceNodes := make([]string, 0)
	for _, ds := range dag.DatasetNodes {
		if ds.Active {
			sourceNodes = append(sourceNodes, "ds:"+ds.ID)
		}
	}

	// If no dataset nodes found, use nodes with no in-edges
	if len(sourceNodes) == 0 {
		for id, revs := range rev {
			if len(revs) == 0 && id != "" {
				sourceNodes = append(sourceNodes, id)
			}
		}
	}

	// Sink nodes: nodes with no out-edges
	sinkMap := make(map[string]bool)
	for id, fwds := range fwd {
		if len(fwds) == 0 && id != "" {
			sinkMap[id] = true
		}
	}
	sinks := make([]string, 0, len(sinkMap))
	for s := range sinkMap {
		sinks = append(sinks, s)
	}

	// Ensure at least one sink (the highest-degree evolver nodes)
	if len(sinks) == 0 {
		for _, n := range dag.Nodes {
			if n.Type == "evolver" {
				sinks = append(sinks, n.ID)
			}
		}
	}
	if len(sinks) == 0 {
		sinks = []string{"apex-tiangong-skill"}
	}

	// Generate one trajectory per source
	trajectories := make([]Trajectory, 0, len(sourceNodes))

	for _, src := range sourceNodes {
		traj := Trajectory{
			ID: func() string {
				h := sha256.Sum256([]byte(src + fmt.Sprintf("%d", time.Now().UnixNano())))
				return fmt.Sprintf("traj-%x", h)[:20]
			}(),
			Steps: make([]TrajectoryStep, 0),
		}

		curr := src
		stepIdx := 0
		visited := make(map[string]bool)
		visited[curr] = true

		for {
			neighbors := fwd[curr]
			if len(neighbors) == 0 {
				break
			}

			// Filter unvisited
			var candidates []AdjEntry
			for _, n := range neighbors {
				if !visited[n.To] {
					candidates = append(candidates, n)
				}
			}
			if len(candidates) == 0 {
				// Allow revisiting if no unvisited (prevents deadlock for DAGs with limited branching)
				candidates = neighbors
			}

			// Flow(s→a) ∝ Reward: probabilistic selection weighted by reward
			totalReward := 0.0
			for _, c := range candidates {
				totalReward += c.Reward
			}
			if totalReward <= 0 {
				totalReward = 1.0
			}

			epsilon := candidates[0].ExplorationRate
			var chosen AdjEntry
			epsilonGreedy := false

			if rng.Float64() < epsilon {
				chosen = candidates[rng.Intn(len(candidates))]
				epsilonGreedy = true
			} else {
				r := rng.Float64() * totalReward
				cum := 0.0
				for _, c := range candidates {
					cum += c.Reward
					if r <= cum {
						chosen = c
						break
					}
				}
			}

			stepIdx++
			traj.Steps = append(traj.Steps, TrajectoryStep{
				Step:            stepIdx,
				From:            curr,
				To:              chosen.To,
				Reward:          chosen.Reward,
				Cost:            chosen.Cost,
				EpsilonGreedy:   epsilonGreedy,
				ExplorationRate: chosen.ExplorationRate,
			})
			traj.TotalReward += chosen.Reward
			traj.TotalCost += chosen.Cost

			visited[chosen.To] = true
			curr = chosen.To

			// Stop if we hit a sink
			if sinkMap[curr] {
				break
			}
		}

		// Count peaks: nodes with ≥2 outgoing edges that were visited
		usageCount := make(map[string]int)
		for _, step := range traj.Steps {
			usageCount[step.From]++
		}
		peakNodes := make(map[string]bool)
		for nodeID, cnt := range usageCount {
			if cnt >= 2 || len(fwd[nodeID]) >= 3 {
				peakNodes[nodeID] = true
			}
		}
		traj.PeakCount = len(peakNodes)

		trajectories = append(trajectories, traj)
	}

	return trajectories
}

// ──────────────────────────────────────────────────────────────
// Credit Allocation
// ──────────────────────────────────────────────────────────────

func allocateCredit(dag *DAG, trajectories []Trajectory) map[string]float64 {
	credits := make(map[string]float64)
	nodeTotalReward := make(map[string]float64)
	nodeCount := make(map[string]int)

	for _, traj := range trajectories {
		for _, step := range traj.Steps {
			nodeTotalReward[step.From] += step.Reward
			nodeCount[step.From]++
			edgeKey := fmt.Sprintf("edge:%s→%s", step.From, step.To)
			credits[edgeKey] += step.Reward / math.Max(step.Cost, 0.001)
			// Also credit the target node
			nodeTotalReward[step.To] += step.Reward * 0.8 // attenuated for target
			nodeCount[step.To]++
		}
	}

	// Normalize node-level credits
	maxNode := 0.0
	for id, cnt := range nodeCount {
		if cnt == 0 {
			continue
		}
		avgR := nodeTotalReward[id] / float64(cnt)
		c := avgR
		credits["node:"+id] = round3(c)
		if c > maxNode {
			maxNode = c
		}
	}

	if maxNode > 0 {
		for k, v := range credits {
			credits[k] = round3(v / maxNode)
		}
	}

	// Write evidence file
	_ = writeCreditEvidence(credits, dag)
	return credits
}

func writeCreditEvidence(credits map[string]float64, dag *DAG) error {
	home := os.Getenv("HOME")
	evPath := filepath.Join(home, ".openclaw", "workspace", "state", "apex-skillflow-credit-evidence.json")
	ev := map[string]interface{}{
		"type":    "skillflow-credit-allocation",
		"credits": credits,
		"dag_id":  dag.Format,
		"format":  "skillflow-evidence-1.0",
		"written": time.Now().Format(time.RFC3339),
	}
	data, _ := json.MarshalIndent(ev, "", "  ")
	return os.WriteFile(evPath, data, 0644)
}

// ──────────────────────────────────────────────────────────────
// CollapseDetection
// ──────────────────────────────────────────────────────────────

func CollapseDetection(trajectories []Trajectory, dag *DAG) CollapseReport {
	report := CollapseReport{Collapsed: false}

	if len(trajectories) == 0 {
		return report
	}

	// 1. Bottleneck: nodes appearing in > 80% of trajectories
	nodeFreq := make(map[string]int)
	for _, traj := range trajectories {
		for _, step := range traj.Steps {
			nodeFreq[step.From]++
			nodeFreq[step.To]++
		}
	}
	total := float64(len(trajectories))
	for node, count := range nodeFreq {
		ratio := float64(count) / total
		if ratio > 0.80 {
			report.Symptoms = append(report.Symptoms,
				fmt.Sprintf("bottleneck node %q appears in %.0f%% of trajectories", node, ratio*100))
			report.CollapsedKeys = append(report.CollapsedKeys, node)
			report.Collapsed = true
		}
	}

	// 2. Reward convergence
	if len(trajectories) >= 3 {
		rewards := make([]float64, len(trajectories))
		for i, t := range trajectories {
			rewards[i] = t.TotalReward
		}
		_, variance := meanVar(rewards)
		stddev := math.Sqrt(variance)
		if stddev < 0.05 {
			report.Symptoms = append(report.Symptoms,
				fmt.Sprintf("reward convergence: std=%.4f (< 0.05)", stddev))
			report.Collapsed = true
		}
	}

	// 3. Coverage
	allVisited := make(map[string]bool)
	for _, traj := range trajectories {
		for _, step := range traj.Steps {
			allVisited[step.From] = true
			allVisited[step.To] = true
		}
	}
	totalNodes := len(dag.Nodes) + len(dag.DatasetNodes)
	if totalNodes > 0 {
		coverage := float64(len(allVisited)) / float64(totalNodes)
		if coverage < 0.3 {
			report.Symptoms = append(report.Symptoms,
				fmt.Sprintf("low coverage: %.0f%% of nodes visited (< 30%%", coverage*100))
			report.Collapsed = true
			if coverage < 0.15 {
				report.Severity = 1.0
			} else {
				report.Severity = 0.6
			}
		}
	}

	if report.Collapsed && report.Severity == 0 {
		report.Severity = 0.5
	}
	return report
}

// ──────────────────────────────────────────────────────────────
// MultiPeakRedundancy
// ──────────────────────────────────────────────────────────────

func MultiPeakRedundancy(dag *DAG, fwd, rev map[string][]AdjEntry) []MultiPeakRoute {
	// Identify peak nodes: those with high branching or explicitly listed
	peakSet := make(map[string]bool)
	for _, id := range dag.MultiPeakNodes {
		peakSet[id] = true
	}
	// Also detect by degree
	for id := range fwd {
		if len(fwd[id]) >= 3 || len(rev[id]) >= 3 {
			peakSet[id] = true
		}
	}

	results := make([]MultiPeakRoute, 0)
	seen := make(map[string]bool)

	for peak := range peakSet {
		routeID := fmt.Sprintf("peak-redundant-%s", peak)
		if seen[routeID] {
			continue
		}
		seen[routeID] = true

		// Primary path: this node + all its predecessors
		primary := make([]string, 0)
		for _, r := range rev[peak] {
			primary = append(primary, r.To)
		}
		primary = append(primary, peak)
		primary = uniqueStrs(primary)
		sort.Strings(primary)

		// Redundant path: alternate paths via downstream + alternative predecessors
		redundant := make([]string, 0)
		seenPred := make(map[string]bool)
		for _, r := range rev[peak] {
			if !seenPred[r.To] {
				seenPred[r.To] = true
				// Add 2-hop alternate paths
				for _, r2 := range rev[r.To] {
					if r2.To != peak && !seenPred[r2.To] {
						redundant = append(redundant, r2.To)
						seenPred[r2.To] = true
					}
				}
			}
		}
		redundant = append(redundant, peak)
		// Add downstream alternatives
		for _, f := range fwd[peak] {
			if !seenPred[f.To] {
				redundant = append(redundant, f.To)
				seenPred[f.To] = true
			}
		}
		redundant = uniqueStrs(redundant)
		sort.Strings(redundant)

		overlap := jaccardSim(primary, redundant)
		diversity := 1.0 - overlap

		results = append(results, MultiPeakRoute{
			RouteID:       routeID,
			PrimaryPath:   primary,
			RedundantPath: redundant,
			Diversity:     round3(diversity),
			Overlap:       round3(overlap),
		})
	}

	return results
}

// ──────────────────────────────────────────────────────────────
// EvolveSkills — reverse policy reward adjustment
// ──────────────────────────────────────────────────────────────

func EvolveSkills(dag *DAG, trajectories []Trajectory) []EvolvedSkill {
	nodeReward := make(map[string]float64)
	nodeCount := make(map[string]int)

	for _, traj := range trajectories {
		for _, step := range traj.Steps {
			nodeReward[step.From] += step.Reward
			nodeCount[step.From]++
			nodeReward[step.To] += step.Reward * 0.6
			nodeCount[step.To]++
		}
	}

	results := make([]EvolvedSkill, 0)

	for _, node := range dag.Nodes {
		if nodeCount[node.ID] == 0 || node.ID == "apex-core" {
			continue
		}
		actualAvg := nodeReward[node.ID] / float64(nodeCount[node.ID])
		expectedBase := rewardBaseForNode(node)

		if actualAvg < 0.01 {
			continue
		}

		delta := actualAvg - expectedBase
		if math.Abs(delta) < 0.01 {
			continue
		}

		alpha := 0.08
		adjustment := alpha * delta
		newBase := expectedBase + adjustment
		if newBase < 0.1 {
			newBase = 0.1
		}
		if newBase > 1.0 {
			newBase = 1.0
		}

		direction := "up"
		if adjustment < 0 {
			direction = "down"
		}

		results = append(results, EvolvedSkill{
			SkillID:        node.ID,
			OldRewardBase:  round3(expectedBase),
			NewRewardBase:  round3(newBase),
			Adjustment:     round4(adjustment),
			Direction:      direction,
			Alpha:          alpha,
			ReversePolicyR: round3(actualAvg),
		})
	}

	return results
}

func rewardBaseForNode(node DAGNode) float64 {
	switch node.Type {
	case "core":
		return 0.85
	case "planner":
		return 0.82
	case "executor":
		return 0.78
	case "validator":
		return 0.88
	case "memory":
		return 0.74
	case "evolver":
		return 0.76
	default:
		return 0.80
	}
}

// ──────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────

func meanVar(vals []float64) (mean, variance float64) {
	if len(vals) == 0 {
		return 0, 0
	}
	sum := 0.0
	for _, v := range vals {
		sum += v
	}
	mean = sum / float64(len(vals))
	varSum := 0.0
	for _, v := range vals {
		d := v - mean
		varSum += d * d
	}
	variance = varSum / float64(len(vals))
	return
}

func jaccardSim(a, b []string) float64 {
	sa := make(map[string]bool)
	sb := make(map[string]bool)
	for _, s := range a {
		sa[s] = true
	}
	for _, s := range b {
		sb[s] = true
	}
	inter := 0
	union := len(sa)
	for s := range sb {
		if sa[s] {
			inter++
		} else {
			union++
		}
	}
	if union == 0 {
		return 0
	}
	return float64(inter) / float64(union)
}

func uniqueStrs(s []string) []string {
	seen := make(map[string]bool)
	r := make([]string, 0, len(s))
	for _, v := range s {
		if !seen[v] {
			seen[v] = true
			r = append(r, v)
		}
	}
	return r
}

func round3(v float64) float64 {
	return math.Round(v*1000) / 1000
}

func round4(v float64) float64 {
	return math.Round(v*10000) / 10000
}

func avgTrajectoryReward(trajs []Trajectory) float64 {
	if len(trajs) == 0 {
		return 0
	}
	s := 0.0
	for _, t := range trajs {
		s += t.TotalReward
	}
	return s / float64(len(trajs))
}

func avgTrajectoryCost(trajs []Trajectory) float64 {
	if len(trajs) == 0 {
		return 0
	}
	s := 0.0
	for _, t := range trajs {
		s += t.TotalCost
	}
	return s / float64(len(trajs))
}

func computePeakEntropy(trajs []Trajectory, nodes []DAGNode) float64 {
	if len(trajs) == 0 {
		return 0
	}
	peaks := make([]float64, len(trajs))
	for i, t := range trajs {
		peaks[i] = float64(t.PeakCount)
	}
	total := 0.0
	for _, p := range peaks {
		total += p
	}
	if total <= 0 {
		return 0
	}
	entropy := 0.0
	for _, p := range peaks {
		if p == 0 {
			continue
		}
		prob := p / total
		entropy -= prob * math.Log2(prob)
	}
	norm := math.Log2(float64(len(trajs)))
	if norm <= 0 {
		return 0
	}
	return entropy / norm
}
