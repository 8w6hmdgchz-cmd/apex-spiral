package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"time"
)

// ============ 具身智能核心结构 ============

type EmbodiedSystem struct {
	port           int
	sensors        *SensorHub
	actuators      *ActuatorHub
	闭环反馈       chan *SensorEvent
	eventQueue     chan *AGIEvent
	processCache   map[int]*ProcessInfo
	fileStates     map[string]os.FileInfo
	cacheMu        sync.RWMutex
	stopChan       chan struct{}
	wg             sync.WaitGroup
}

type SensorHub struct {
	processTicker *time.Ticker
	timeTicker    *time.Ticker
	stopChan      chan struct{}
}

type ActuatorHub struct {
	httpClient *http.Client
}

type SensorEvent struct {
	Type      string      `json:"type"`       // file_created, file_modified, process_update, time_tick
	Timestamp time.Time   `json:"timestamp"`
	Data      interface{} `json:"data"`
}

type ProcessInfo struct {
	PID     int     `json:"pid"`
	Name    string  `json:"name"`
	CPU     float64 `json:"cpu_percent"`
	Memory  float64 `json:"memory_percent"`
	RSS     uint64  `json:"rss_bytes"`
	Updated time.Time `json:"updated"`
}

type AGIEvent struct {
	SensorEvent *SensorEvent `json:"sensor_event"`
	Decision    string       `json:"decision"`
	Action      *Action      `json:"action"`
	Result      string       `json:"result"`
}

type Action struct {
	Type    string `json:"type"`    // file_write, command_exec, http_request
	Target  string `json:"target"`
	Payload interface{} `json:"payload"`
}

type FileEvent struct {
	Path    string `json:"path"`
	Op      string `json:"op"`      // CREATE, WRITE, REMOVE, RENAME
	Content string `json:"content,omitempty"`
}

type TimeEvent struct {
	Interval string `json:"interval"`
	Count    int    `json:"count"`
}

type CommandResult struct {
	Command   string `json:"command"`
	Output    string `json:"output"`
	Error     string `json:"error,omitempty"`
	ExitCode  int    `json:"exit_code"`
	StartTime time.Time `json:"start_time"`
	EndTime   time.Time `json:"end_time"`
}

type HTTPRequest struct {
	Method  string            `json:"method"`
	URL     string            `json:"url"`
	Headers map[string]string `json:"headers,omitempty"`
	Body    string            `json:"body,omitempty"`
	Timeout int               `json:"timeout_seconds"`
}

type HTTPResponse struct {
	StatusCode int               `json:"status_code"`
	Headers    map[string]string `json:"headers"`
	Body       string            `json:"body"`
	Error      string            `json:"error,omitempty"`
}

// ============ 初始化 ============

func NewEmbodiedSystem(port int) *EmbodiedSystem {
	return &EmbodiedSystem{
		port:         port,
		sensors:      &SensorHub{},
		actuators:    &ActuatorHub{
			httpClient: &http.Client{Timeout: 30 * time.Second},
		},
		闭环反馈:    make(chan *SensorEvent, 100),
		eventQueue: make(chan *AGIEvent, 100),
		processCache: make(map[int]*ProcessInfo),
		fileStates:   make(map[string]os.FileInfo),
		stopChan:     make(chan struct{}),
	}
}

func (e *EmbodiedSystem) Start() error {
	fmt.Printf("[具身智能] 启动中，端口: %d\n", e.port)
	
	// 启动传感器
	e.startFileWatcher()
	e.startProcessMonitor()
	e.startTimeSensor()
	
	// 启动HTTP服务器
	go e.startHTTPServer()
	
	// 启动闭环处理
	go e.feedbackLoop()
	
	fmt.Println("[具身智能] 系统已就绪")
	return nil
}

func (e *EmbodiedSystem) Stop() {
	close(e.stopChan)
	fmt.Println("[具身智能] 系统已关闭")
}

// ============ 传感器实现 ============

// 文件系统监控 (基于轮询)
func (e *EmbodiedSystem) startFileWatcher() {
	// 初始化文件状态
	paths := []string{
		os.Getenv("HOME") + "/Desktop/开智",
		os.Getenv("HOME") + "/Downloads",
		"/tmp",
	}
	
	for _, p := range paths {
		if info, err := os.Stat(p); err == nil && info.IsDir() {
			e.scanDirectory(p)
		}
	}
	
	e.sensors.stopChan = make(chan struct{})
	e.wg.Add(1)
	go func() {
		defer e.wg.Done()
		ticker := time.NewTicker(10 * time.Second)
		defer ticker.Stop()
		
		for {
			select {
			case <-ticker.C:
				e.checkFileChanges()
			case <-e.sensors.stopChan:
				return
			case <-e.stopChan:
				return
			}
		}
	}()
	
	fmt.Println("[传感器] 文件系统监控已启动")
}

func (e *EmbodiedSystem) scanDirectory(dir string) {
	filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil || info == nil {
			return nil
		}
		if !info.IsDir() {
			e.fileStates[path] = info
		}
		return nil
	})
}

func (e *EmbodiedSystem) checkFileChanges() {
	paths := []string{
		os.Getenv("HOME") + "/Desktop/开智",
		os.Getenv("HOME") + "/Downloads",
		"/tmp",
	}
	
	for _, dir := range paths {
		if info, err := os.Stat(dir); err != nil || !info.IsDir() {
			continue
		}
		
		filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
			if err != nil || info == nil {
				return nil
			}
			
			if info.IsDir() {
				return nil
			}
			
			e.cacheMu.Lock()
			oldInfo, exists := e.fileStates[path]
			
			if !exists {
				// 新文件
				e.fileStates[path] = info
				e.cacheMu.Unlock()
				e.handleFileEvent(path, "CREATE")
			} else if info.Size() != oldInfo.Size() || info.ModTime().After(oldInfo.ModTime()) {
				// 文件修改
				e.fileStates[path] = info
				e.cacheMu.Unlock()
				e.handleFileEvent(path, "WRITE")
			} else {
				e.cacheMu.Unlock()
			}
			return nil
		})
	}
}

func (e *EmbodiedSystem) handleFileEvent(path string, op string) {
	sensorEvent := &SensorEvent{
		Type:      "file_" + strings.ToLower(op),
		Timestamp: time.Now(),
		Data: FileEvent{
			Path: path,
			Op:   op,
		},
	}
	
	fmt.Printf("[传感器] 文件事件: %s -> %s\n", op, path)
	select {
	case e.闭环反馈 <- sensorEvent:
		e.eventQueue <- &AGIEvent{SensorEvent: sensorEvent}
	default:
	}
}

// 进程监控
func (e *EmbodiedSystem) startProcessMonitor() {
	e.sensors.processTicker = time.NewTicker(5 * time.Second)
	e.wg.Add(1)
	go func() {
		defer e.wg.Done()
		for {
			select {
			case <-e.sensors.processTicker.C:
				e.updateProcessInfo()
			case <-e.stopChan:
				return
			}
		}
	}()
}

func (e *EmbodiedSystem) updateProcessInfo() {
	if runtime.GOOS != "linux" && runtime.GOOS != "darwin" {
		return
	}
	
	cmd := exec.Command("ps", "-eo", "pid,comm,%cpu,%mem,rss")
	output, err := cmd.Output()
	if err != nil {
		return
	}
	
	lines := strings.Split(string(output), "\n")
	e.cacheMu.Lock()
	defer e.cacheMu.Unlock()
	
	for i, line := range lines[1:] {
		if i >= 20 { // 只监控前20个进程
			break
		}
		fields := strings.Fields(line)
		if len(fields) < 5 {
			continue
		}
		
		pid, _ := strconv.Atoi(fields[0])
		cpu, _ := strconv.ParseFloat(fields[2], 64)
		mem, _ := strconv.ParseFloat(fields[3], 64)
		rss, _ := strconv.ParseUint(fields[4], 10, 64)
		
		e.processCache[pid] = &ProcessInfo{
			PID:     pid,
			Name:    fields[1],
			CPU:     cpu,
			Memory:  mem,
			RSS:     rss * 1024, // KB to bytes
			Updated: time.Now(),
		}
	}
	
	// 发送进程更新事件
	sensorEvent := &SensorEvent{
		Type:      "process_update",
		Timestamp: time.Now(),
		Data:      e.getTopProcesses(5),
	}
	select {
	case e.闭环反馈 <- sensorEvent:
		e.eventQueue <- &AGIEvent{SensorEvent: sensorEvent}
	default:
	}
}

func (e *EmbodiedSystem) getTopProcesses(n int) []*ProcessInfo {
	e.cacheMu.RLock()
	defer e.cacheMu.RUnlock()
	
	procs := make([]*ProcessInfo, 0, len(e.processCache))
	for _, p := range e.processCache {
		procs = append(procs, p)
	}
	
	// 按CPU使用率排序
	for i := 0; i < len(procs)-1; i++ {
		for j := i + 1; j < len(procs); j++ {
			if procs[j].CPU > procs[i].CPU {
				procs[i], procs[j] = procs[j], procs[i]
			}
		}
	}
	
	if len(procs) > n {
		procs = procs[:n]
	}
	return procs
}

// 时间感知
func (e *EmbodiedSystem) startTimeSensor() {
	e.sensors.timeTicker = time.NewTicker(1 * time.Minute)
	e.wg.Add(1)
	go func() {
		defer e.wg.Done()
		count := 0
		for {
			select {
			case <-e.sensors.timeTicker.C:
				count++
				sensorEvent := &SensorEvent{
					Type:      "time_tick",
					Timestamp: time.Now(),
					Data: TimeEvent{
						Interval: "1m",
						Count:    count,
					},
				}
				select {
				case e.闭环反馈 <- sensorEvent:
					e.eventQueue <- &AGIEvent{SensorEvent: sensorEvent}
				default:
				}
			case <-e.stopChan:
				return
			}
		}
	}()
}

// ============ 执行器实现 ============

// 文件操作
func (a *ActuatorHub) WriteFile(path string, content string) error {
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("创建目录失败: %w", err)
	}
	
	if err := os.WriteFile(path, []byte(content), 0644); err != nil {
		return fmt.Errorf("写入文件失败: %w", err)
	}
	
	fmt.Printf("[执行器] 文件写入: %s\n", path)
	return nil
}

func (a *ActuatorHub) ReadFile(path string) (string, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return "", fmt.Errorf("读取文件失败: %w", err)
	}
	return string(content), nil
}

func (a *ActuatorHub) AppendFile(path string, content string) error {
	f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return fmt.Errorf("打开文件失败: %w", err)
	}
	defer f.Close()
	
	if _, err := f.WriteString(content); err != nil {
		return fmt.Errorf("追加内容失败: %w", err)
	}
	
	fmt.Printf("[执行器] 文件追加: %s\n", path)
	return nil
}

// 命令执行
func (a *ActuatorHub) ExecuteCommand(command string, args ...string) *CommandResult {
	result := &CommandResult{
		Command:   command,
		StartTime: time.Now(),
	}
	
	cmd := exec.Command(command, args...)
	output, err := cmd.CombinedOutput()
	
	result.EndTime = time.Now()
	result.Output = string(output)
	
	if err != nil {
		result.Error = err.Error()
		if exitErr, ok := err.(*exec.ExitError); ok {
			result.ExitCode = exitErr.ExitCode()
		}
	}
	
	fmt.Printf("[执行器] 命令执行: %s, 退出码: %d\n", command, result.ExitCode)
	return result
}

func (a *ActuatorHub) ExecuteShell(shellCmd string) *CommandResult {
	result := &CommandResult{
		Command:   shellCmd,
		StartTime: time.Now(),
	}
	
	var shell string
	if runtime.GOOS == "windows" {
		shell = "cmd"
	} else {
		shell = "/bin/sh"
	}
	
	cmd := exec.Command(shell, "-c", shellCmd)
	output, err := cmd.CombinedOutput()
	
	result.EndTime = time.Now()
	result.Output = string(output)
	
	if err != nil {
		result.Error = err.Error()
		if exitErr, ok := err.(*exec.ExitError); ok {
			result.ExitCode = exitErr.ExitCode()
		}
	}
	
	preview := shellCmd
	if len(preview) > 50 {
		preview = preview[:50] + "..."
	}
	fmt.Printf("[执行器] Shell执行: %s, 退出码: %d\n", preview, result.ExitCode)
	return result
}

// HTTP请求
func (a *ActuatorHub) DoHTTPRequest(req *HTTPRequest) *HTTPResponse {
	response := &HTTPResponse{}
	
	client := &http.Client{Timeout: time.Duration(req.Timeout) * time.Second}
	
	var body io.Reader
	if req.Body != "" {
		body = strings.NewReader(req.Body)
	}
	
	httpReq, err := http.NewRequest(req.Method, req.URL, body)
	if err != nil {
		response.Error = err.Error()
		return response
	}
	
	for k, v := range req.Headers {
		httpReq.Header.Set(k, v)
	}
	
	resp, err := client.Do(httpReq)
	if err != nil {
		response.Error = err.Error()
		return response
	}
	defer resp.Body.Close()
	
	response.StatusCode = resp.StatusCode
	response.Headers = make(map[string]string)
	for k, v := range resp.Header {
		if len(v) > 0 {
			response.Headers[k] = v[0]
		}
	}
	
	respBody, _ := io.ReadAll(resp.Body)
	response.Body = string(respBody)
	
	fmt.Printf("[执行器] HTTP请求: %s %s -> %d\n", req.Method, req.URL, resp.StatusCode)
	return response
}

// ============ 反馈闭环 ============

func (e *EmbodiedSystem) feedbackLoop() {
	e.wg.Add(1)
	defer e.wg.Done()
	
	for {
		select {
		case event := <-e.eventQueue:
			e.processAGIEvent(event)
		case <-e.stopChan:
			return
		}
	}
}

func (e *EmbodiedSystem) processAGIEvent(event *AGIEvent) {
	// 简单的规则匹配决策
	switch event.SensorEvent.Type {
	case "file_create":
		data, ok := event.SensorEvent.Data.(FileEvent)
		if ok && strings.HasSuffix(data.Path, ".go") {
			event.Decision = "分析新的Go文件"
			event.Action = &Action{
				Type:    "command_exec",
				Target:  "gofmt",
				Payload: data.Path,
			}
		}
	case "process_update":
		procs, ok := event.SensorEvent.Data.([]*ProcessInfo)
		if ok && len(procs) > 0 && procs[0].CPU > 80 {
			event.Decision = fmt.Sprintf("高CPU警告: %s (%.1f%%)", procs[0].Name, procs[0].CPU)
		}
	case "time_tick":
		event.Decision = "定时检查系统状态"
	}
	
	// 执行动作
	if event.Action != nil {
		e.executeAction(event)
	}
}

func (e *EmbodiedSystem) executeAction(event *AGIEvent) {
	switch event.Action.Type {
	case "file_write":
		payload, ok := event.Action.Payload.(map[string]interface{})
		if !ok {
			event.Result = "无效的payload"
			return
		}
		path, ok := payload["path"].(string)
		if !ok {
			event.Result = "无效的path"
			return
		}
		content, ok := payload["content"].(string)
		if !ok {
			event.Result = "无效的content"
			return
		}
		if err := e.actuators.WriteFile(path, content); err != nil {
			event.Result = fmt.Sprintf("失败: %v", err)
		} else {
			event.Result = "成功"
		}
	case "command_exec":
		cmdStr, ok := event.Action.Payload.(string)
		if !ok {
			event.Result = "无效的command"
			return
		}
		result := e.actuators.ExecuteShell(cmdStr)
		event.Result = result.Output
	case "http_request":
		urlStr, ok := event.Action.Payload.(string)
		if !ok {
			event.Result = "无效的url"
			return
		}
		req := &HTTPRequest{
			Method:  "GET",
			URL:     urlStr,
			Timeout: 30,
		}
		resp := e.actuators.DoHTTPRequest(req)
		event.Result = fmt.Sprintf("状态: %d", resp.StatusCode)
	}
	
	fmt.Printf("[闭环] 决策: %s, 结果: %s\n", event.Decision, event.Result)
}

// ============ HTTP API ============

func (e *EmbodiedSystem) startHTTPServer() {
	mux := http.NewServeMux()
	
	// 健康检查
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
	})
	
	// 获取传感器数据
	mux.HandleFunc("/api/sensors/processes", func(w http.ResponseWriter, r *http.Request) {
		e.cacheMu.RLock()
		procs := make([]*ProcessInfo, 0, len(e.processCache))
		for _, p := range e.processCache {
			procs = append(procs, p)
		}
		e.cacheMu.RUnlock()
		json.NewEncoder(w).Encode(procs)
	})
	
	mux.HandleFunc("/api/sensors/events", func(w http.ResponseWriter, r *http.Request) {
		events := make([]*SensorEvent, 0)
		for len(e.闭环反馈) > 0 {
			select {
			case ev := <-e.闭环反馈:
				events = append(events, ev)
			default:
				break
			}
		}
		json.NewEncoder(w).Encode(events)
	})
	
	// 执行器API
	mux.HandleFunc("/api/actuator/file/write", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}
		
		var req struct {
			Path    string `json:"path"`
			Content string `json:"content"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		
		if err := e.actuators.WriteFile(req.Path, req.Content); err != nil {
			json.NewEncoder(w).Encode(map[string]string{"error": err.Error()})
		} else {
			json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
		}
	})
	
	mux.HandleFunc("/api/actuator/file/read", func(w http.ResponseWriter, r *http.Request) {
		path := r.URL.Query().Get("path")
		if path == "" {
			http.Error(w, "path required", http.StatusBadRequest)
			return
		}
		
		content, err := e.actuators.ReadFile(path)
		if err != nil {
			json.NewEncoder(w).Encode(map[string]string{"error": err.Error()})
		} else {
			json.NewEncoder(w).Encode(map[string]string{"content": content})
		}
	})
	
	mux.HandleFunc("/api/actuator/command", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}
		
		var req struct {
			Command string `json:"command"`
			Shell   bool   `json:"shell"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		
		var result *CommandResult
		if req.Shell {
			result = e.actuators.ExecuteShell(req.Command)
		} else {
			parts := strings.Fields(req.Command)
			if len(parts) == 0 {
				http.Error(w, "empty command", http.StatusBadRequest)
				return
			}
			result = e.actuators.ExecuteCommand(parts[0], parts[1:]...)
		}
		
		json.NewEncoder(w).Encode(result)
	})
	
	mux.HandleFunc("/api/actuator/http", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}
		
		var req HTTPRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		
		if req.Timeout == 0 {
			req.Timeout = 30
		}
		
		resp := e.actuators.DoHTTPRequest(&req)
		json.NewEncoder(w).Encode(resp)
	})
	
	addr := fmt.Sprintf(":%d", e.port)
	fmt.Printf("[具身智能] HTTP服务器启动: http://localhost%s\n", addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		fmt.Printf("[具身智能] HTTP服务器错误: %v\n", err)
	}
}

// ============ 主函数 ============

func main() {
	system := NewEmbodiedSystem(8101)
	
	if err := system.Start(); err != nil {
		fmt.Printf("[具身智能] 启动失败: %v\n", err)
		os.Exit(1)
	}
	
	fmt.Println("[具身智能] 按Ctrl+C停止")
	
	// 阻塞
	<-make(chan struct{})
}
