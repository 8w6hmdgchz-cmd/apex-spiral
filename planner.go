// planner.go — 目标分解与规划
// AGI核心能力：能理解复杂任务、自动分解、制定执行计划

package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"sort"
	"strings"
	"time"
)

// Task 任务
type Task struct {
	ID          string    `json:"id"`
	Description string    `json:"description"`
	Priority    int       `json:"priority"`    // 1-5, 1最高
	EstimatedTime string  `json:"estimated_time"` // 分钟
	Status      string    `json:"status"`      // pending/in_progress/completed/blocked
	Dependencies []string `json:"dependencies"` // 依赖的任务ID
	SubTasks    []*Task   `json:"sub_tasks,omitempty"`
	Result      string    `json:"result,omitempty"`
	CreatedAt   time.Time `json:"created_at"`
	StartedAt   time.Time `json:"started_at,omitempty"`
	CompletedAt time.Time `json:"completed_at,omitempty"`
}

// Plan 执行计划
type Plan struct {
	ID          string    `json:"id"`
	Goal        string    `json:"goal"`           // 最终目标
	Tasks       []*Task   `json:"tasks"`          // 所有任务
	TaskMap     map[string]*Task `json:"-"`       // 任务ID映射
	CurrentTask int       `json:"current_task"`   // 当前任务索引
	Status      string    `json:"status"`         // planning/executing/completed/failed
	CreatedAt   time.Time `json:"created_at"`
	CompletedAt time.Time `json:"completed_at,omitempty"`
}

// Planner 规划器
type Planner struct {
	Plans map[string]*Plan
}

// NewPlanner 创建规划器
func NewPlanner() *Planner {
	return &Planner{
		Plans: make(map[string]*Plan),
	}
}

// DecomposeGoal 分解目标
func (p *Planner) DecomposeGoal(goal string) *Plan {
	id := fmt.Sprintf("plan_%d", time.Now().UnixNano())
	plan := &Plan{
		ID:        id,
		Goal:      goal,
		Tasks:     make([]*Task, 0),
		TaskMap:   make(map[string]*Task),
		Status:    "planning",
		CreatedAt: time.Now(),
	}

	// 分析目标类型并分解
	tasks := p.analyzeAndDecompose(goal)
	for _, task := range tasks {
		plan.AddTask(task)
	}

	// 排序（按优先级和依赖）
	plan.SortTasks()

	plan.Status = "planning"
	p.Plans[id] = plan
	return plan
}

// analyzeAndDecompose 分析并分解
func (p *Planner) analyzeAndDecompose(goal string) []*Task {
	var tasks []*Task
	goalLower := strings.ToLower(goal)

	// 分析目标类型
	if strings.Contains(goalLower, "学习") || strings.Contains(goalLower, "理解") {
		tasks = p.decomposeLearning(goal)
	} else if strings.Contains(goalLower, "构建") || strings.Contains(goalLower, "创建") || strings.Contains(goalLower, "实现") {
		tasks = p.decomposeBuilding(goal)
	} else if strings.Contains(goalLower, "分析") || strings.Contains(goalLower, "研究") {
		tasks = p.decomposeAnalysis(goal)
	} else if strings.Contains(goalLower, "优化") || strings.Contains(goalLower, "改进") {
		tasks = p.decomposeOptimization(goal)
	} else {
		tasks = p.decomposeGeneric(goal)
	}

	return tasks
}

// decomposeLearning 分解学习类目标
func (p *Planner) decomposeLearning(goal string) []*Task {
	var tasks []*Task

	tasks = append(tasks, &Task{
		Description: "理解核心概念和定义",
		Priority:    1,
		EstimatedTime: "30",
	})

	tasks = append(tasks, &Task{
		Description: "收集相关资料和例子",
		Priority:    2,
		EstimatedTime: "60",
	})

	tasks = append(tasks, &Task{
		Description: "实践练习和验证",
		Priority:    3,
		EstimatedTime: "90",
		Dependencies: []string{"2"}, // 依赖收集资料
	})

	tasks = append(tasks, &Task{
		Description: "总结和知识沉淀",
		Priority:    4,
		EstimatedTime: "30",
		Dependencies: []string{"3"},
	})

	return tasks
}

// decomposeBuilding 分解构建类目标
func (p *Planner) decomposeBuilding(goal string) []*Task {
	var tasks []*Task

	tasks = append(tasks, &Task{
		Description: "需求分析和设计",
		Priority:    1,
		EstimatedTime: "60",
	})

	tasks = append(tasks, &Task{
		Description: "技术选型和架构设计",
		Priority:    1,
		EstimatedTime: "45",
		Dependencies: []string{"1"},
	})

	tasks = append(tasks, &Task{
		Description: "核心功能实现",
		Priority:    2,
		EstimatedTime: "180",
		Dependencies: []string{"2"},
	})

	tasks = append(tasks, &Task{
		Description: "测试和修复",
		Priority:    3,
		EstimatedTime: "90",
		Dependencies: []string{"3"},
	})

	tasks = append(tasks, &Task{
		Description: "部署和上线",
		Priority:    4,
		EstimatedTime: "30",
		Dependencies: []string{"4"},
	})

	return tasks
}

// decomposeAnalysis 分解分析类目标
func (p *Planner) decomposeAnalysis(goal string) []*Task {
	var tasks []*Task

	tasks = append(tasks, &Task{
		Description: "定义分析范围和目标",
		Priority:    1,
		EstimatedTime: "20",
	})

	tasks = append(tasks, &Task{
		Description: "收集和整理数据",
		Priority:    2,
		EstimatedTime: "60",
		Dependencies: []string{"1"},
	})

	tasks = append(tasks, &Task{
		Description: "数据分析",
		Priority:    2,
		EstimatedTime: "120",
		Dependencies: []string{"2"},
	})

	tasks = append(tasks, &Task{
		Description: "生成分析报告",
		Priority:    3,
		EstimatedTime: "45",
		Dependencies: []string{"3"},
	})

	return tasks
}

// decomposeOptimization 分解优化类目标
func (p *Planner) decomposeOptimization(goal string) []*Task {
	var tasks []*Task

	tasks = append(tasks, &Task{
		Description: "定位性能瓶颈",
		Priority:    1,
		EstimatedTime: "30",
	})

	tasks = append(tasks, &Task{
		Description: "分析问题根因",
		Priority:    1,
		EstimatedTime: "45",
		Dependencies: []string{"1"},
	})

	tasks = append(tasks, &Task{
		Description: "制定优化方案",
		Priority:    2,
		EstimatedTime: "30",
		Dependencies: []string{"2"},
	})

	tasks = append(tasks, &Task{
		Description: "实施优化",
		Priority:    3,
		EstimatedTime: "90",
		Dependencies: []string{"3"},
	})

	tasks = append(tasks, &Task{
		Description: "验证优化效果",
		Priority:    4,
		EstimatedTime: "30",
		Dependencies: []string{"4"},
	})

	return tasks
}

// decomposeGeneric 通用分解
func (p *Planner) decomposeGeneric(goal string) []*Task {
	return []*Task{
		{
			Description: fmt.Sprintf("理解目标: %s", goal),
			Priority:    1,
			EstimatedTime: "30",
		},
		{
			Description: "分解任务步骤",
			Priority:    1,
			EstimatedTime: "20",
			Dependencies: []string{"1"},
		},
		{
			Description: "执行任务",
			Priority:    2,
			EstimatedTime: "120",
			Dependencies: []string{"2"},
		},
		{
			Description: "验证结果",
			Priority:    3,
			EstimatedTime: "30",
			Dependencies: []string{"3"},
		},
	}
}

// AddTask 添加任务
func (p *Plan) AddTask(task *Task) {
	// 分配ID
	if task.ID == "" {
		task.ID = fmt.Sprintf("%d", len(p.Tasks)+1)
	}

	// 设置状态
	task.Status = "pending"
	task.CreatedAt = time.Now()

	p.Tasks = append(p.Tasks, task)
	p.TaskMap[task.ID] = task
}

// SortTasks 排序任务
func (p *Plan) SortTasks() {
	// 拓扑排序：按依赖关系排序
	sorted := p.topologicalSort()

	// 按优先级微调
	sort.Slice(sorted, func(i, j int) bool {
		if sorted[i].Priority != sorted[j].Priority {
			return sorted[i].Priority < sorted[j].Priority
		}
		return sorted[i].EstimatedTime < sorted[j].EstimatedTime
	})

	p.Tasks = sorted
}

// topologicalSort 拓扑排序
func (p *Plan) topologicalSort() []*Task {
	var sorted []*Task
	visited := make(map[string]bool)
	var visit func(t *Task)

	visit = func(t *Task) {
		if visited[t.ID] {
			return
		}
		visited[t.ID] = true

		// 先访问依赖
		for _, depID := range t.Dependencies {
			if dep, ok := p.TaskMap[depID]; ok {
				visit(dep)
			}
		}

		sorted = append(sorted, t)
	}

	for _, task := range p.Tasks {
		visit(task)
	}

	return sorted
}

// GetNextExecutable 获取下一个可执行任务
func (p *Plan) GetNextExecutable() *Task {
	for _, task := range p.Tasks {
		if task.Status != "pending" {
			continue
		}

		// 检查依赖是否都已完成
		allDepsDone := true
		for _, depID := range task.Dependencies {
			if dep, ok := p.TaskMap[depID]; ok {
				if dep.Status != "completed" {
					allDepsDone = false
					break
				}
			}
		}

		if allDepsDone {
			return task
		}
	}

	return nil
}

// StartTask 开始任务
func (p *Plan) StartTask(taskID string) (*Task, error) {
	task, ok := p.TaskMap[taskID]
	if !ok {
		return nil, fmt.Errorf("task not found: %s", taskID)
	}

	if task.Status != "pending" {
		return nil, fmt.Errorf("task not in pending state")
	}

	// 检查依赖
	for _, depID := range task.Dependencies {
		if dep, ok := p.TaskMap[depID]; ok {
			if dep.Status != "completed" {
				return nil, fmt.Errorf("dependency %s not completed", depID)
			}
		}
	}

	task.Status = "in_progress"
	task.StartedAt = time.Now()
	p.Status = "executing"

	return task, nil
}

// CompleteTask 完成任务
func (p *Plan) CompleteTask(taskID string, result string) error {
	task, ok := p.TaskMap[taskID]
	if !ok {
		return fmt.Errorf("task not found: %s", taskID)
	}

	if task.Status != "in_progress" {
		return fmt.Errorf("task not in progress")
	}

	task.Status = "completed"
	task.Result = result
	task.CompletedAt = time.Now()

	// 检查是否全部完成
	if p.IsComplete() {
		p.Status = "completed"
		p.CompletedAt = time.Now()
	}

	return nil
}

// BlockTask 阻塞任务
func (p *Plan) BlockTask(taskID string, reason string) error {
	task, ok := p.TaskMap[taskID]
	if !ok {
		return fmt.Errorf("task not found: %s", taskID)
	}

	task.Status = "blocked"

	// 递归阻塞依赖此任务的任务
	for _, t := range p.Tasks {
		for _, depID := range t.Dependencies {
			if depID == taskID && t.Status == "pending" {
				t.Status = "blocked"
			}
		}
	}

	return nil
}

// IsComplete 检查是否全部完成
func (p *Plan) IsComplete() bool {
	for _, task := range p.Tasks {
		if task.Status != "completed" {
			return false
		}
	}
	return true
}

// GetProgress 获取进度
func (p *Plan) GetProgress() map[string]interface{} {
	total := len(p.Tasks)
	completed := 0
	inProgress := 0
	blocked := 0
	pending := 0

	for _, task := range p.Tasks {
		switch task.Status {
		case "completed":
			completed++
		case "in_progress":
			inProgress++
		case "blocked":
			blocked++
		case "pending":
			pending++
		}
	}

	return map[string]interface{}{
		"total": total,
		"completed": completed,
		"in_progress": inProgress,
		"blocked": blocked,
		"pending": pending,
		"percentage": float64(completed) / float64(max(1, total)) * 100,
	}
}

// GetExecutionOrder 获取执行顺序
func (p *Plan) GetExecutionOrder() []string {
	var order []string
	for _, task := range p.Tasks {
		order = append(order, fmt.Sprintf("%s(%s)", task.ID, task.Description[:min(20, len(task.Description))]))
	}
	return order
}

// GetPlanSummary 获取计划摘要
func (p *Plan) GetSummary() string {
	var sb strings.Builder

	sb.WriteString(fmt.Sprintf("=== 计划 %s ===\n", p.ID))
	sb.WriteString(fmt.Sprintf("目标: %s\n", p.Goal))
	sb.WriteString(fmt.Sprintf("状态: %s\n", p.Status))

	progress := p.GetProgress()
	sb.WriteString(fmt.Sprintf("进度: %d/%d (%.1f%%)\n",
		progress["completed"], progress["total"], progress["percentage"]))

	sb.WriteString("\n执行顺序:\n")
	for i, task := range p.Tasks {
		statusIcon := map[string]string{
			"pending": "○",
			"in_progress": "◐",
			"completed": "●",
			"blocked": "✗",
		}[task.Status]
		sb.WriteString(fmt.Sprintf("  %d. %s %s\n", i+1, statusIcon, task.Description))
	}

	return sb.String()
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// ============ API ============

var planner *Planner

func init() {
	planner = NewPlanner()
}

type DecomposeRequest struct {
	Goal string `json:"goal"`
}

type StartTaskRequest struct {
	PlanID string `json:"plan_id"`
	TaskID string `json:"task_id"`
}

type CompleteTaskRequest struct {
	PlanID string `json:"plan_id"`
	TaskID string `json:"task_id"`
	Result string `json:"result"`
}

func decomposeHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req DecomposeRequest
	json.NewDecoder(r.Body).Decode(&req)

	if req.Goal == "" {
		json.NewEncoder(w).Encode(map[string]string{"error": "goal required"})
		return
	}

	plan := planner.DecomposeGoal(req.Goal)
	json.NewEncoder(w).Encode(plan)
}

func getPlanHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	id := r.URL.Query().Get("id")
	plan, ok := planner.Plans[id]
	if !ok {
		json.NewEncoder(w).Encode(map[string]string{"error": "plan not found"})
		return
	}
	json.NewEncoder(w).Encode(plan)
}

func startTaskHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req StartTaskRequest
	json.NewDecoder(r.Body).Decode(&req)

	plan, ok := planner.Plans[req.PlanID]
	if !ok {
		json.NewEncoder(w).Encode(map[string]string{"error": "plan not found"})
		return
	}

	task, err := plan.StartTask(req.TaskID)
	if err != nil {
		json.NewEncoder(w).Encode(map[string]string{"error": err.Error()})
		return
	}

	json.NewEncoder(w).Encode(task)
}

func completeTaskHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	var req CompleteTaskRequest
	json.NewDecoder(r.Body).Decode(&req)

	plan, ok := planner.Plans[req.PlanID]
	if !ok {
		json.NewEncoder(w).Encode(map[string]string{"error": "plan not found"})
		return
	}

	err := plan.CompleteTask(req.TaskID, req.Result)
	if err != nil {
		json.NewEncoder(w).Encode(map[string]string{"error": err.Error()})
		return
	}

	// 返回更新后的计划
	json.NewEncoder(w).Encode(map[string]interface{}{
		"plan": plan,
		"progress": plan.GetProgress(),
	})
}

func getNextTaskHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	id := r.URL.Query().Get("plan_id")
	plan, ok := planner.Plans[id]
	if !ok {
		json.NewEncoder(w).Encode(map[string]string{"error": "plan not found"})
		return
	}

	task := plan.GetNextExecutable()
	if task == nil {
		json.NewEncoder(w).Encode(map[string]string{"status": "no_more_tasks"})
		return
	}

	json.NewEncoder(w).Encode(task)
}

func getProgressHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	id := r.URL.Query().Get("plan_id")
	plan, ok := planner.Plans[id]
	if !ok {
		json.NewEncoder(w).Encode(map[string]string{"error": "plan not found"})
		return
	}

	json.NewEncoder(w).Encode(plan.GetProgress())
}

func getSummaryHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	id := r.URL.Query().Get("plan_id")
	plan, ok := planner.Plans[id]
	if !ok {
		json.NewEncoder(w).Encode(map[string]string{"error": "plan not found"})
		return
	}

	json.NewEncoder(w).Encode(map[string]string{"summary": plan.GetSummary()})
}

func plannerHealthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status": "ok",
		"service": "planner",
		"active_plans": len(planner.Plans),
	})
}

func mainPlannerServer() {
	planner = NewPlanner()

	mux := http.NewServeMux()
	mux.HandleFunc("/decompose", decomposeHandler)
	mux.HandleFunc("/get", getPlanHandler)
	mux.HandleFunc("/start_task", startTaskHandler)
	mux.HandleFunc("/complete_task", completeTaskHandler)
	mux.HandleFunc("/next_task", getNextTaskHandler)
	mux.HandleFunc("/progress", getProgressHandler)
	mux.HandleFunc("/summary", getSummaryHandler)
	mux.HandleFunc("/health", plannerHealthHandler)

	fmt.Println("[规划器] 服务启动在 :8098")
	fmt.Println("  /decompose   - 分解目标")
	fmt.Println("  /get         - 获取计划")
	fmt.Println("  /start_task  - 开始任务")
	fmt.Println("  /complete_task - 完成子任务")
	fmt.Println("  /next_task   - 获取下一个可执行任务")
	fmt.Println("  /progress    - 获取进度")
	http.ListenAndServe(":8098", mux)
}

func main() {
	mainPlannerServer()
}
