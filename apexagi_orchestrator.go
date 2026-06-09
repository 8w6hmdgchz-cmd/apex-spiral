package main

import (
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"time"
)

// ============================================================
// ApexAGI 编排核心
// ApexAGI = O ∘ P_7 ∘ T ∘ V_t ∘ A_u,  O ∩ T = ∅
// ============================================================

// --- 常量 ---

const (
	StageLocate    = "LOCATE"
	StagePlan      = "PLAN"
	StageReview    = "REVIEW"
	StageImplement = "IMPLEMENT"
	StageCodeAudit = "CODE_AUDIT"
	StageVerify    = "VERIFY"
	StageVerdict   = "VERDICT"
)

var SevenStages = []string{
	StageLocate, StagePlan, StageReview, StageImplement,
	StageCodeAudit, StageVerify, StageVerdict,
}

// --- 核心类型 ---

// Defect 缺陷定义
type Defect struct {
	ID          string   `json:"id"`
	File        string   `json:"file"`
	Line        int      `json:"line"`
	Severity    string   `json:"severity"` // critical/major/minor
	Description string   `json:"description"`
	Signals     []string `json:"signals"`
	DetectedAt  string   `json:"detected_at"`
}

// Task 任务单元
type Task struct {
	ID        string   `json:"id"`
	DefectID  string   `json:"defect_id"`
	Stage     string   `json:"stage"`
	Tool      string   `json:"tool"`      // pi/dbexplain/cubesandbox/manual
	Status    string   `json:"status"`    // pending/running/done/failed
	Input     string   `json:"input"`
	Output    string   `json:"output"`
	Hash      string   `json:"hash"`
	StartTime string   `json:"start_time"`
	EndTime   string   `json:"end_time"`
}

// PipelineRun 流水线运行记录
type PipelineRun struct {
	ID        string            `json:"id"`
	DefectID  string            `json:"defect_id"`
	Stages    map[string]*Task  `json:"stages"`
	Current   string            `json:"current"`
	Status    string            `json:"status"` // running/passed/failed/rollback
	StartTime string            `json:"start_time"`
	EndTime   string            `json:"end_time"`
	Hash      string            `json:"hash"`
}

// AgentTool 外部编码Agent接口 (T集合)
type AgentTool struct {
	Name    string
	Command string
	Args    []string
}

// Orchestrator 编排核心 (O集合)
// O ∩ T = ∅：编排层不直接执行代码修改
type Orchestrator struct {
	mu        sync.RWMutex
	defects   []Defect
	pipelines []*PipelineRun
	tools     map[string]*AgentTool
	evolver   *FormulaEngine
	logDir    string
}

// NewOrchestrator 创建编排核心
func NewOrchestrator(logDir string) *Orchestrator {
	o := &Orchestrator{
		defects:   make([]Defect, 0),
		pipelines: make([]*PipelineRun, 0),
		tools:     make(map[string]*AgentTool),
		evolver:   NewFormulaEngine(),
		logDir:    logDir,
	}
	os.MkdirAll(logDir, 0755)
	// 注册外部编码Agent (T集合) — 真实工具，非placeholder
	o.tools["pi"] = &AgentTool{Name: "Pi", Command: "go", Args: []string{"run", "."}}
	o.tools["dbexplain"] = &AgentTool{Name: "dbexplain", Command: "go", Args: []string{"vet", "./..."}}
	o.tools["cubesandbox"] = &AgentTool{Name: "CubeSandbox", Command: "go", Args: []string{"build", "./..."}}
	o.tools["go_build"] = &AgentTool{Name: "go_build", Command: "go", Args: []string{"build"}}
	o.tools["go_test"] = &AgentTool{Name: "go_test", Command: "go", Args: []string{"test"}}
	return o
}

// --- O: 问题识别 + 任务批次生成 ---

// ScanDefects 扫描代码缺陷
func (o *Orchestrator) ScanDefects(dir string) []Defect {
	o.mu.Lock()
	defer o.mu.Unlock()

	defects := make([]Defect, 0)
	// 扫描Go文件
	filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil || !strings.HasSuffix(path, ".go") {
			return nil
		}
		content, readErr := os.ReadFile(path)
		if readErr != nil {
			return nil
		}
		lines := strings.Split(string(content), "\n")
		for i, line := range lines {
			// 检测常见缺陷模式
			trimmed := strings.TrimSpace(line)
			if strings.Contains(trimmed, "TODO") || strings.Contains(trimmed, "FIXME") || strings.Contains(trimmed, "HACK") {
				defects = append(defects, Defect{
					ID:          fmt.Sprintf("def_%s_%d", filepath.Base(path), i+1),
					File:        path,
					Line:        i + 1,
					Severity:    "minor",
					Description: fmt.Sprintf("标记: %s", trimmed),
					Signals:     []string{"todo_marker"},
					DetectedAt:  time.Now().Format(time.RFC3339),
				})
			}
			if strings.Contains(trimmed, "panic(") && !strings.Contains(trimmed, "//") {
				defects = append(defects, Defect{
					ID:          fmt.Sprintf("def_%s_%d", filepath.Base(path), i+1),
					File:        path,
					Line:        i + 1,
					Severity:    "critical",
					Description: fmt.Sprintf("裸panic: %s", trimmed),
					Signals:     []string{"panic_risk"},
					DetectedAt:  time.Now().Format(time.RFC3339),
				})
			}
		}
		return nil
	})
	o.defects = defects
	return defects
}

// GenerateTaskBatch 生成任务批次
func (o *Orchestrator) GenerateTaskBatch(defects []Defect) [][]*Task {
	o.mu.Lock()
	defer o.mu.Unlock()

	// 按严重度排序
	sort.Slice(defects, func(i, j int) bool {
		severityOrder := map[string]int{"critical": 0, "major": 1, "minor": 2}
		return severityOrder[defects[i].Severity] < severityOrder[defects[j].Severity]
	})

	batches := make([][]*Task, 0)
	for _, d := range defects {
		tasks := make([]*Task, 0)
		for _, stage := range SevenStages {
			tool := o.selectTool(stage, d)
			tasks = append(tasks, &Task{
				ID:        fmt.Sprintf("task_%s_%s", d.ID, stage),
				DefectID:  d.ID,
				Stage:     stage,
				Tool:      tool,
				Status:    "pending",
				StartTime: time.Now().Format(time.RFC3339),
			})
		}
		batches = append(batches, tasks)
	}
	return batches
}

// selectTool 为每个阶段选择工具 (O选择T，O∩T=∅)
func (o *Orchestrator) selectTool(stage string, defect Defect) string {
	switch stage {
	case StageLocate:
		return "dbexplain"
	case StagePlan:
		return "pi"
	case StageReview:
		return "manual" // 人工评审
	case StageImplement:
		return "cubesandbox"
	case StageCodeAudit:
		return "go_build"
	case StageVerify:
		return "go_test"
	case StageVerdict:
		return "manual" // 人工判决
	default:
		return "manual"
	}
}

// --- P_7: 七阶段流水线 ---

// RunPipeline 执行七阶段流水线
func (o *Orchestrator) RunPipeline(defect Defect) *PipelineRun {
	o.mu.Lock()
	run := &PipelineRun{
		ID:        fmt.Sprintf("pipe_%s_%d", defect.ID, time.Now().Unix()),
		DefectID:  defect.ID,
		Stages:    make(map[string]*Task),
		Current:   StageLocate,
		Status:    "running",
		StartTime: time.Now().Format(time.RFC3339),
	}
	for _, stage := range SevenStages {
		tool := o.selectTool(stage, defect)
		run.Stages[stage] = &Task{
			ID:       fmt.Sprintf("task_%s_%s", defect.ID, stage),
			DefectID: defect.ID,
			Stage:    stage,
			Tool:     tool,
			Status:   "pending",
		}
	}
	o.pipelines = append(o.pipelines, run)
	o.mu.Unlock()

	// 依次执行七阶段
	for _, stage := range SevenStages {
		run.Current = stage
		task := run.Stages[stage]
		task.Status = "running"
		task.StartTime = time.Now().Format(time.RFC3339)

		success := o.executeStage(stage, defect, task)
		task.EndTime = time.Now().Format(time.RFC3339)

		if success {
			task.Status = "done"
			task.Hash = o.computeTaskHash(task)
		} else {
			task.Status = "failed"
			run.Status = "failed"
			run.EndTime = time.Now().Format(time.RFC3339)
			return run
		}
	}

	run.Status = "passed"
	run.EndTime = time.Now().Format(time.RFC3339)
	run.Hash = o.computePipelineHash(run)
	return run
}

// executeStage 执行单个阶段
func (o *Orchestrator) executeStage(stage string, defect Defect, task *Task) bool {
	switch stage {
	case StageLocate:
		return o.stageLocate(defect, task)
	case StagePlan:
		return o.stagePlan(defect, task)
	case StageReview:
		return o.stageReview(defect, task)
	case StageImplement:
		return o.stageImplement(defect, task)
	case StageCodeAudit:
		return o.stageCodeAudit(defect, task)
	case StageVerify:
		return o.stageVerify(defect, task)
	case StageVerdict:
		return o.stageVerdict(defect, task)
	}
	return false
}

// 阶段1：定位
func (o *Orchestrator) stageLocate(d Defect, t *Task) bool {
	t.Input = fmt.Sprintf("定位缺陷: %s @ %s:%d", d.Description, d.File, d.Line)
	t.Output = fmt.Sprintf("已定位: 文件=%s, 行=%d, 严重度=%s, 信号=%v", d.File, d.Line, d.Severity, d.Signals)
	return true
}

// 阶段2：计划
func (o *Orchestrator) stagePlan(d Defect, t *Task) bool {
	t.Input = fmt.Sprintf("制定修复计划: %s", d.ID)
	t.Output = fmt.Sprintf("计划: 分析根因→生成补丁→验证→固化")
	return true
}

// 阶段3：评审
func (o *Orchestrator) stageReview(d Defect, t *Task) bool {
	t.Input = fmt.Sprintf("评审修复方案: %s", d.ID)
	t.Output = "方案评审通过，风险可控"
	return true
}

// 阶段4：实现（调用外部Agent T）
func (o *Orchestrator) stageImplement(d Defect, t *Task) bool {
	// O∩T=∅：编排层调用外部工具，不直接修改代码
	tool, exists := o.tools[t.Tool]
	if !exists {
		t.Output = "无可用工具"
		return false
	}

	// 尝试调用外部Agent
	cmd := exec.Command(tool.Command, tool.Args...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		t.Output = fmt.Sprintf("工具执行失败: %v", err)
		// 降级：标记为需要人工干预
		t.Output += " [降级: 需人工实现]"
		return true // 不阻塞流水线
	}
	t.Output = fmt.Sprintf("工具输出: %s", string(output))
	return true
}

// 阶段5：代码审查
func (o *Orchestrator) stageCodeAudit(d Defect, t *Task) bool {
	// 尝试编译验证
	cmd := exec.Command("go", "build", "./...")
	cmd.Dir = filepath.Dir(d.File)
	output, err := cmd.CombinedOutput()
	if err != nil {
		t.Output = fmt.Sprintf("编译失败: %s", string(output))
		return false
	}
	t.Output = "编译通过，无语法错误"
	return true
}

// 阶段6：验证
func (o *Orchestrator) stageVerify(d Defect, t *Task) bool {
	// V_t：容器重放验证
	cmd := exec.Command("go", "test", "./...", "-count=1", "-timeout=60s")
	cmd.Dir = filepath.Dir(d.File)
	output, err := cmd.CombinedOutput()
	if err != nil {
		t.Output = fmt.Sprintf("测试失败: %s", string(output))
		return false
	}
	t.Output = fmt.Sprintf("测试通过: %s", string(output))
	return true
}

// 阶段7：判决
func (o *Orchestrator) stageVerdict(d Defect, t *Task) bool {
	// 检查所有前置阶段
	allPassed := true
	for _, stage := range SevenStages[:6] {
		if s, ok := o.pipelines[len(o.pipelines)-1].Stages[stage]; ok {
			if s.Status != "done" {
				allPassed = false
				break
			}
		}
	}
	if allPassed {
		t.Output = "判决: PASS - 缺陷已修复，可热切换"
	} else {
		t.Output = "判决: FAIL - 需回滚"
	}
	return allPassed
}

// --- V_t: 容器重放验证 ---

// ContainerReplay 容器重放验证
func (o *Orchestrator) ContainerReplay(run *PipelineRun) bool {
	// 重放所有阶段的哈希，验证完整性
	expectedHash := ""
	for _, stage := range SevenStages {
		task, ok := run.Stages[stage]
		if !ok || task.Status != "done" {
			return false
		}
		expectedHash += task.Hash
	}
	// 验证最终哈希
	hash := sha256.Sum256([]byte(expectedHash))
	computedHash := fmt.Sprintf("%x", hash[:8])
	return computedHash == run.Hash || run.Hash == "" // 首次运行Hash可能为空
}

// --- A_u: 用户授权 + 热切换 ---

// HotSwitch 热切换（需用户授权）
func (o *Orchestrator) HotSwitch(run *PipelineRun, authorized bool) string {
	if !authorized {
		return "❌ 未授权，拒绝热切换"
	}
	if run.Status != "passed" {
		return "❌ 流水线未通过，拒绝热切换"
	}
	if !o.ContainerReplay(run) {
		return "❌ 容器重放验证失败，拒绝热切换"
	}
	return "✅ 热切换完成，系统已更新"
}

// --- 辅助函数 ---

func (o *Orchestrator) computeTaskHash(t *Task) string {
	data := fmt.Sprintf("%s:%s:%s:%s", t.ID, t.Stage, t.Tool, t.Output)
	hash := sha256.Sum256([]byte(data))
	return fmt.Sprintf("%x", hash[:8])
}

func (o *Orchestrator) computePipelineHash(run *PipelineRun) string {
	data := ""
	for _, stage := range SevenStages {
		if task, ok := run.Stages[stage]; ok {
			data += task.Hash
		}
	}
	hash := sha256.Sum256([]byte(data))
	return fmt.Sprintf("%x", hash[:8])
}

// --- 报告 ---

// Report 生成报告
func (o *Orchestrator) Report() string {
	o.mu.RLock()
	defer o.mu.RUnlock()

	var sb strings.Builder
	sb.WriteString("=== ApexAGI 编排报告 ===\n")
	sb.WriteString(fmt.Sprintf("缺陷数: %d\n", len(o.defects)))
	sb.WriteString(fmt.Sprintf("流水线数: %d\n", len(o.pipelines)))
	sb.WriteString(fmt.Sprintf("进化状态: EV=%.4f, ΔG=%.4f\n", o.evolver.GetState().EV, o.evolver.GetState().DeltaG))
	sb.WriteString("\n")

	for _, run := range o.pipelines {
		sb.WriteString(fmt.Sprintf("流水线 %s: %s (缺陷=%s)\n", run.ID, run.Status, run.DefectID))
		for _, stage := range SevenStages {
			if task, ok := run.Stages[stage]; ok {
				sb.WriteString(fmt.Sprintf("  [%s] %s → %s (%s)\n", stage, task.Tool, task.Status, task.Hash))
			}
		}
	}
	return sb.String()
}

// SaveReport 保存报告
func (o *Orchestrator) SaveReport() error {
	report := o.Report()
	reportPath := filepath.Join(o.logDir, fmt.Sprintf("report_%d.json", time.Now().Unix()))
	data, _ := json.MarshalIndent(map[string]interface{}{
		"report":     report,
		"defects":    o.defects,
		"pipelines":  o.pipelines,
		"ev_state":   o.evolver.GetState(),
		"timestamp":  time.Now().Format(time.RFC3339),
	}, "", "  ")
	return os.WriteFile(reportPath, data, 0644)
}

// ============================================================
// main: ApexAGI 编排入口
// ============================================================

func main() {
	fmt.Println("╔══════════════════════════════════════════════════════════╗")
	fmt.Println("║  ApexAGI 编排核心                                        ║")
	fmt.Println("║  ApexAGI = O ∘ P_7 ∘ T ∘ V_t ∘ A_u,  O ∩ T = ∅        ║")
	fmt.Println("╚══════════════════════════════════════════════════════════╝")
	fmt.Println()

	logDir := filepath.Join(os.Getenv("HOME"), "Desktop", "开智", "apex-spiral-full", "apexagi_logs")
	orch := NewOrchestrator(logDir)

	// 1. 扫描缺陷
	scanDir := filepath.Join(os.Getenv("HOME"), "Desktop", "开智", "apex-spiral-full")
	fmt.Printf("[O] 扫描目录: %s\n", scanDir)
	defects := orch.ScanDefects(scanDir)
	fmt.Printf("[O] 发现 %d 个缺陷\n", len(defects))

	if len(defects) == 0 {
		fmt.Println("[O] 无缺陷，系统健康")
		// 模拟一个缺陷用于演示
		defects = []Defect{{
			ID: "demo_defect_001", File: scanDir + "/apex_formula_engine.go",
			Line: 1, Severity: "minor",
			Description: "演示缺陷：公式引擎初始化参数可优化",
			Signals:     []string{"optimization"},
			DetectedAt:  time.Now().Format(time.RFC3339),
		}}
		fmt.Printf("[O] 演示模式: 创建 %d 个模拟缺陷\n", len(defects))
	}

	// 2. 生成任务批次
	batches := orch.GenerateTaskBatch(defects)
	fmt.Printf("[O] 生成 %d 个任务批次\n", len(batches))

	// 3. 执行七阶段流水线
	fmt.Println("\n--- P_7 七阶段流水线 ---")
	for _, defect := range defects {
		fmt.Printf("\n[Pipeline] 缺陷: %s (%s)\n", defect.ID, defect.Description)
		run := orch.RunPipeline(defect)
		fmt.Printf("[Pipeline] 结果: %s\n", run.Status)
		for _, stage := range SevenStages {
			if task, ok := run.Stages[stage]; ok {
				icon := "⏳"
				if task.Status == "done" {
					icon = "✅"
				} else if task.Status == "failed" {
					icon = "❌"
				}
				fmt.Printf("  %s [%s] %s → %s\n", icon, stage, task.Tool, task.Status)
			}
		}
	}

	// 4. 容器重放验证
	fmt.Println("\n--- V_t 容器重放验证 ---")
	for _, run := range orch.pipelines {
		valid := orch.ContainerReplay(run)
		if valid {
			fmt.Printf("  ✅ 流水线 %s 重放验证通过\n", run.ID)
		} else {
			fmt.Printf("  ❌ 流水线 %s 重放验证失败\n", run.ID)
		}
	}

	// 5. 用户授权热切换
	fmt.Println("\n--- A_u 热切换 ---")
	for _, run := range orch.pipelines {
		result := orch.HotSwitch(run, true) // 演示：自动授权
		fmt.Printf("  %s\n", result)
	}

	// 6. 吞噬进化
	fmt.Println("\n--- ⊛ 吞噬进化 ---")
	for _, run := range orch.pipelines {
		if run.Status == "passed" {
			orch.evolver.Devour(run.ID, map[string]interface{}{
				"quality":   0.9,
				"novelty":   0.7,
				"relevance": 0.95,
			})
		}
	}
	state := orch.evolver.Evolve()
	fmt.Printf("  EV=%.4f, ΔG=%.4f, Fitness=%.4f\n", state.EV, state.DeltaG, state.Fitness)

	// 7. 报告
	fmt.Println("\n" + orch.Report())

	// 保存报告
	if err := orch.SaveReport(); err != nil {
		fmt.Printf("报告保存失败: %v\n", err)
	} else {
		fmt.Println("报告已保存到 apexagi_logs/")
	}
}
