// apex_token_optimizer.go - APEX Token 优化核心
// 解决三类原生工程缺陷：
// 1. 截图缩放导致坐标偏移
// 2. 单帧截图Token过高导致上下文溢出
// 3. 无效思维开销造成算力空耗
package main

import (
	"encoding/json"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
)

// ============ 配置 ============

type Config struct {
	MaxScreenshotFrames int     `json:"max_screenshot_frames"` // 默认3
	ScreenWidth         float64 `json:"screen_width"`
	ScreenHeight        float64 `json:"screen_height"`
	EnablePurification  bool    `json:"enable_purification"`
	PurifyIntervalSec   int     `json:"purify_interval_sec"` // 25步净化周期
}

var defaultConfig = Config{
	MaxScreenshotFrames: 3,
	ScreenWidth:         1920.0,
	ScreenHeight:        1080.0,
	EnablePurification: true,
	PurifyIntervalSec:   900, // 15分钟一个净化周期，25步=约6小时
}

// ============ 坐标校正 ============

type ScreenshotInfo struct {
	Width       float64 `json:"width"`        // 截图宽度
	Height      float64 `json:"height"`       // 截图高度
	ScaleX      float64 `json:"scale_x"`      // X缩放比例
	ScaleY      float64 `json:"scale_y"`      // Y缩放比例
	Path        string  `json:"path"`         // 截图路径
	Timestamp   int64   `json:"timestamp"`    // 时间戳
	TokenCost   int     `json:"token_cost"`   // Token消耗
}

type CoordinateCorrection struct {
	ScreenWidth  float64 `json:"screen_width"`
	ScreenHeight float64 `json:"screen_height"`
}

// NewCoordinateCorrection 创坐标校正器
func NewCoordinateCorrection(w, h float64) *CoordinateCorrection {
	return &CoordinateCorrection{
		ScreenWidth:  w,
		ScreenHeight: h,
	}
}

// Correct 坐标校正核心公式
// X_real = X_out × (W_screen / W_img)
// Y_real = Y_out × (H_screen / H_img)
func (c *CoordinateCorrection) Correct(x, y, imgWidth, imgHeight float64) (float64, float64) {
	scaleX := c.ScreenWidth / imgWidth
	scaleY := c.ScreenHeight / imgHeight
	return x * scaleX, y * scaleY
}

// CorrectFromScreenshot 从截图信息校正坐标
func (c *CoordinateCorrection) CorrectFromScreenshot(x, y float64, shot *ScreenshotInfo) (float64, float64) {
	return c.Correct(x, y, shot.Width, shot.Height)
}

// ============ 上下文Token控制器 ============

type TokenStats struct {
	TextTokenCount int `json:"text_token_count"`
	ImageTokenCount int `json:"image_token_count"`
	TotalTokenCount int `json:"total_token_count"`
	ReservedTokenCount int `json:"reserved_token_count"`
	DroppedFrames int `json:"dropped_frames"`
}

type ContextController struct {
	config         Config
	mutex          sync.RWMutex
	screenshotLog  []*ScreenshotInfo
	textTokenCount int
	totalEffort   float64
	wasteEffort   float64
}

// NewContextController 创建上下文控制器
func NewContextController(cfg Config) *ContextController {
	return &ContextController{
		config:        cfg,
		screenshotLog: make([]*ScreenshotInfo, 0),
	}
}

// AddScreenshot 添加截图（自动保留最新N帧）
// Token_reserve = Token_text + ΣToken_img(n) for n=N-2 to N
func (cc *ContextController) AddScreenshot(info *ScreenshotInfo) {
	cc.mutex.Lock()
	defer cc.mutex.Unlock()

	cc.screenshotLog = append(cc.screenshotLog, info)
	
	// 仅保留最新 MaxScreenshotFrames 帧
	if len(cc.screenshotLog) > cc.config.MaxScreenshotFrames {
		dropped := cc.screenshotLog[:len(cc.screenshotLog)-cc.config.MaxScreenshotFrames]
		cc.screenshotLog = cc.screenshotLog[len(cc.screenshotLog)-cc.config.MaxScreenshotFrames:]
		
		// 记录被丢弃的帧
		for range dropped {
			info.TokenCost = 0 // 标记为已释放
		}
	}
}

// SetTextTokens 设置文本Token数
func (cc *ContextController) SetTextTokens(count int) {
	cc.mutex.Lock()
	defer cc.mutex.Unlock()
	cc.textTokenCount = count
}

// GetTokenStats 获取Token统计
func (cc *ContextController) GetTokenStats() TokenStats {
	cc.mutex.RLock()
	defer cc.mutex.RUnlock()

	stats := TokenStats{
		TextTokenCount: cc.textTokenCount,
	}

	var imgTokens int
	for _, shot := range cc.screenshotLog {
		imgTokens += shot.TokenCost
		stats.ReservedTokenCount += shot.TokenCost
	}
	stats.ImageTokenCount = imgTokens
	stats.TotalTokenCount = cc.textTokenCount + imgTokens

	return stats
}

// GetReservedTokenCount 计算保留的Token数
// Token_reserve = Token_text + ΣToken_img(n) for n=N-2 to N
func (cc *ContextController) GetReservedTokenCount() int {
	stats := cc.GetTokenStats()
	return stats.ReservedTokenCount + stats.TextTokenCount
}

// GetLatestScreenshots 获取最新N帧截图
func (cc *ContextController) GetLatestScreenshots() []*ScreenshotInfo {
	cc.mutex.RLock()
	defer cc.mutex.RUnlock()

	result := make([]*ScreenshotInfo, len(cc.screenshotLog))
	copy(result, cc.screenshotLog)
	return result
}

// ============ 算力有效率 ============

type EffortStats struct {
	TotalEffort  float64 `json:"total_effort"`
	WasteEffort  float64 `json:"waste_effort"`
	ValidEffort  float64 `json:"valid_effort"`
	Efficiency   float64 `json:"efficiency"` // 0-1
}

type EffortTracker struct {
	mutex        sync.RWMutex
	totalEffort  float64
	wasteEffort  float64
	invalidTypes map[string]float64 // 各类无效开销
}

func NewEffortTracker() *EffortTracker {
	return &EffortTracker{
		invalidTypes: make(map[string]float64),
	}
}

// AddEffort 添加总开销
func (et *EffortTracker) AddEffort(amount float64) {
	et.mutex.Lock()
	defer et.mutex.Unlock()
	et.totalEffort += amount
}

// AddWaste 添加无效开销
func (et *EffortTracker) AddWaste(amount float64, wasteType string) {
	et.mutex.Lock()
	defer et.mutex.Unlock()
	et.wasteEffort += amount
	et.invalidTypes[wasteType] += amount
}

// GetStats 计算算力有效率
// Effort_valid = Total_effort - Waste_effort
func (et *EffortTracker) GetStats() EffortStats {
	et.mutex.RLock()
	defer et.mutex.RUnlock()

	valid := et.totalEffort - et.wasteEffort
	efficiency := 0.0
	if et.totalEffort > 0 {
		efficiency = valid / et.totalEffort
	}

	return EffortStats{
		TotalEffort: et.totalEffort,
		WasteEffort: et.wasteEffort,
		ValidEffort: valid,
		Efficiency:  efficiency,
	}
}

// GetInvalidTypes 获取无效开销分类
func (et *EffortTracker) GetInvalidTypes() map[string]float64 {
	et.mutex.RLock()
	defer et.mutex.RUnlock()

	result := make(map[string]float64)
	for k, v := range et.invalidTypes {
		result[k] = v
	}
	return result
}

// ============ 25步周期性净化策略 ============

type PurificationStats struct {
	PurgedScreenshots int `json:"purged_screenshots"`
	PurgedTokens      int `json:"purged_tokens"`
	CacheCleanedMB    float64 `json:"cache_cleaned_mb"`
	DurationMs        int64   `json:"duration_ms"`
}

type ScreenshotPurifier struct {
	config      Config
	cacheDir    string
	screenshotDir string
	step        int // 当前净化步数（0-24）
	lastPurify  time.Time
}

func NewScreenshotPurifier(cfg Config, cacheDir, screenshotDir string) *ScreenshotPurifier {
	return &ScreenshotPurifier{
		config:        cfg,
		cacheDir:      cacheDir,
		screenshotDir: screenshotDir,
		step:          0,
		lastPurify:    time.Now(),
	}
}

// Purify 执行25步净化策略
func (sp *ScreenshotPurifier) Purify() PurificationStats {
	start := time.Now()
	stats := PurificationStats{}

	// 25步净化策略
	switch sp.step {
	case 0, 5, 10, 15, 20:
		// 步骤0/5/10/15/20: 清理过期截图（>24小时）
		purged, tokens := sp.purgeOldScreenshots(24 * time.Hour)
		stats.PurgedScreenshots += purged
		stats.PurgedTokens += tokens
	case 1, 6, 11, 16, 21:
		// 步骤1/6/11/16/21: 清理临时缓存文件
		stats.CacheCleanedMB += sp.cleanTempCache()
	case 2, 7, 12, 17, 22:
		// 步骤2/7/12/17/22: 清理对话缓存
		stats.CacheCleanedMB += sp.cleanConversationCache()
	case 3, 8, 13, 18, 23:
		// 步骤3/8/13/18/23: 清理重复截图
		purged, tokens := sp.purgeDuplicateScreenshots()
		stats.PurgedScreenshots += purged
		stats.PurgedTokens += tokens
	case 4, 9, 14, 19, 24:
		// 步骤4/9/14/19/24: 压缩早间截图
		stats.CacheCleanedMB += sp.compressOldScreenshots()
	}

	sp.step = (sp.step + 1) % 25
	sp.lastPurify = time.Now()
	stats.DurationMs = time.Since(start).Milliseconds()

	return stats
}

func (sp *ScreenshotPurifier) purgeOldScreenshots(maxAge time.Duration) (int, int) {
	count, tokens := 0, 0
	now := time.Now()

	filepath.Walk(sp.screenshotDir, func(path string, info os.FileInfo, err error) error {
		if err != nil || info.IsDir() {
			return nil
		}
		if strings.HasSuffix(path, ".png") || strings.HasSuffix(path, ".jpg") {
			if now.Sub(info.ModTime()) > maxAge {
				tokens += sp.estimateTokenSize(path)
				os.Remove(path)
				count++
			}
		}
		return nil
	})

	return count, tokens
}

func (sp *ScreenshotPurifier) cleanTempCache() float64 {
	var cleaned float64

	filepath.Walk(sp.cacheDir, func(path string, info os.FileInfo, err error) error {
		if err != nil || info.IsDir() {
			return nil
		}
		// 清理临时文件（以 .tmp, .cache 结尾）
		if strings.HasSuffix(path, ".tmp") || strings.HasSuffix(path, ".cache") {
			cleaned += float64(info.Size()) / (1024 * 1024)
			os.Remove(path)
		}
		return nil
	})

	return cleaned
}

func (sp *ScreenshotPurifier) cleanConversationCache() float64 {
	// 清理对话缓存目录
	cachePath := filepath.Join(sp.cacheDir, "conversations")
	var cleaned float64

	filepath.Walk(cachePath, func(path string, info os.FileInfo, err error) error {
		if err != nil || info.IsDir() {
			return nil
		}
		// 保留最近7天，清理更早的
		if time.Since(info.ModTime()) > 7*24*time.Hour {
			cleaned += float64(info.Size()) / (1024 * 1024)
			os.Remove(path)
		}
		return nil
		return nil
	})

	return cleaned
}

func (sp *ScreenshotPurifier) purgeDuplicateScreenshots() (int, int) {
	// 简单去重：基于文件大小和修改时间
	seen := make(map[string]bool)
	count, tokens := 0, 0

	filepath.Walk(sp.screenshotDir, func(path string, info os.FileInfo, err error) error {
		if err != nil || info.IsDir() {
			return nil
		}
		key := fmt.Sprintf("%d_%d", info.Size(), info.ModTime().Unix())
		if seen[key] {
			tokens += sp.estimateTokenSize(path)
			os.Remove(path)
			count++
		} else {
			seen[key] = true
		}
		return nil
	})

	return count, tokens
}

func (sp *ScreenshotPurifier) compressOldScreenshots() float64 {
	// 压缩早间截图（可集成 jpegoptim / pngquant）
	// 这里只是占位，实际压缩需要调用外部工具
	return 0.0
}

func (sp *ScreenshotPurifier) estimateTokenSize(path string) int {
	// 估算截图Token消耗（简化估算：每MB约150 Token）
	info, err := os.Stat(path)
	if err != nil {
		return 150 // 默认值
	}
	return int(float64(info.Size()) / (1024 * 1024) * 150)
}

// ============ 轨迹日志分析 ============

type TrajectoryLog struct {
	Entries []TrajectoryEntry `json:"entries"`
}

type TrajectoryEntry struct {
	Timestamp    int64   `json:"timestamp"`
	ActionType   string  `json:"action_type"`   // click, scroll, type, screenshot
	X            float64 `json:"x"`             // 原始坐标
	Y            float64 `json:"y"`
	XCorrected   float64 `json:"x_corrected"`  // 校正后坐标
	YCorrected   float64 `json:"y_corrected"`
	ImageWidth   float64 `json:"image_width"`
	ImageHeight  float64 `json:"image_height"`
	TokenCost    int     `json:"token_cost"`
	Error        float64 `json:"error"`         // 坐标误差
	IsValid      bool    `json:"is_valid"`
}

func (l *TrajectoryLog) Add(entry TrajectoryEntry) {
	l.Entries = append(l.Entries, entry)
}

func (l *TrajectoryLog) GetCoordStats() (avgError, maxError float64) {
	if len(l.Entries) == 0 {
		return 0, 0
	}

	var totalErr, maxErr float64
	validCount := 0

	for _, e := range l.Entries {
		if e.IsValid {
			totalErr += e.Error
			maxErr = math.Max(maxErr, e.Error)
			validCount++
		}
	}

	var avgErr float64
	if validCount > 0 {
		avgErr = totalErr / float64(validCount)
	}
	return avgErr, maxErr
}

func (l *TrajectoryLog) GetContextOverflows() int {
	count := 0
	for _, e := range l.Entries {
		if e.TokenCost > 1500 { // 超过1500 Token视为溢出风险
			count++
		}
	}
	return count
}

// ============ 优化器主控制器 ============

type Optimizer struct {
	config        Config
	coordCorrect  *CoordinateCorrection
	ctxController *ContextController
	effortTracker *EffortTracker
	purifier      *ScreenshotPurifier
	trajLog       *TrajectoryLog
}

func NewOptimizer(cfg Config) *Optimizer {
	workDir := os.Getenv("APEX_WORKDIR")
	if workDir == "" {
		workDir = "/Users/lihongxin/.openclaw/workspace"
	}

	return &Optimizer{
		config:        cfg,
		coordCorrect:  NewCoordinateCorrection(cfg.ScreenWidth, cfg.ScreenHeight),
		ctxController: NewContextController(cfg),
		effortTracker: NewEffortTracker(),
		purifier:      NewScreenshotPurifier(cfg, filepath.Join(workDir, "cache"), filepath.Join(workDir, "screenshots")),
		trajLog:       &TrajectoryLog{},
	}
}

// OptimizeClick 优化点击坐标
func (o *Optimizer) OptimizeClick(x, y, imgWidth, imgHeight float64) (float64, float64) {
	correctedX, correctedY := o.coordCorrect.Correct(x, y, imgWidth, imgHeight)
	
	// 记录轨迹
	o.trajLog.Add(TrajectoryEntry{
		Timestamp:   time.Now().Unix(),
		ActionType:  "click",
		X:           x,
		Y:           y,
		XCorrected:  correctedX,
		YCorrected:  correctedY,
		ImageWidth:  imgWidth,
		ImageHeight: imgHeight,
		IsValid:     true,
	})

	return correctedX, correctedY
}

// ProcessScreenshot 处理截图（控制Token）
func (o *Optimizer) ProcessScreenshot(path string, width, height float64, tokenCost int) {
	o.ctxController.AddScreenshot(&ScreenshotInfo{
		Width:     width,
		Height:    height,
		Path:      path,
		Timestamp: time.Now().Unix(),
		TokenCost: tokenCost,
	})
}

// TrackEffort 追踪算力开销
func (o *Optimizer) TrackEffort(total, waste float64, wasteType string) {
	o.effortTracker.AddEffort(total)
	if waste > 0 {
		o.effortTracker.AddWaste(waste, wasteType)
	}
}

// PurifyIfNeeded 执行净化（按25步周期）
func (o *Optimizer) PurifyIfNeeded() PurificationStats {
	return o.purifier.Purify()
}

// GetStats 获取完整统计
type OptimizerStats struct {
	Token         TokenStats   `json:"token"`
	Effort        EffortStats  `json:"effort"`
	CoordAccuracy float64      `json:"coord_accuracy"` // 1 - avgError
	OverflowRisk  int          `json:"overflow_risk"`
}

func (o *Optimizer) GetStats() OptimizerStats {
	avgErr, _ := o.trajLog.GetCoordStats()
	coordAcc := 1.0 - avgErr
	if coordAcc < 0 {
		coordAcc = 0
	}

	return OptimizerStats{
		Token:        o.ctxController.GetTokenStats(),
		Effort:       o.effortTracker.GetStats(),
		CoordAccuracy: coordAcc,
		OverflowRisk:  o.trajLog.GetContextOverflows(),
	}
}

// ============ CLI 接口 ============

func main() {
	if len(os.Args) < 2 {
		fmt.Println("APEX Token Optimizer - Token 优化核心")
		fmt.Println("用法:")
		fmt.Println("  apex_token_optimizer correct -x <x> -y <y> -iw <img_w> -ih <img_h>")
		fmt.Println("  apex_token_optimizer screenshot -p <path> -w <width> -h <height> -t <tokens>")
		fmt.Println("  apex_token_optimizer effort -t <total> -w <waste> -wt <waste_type>")
		fmt.Println("  apex_token_optimizer purify")
		fmt.Println("  apex_token_optimizer stats")
		fmt.Println("  apex_token_optimizer traj -analyze")
		os.Exit(1)
	}

	cmd := os.Args[1]
	cfg := defaultConfig

	// 尝试加载配置
	configPath := os.Getenv("APEX_CONFIG")
	if configPath != "" {
		if data, err := os.ReadFile(configPath); err == nil {
			json.Unmarshal(data, &cfg)
		}
	}

	optimizer := NewOptimizer(cfg)

	switch cmd {
	case "correct":
		args := parseArgs(os.Args[2:])
		x := getFloatArg(args, "-x", 0)
		y := getFloatArg(args, "-y", 0)
		iw := getFloatArg(args, "-iw", 1920)
		ih := getFloatArg(args, "-ih", 1080)

		cx, cy := optimizer.OptimizeClick(x, y, iw, ih)
		fmt.Printf("校正坐标: (%.2f, %.2f) -> (%.2f, %.2f)\n", x, y, cx, cy)

	case "screenshot":
		args := parseArgs(os.Args[2:])
		path := getArg(args, "-p", "")
		w := getFloatArg(args, "-w", 1920)
		h := getFloatArg(args, "-h", 1080)
		tokens := getIntArg(args, "-t", 150)

		optimizer.ProcessScreenshot(path, w, h, tokens)
		stats := optimizer.GetStats()
		fmt.Printf("截图处理完成，Token统计: %+v\n", stats.Token)

	case "effort":
		args := parseArgs(os.Args[2:])
		total := getFloatArg(args, "-t", 0)
		waste := getFloatArg(args, "-w", 0)
		wasteType := getArg(args, "-wt", "unknown")

		optimizer.TrackEffort(total, waste, wasteType)
		effortStats := optimizer.effortTracker.GetStats()
		fmt.Printf("算力统计: 有效=%.2f, 总计=%.2f, 效率=%.2f%%\n", 
			effortStats.ValidEffort, effortStats.TotalEffort, effortStats.Efficiency*100)

	case "purify":
		stats := optimizer.PurifyIfNeeded()
		fmt.Printf("净化完成: 清理截图%d张, 释放Token%d, 清理缓存%.2fMB, 耗时%dms\n",
			stats.PurgedScreenshots, stats.PurgedTokens, stats.CacheCleanedMB, stats.DurationMs)

	case "stats":
		stats := optimizer.GetStats()
		data, _ := json.MarshalIndent(stats, "", "  ")
		fmt.Println(string(data))

	case "traj":
		if len(os.Args) > 2 && os.Args[2] == "-analyze" {
			avgErr, maxErr := optimizer.trajLog.GetCoordStats()
			overflows := optimizer.trajLog.GetContextOverflows()
			fmt.Printf("轨迹分析: 平均误差=%.4f, 最大误差=%.4f, 溢出风险=%d\n", avgErr, maxErr, overflows)
		}

	default:
		fmt.Fprintf(os.Stderr, "未知命令: %s\n", cmd)
		os.Exit(1)
	}
}

// ============ 工具函数 ============

type argsMap map[string]string

func parseArgs(raw []string) argsMap {
	m := make(argsMap)
	for i := 0; i < len(raw)-1; i++ {
		if strings.HasPrefix(raw[i], "-") {
			m[raw[i]] = raw[i+1]
			i++
		}
	}
	return m
}

func getArg(m argsMap, key, def string) string {
	if v, ok := m[key]; ok {
		return v
	}
	return def
}

func getFloatArg(m argsMap, key string, def float64) float64 {
	if v, ok := m[key]; ok {
		var f float64
		fmt.Sscanf(v, "%f", &f)
		return f
	}
	return def
}

func getIntArg(m argsMap, key string, def int) int {
	if v, ok := m[key]; ok {
		var i int
		fmt.Sscanf(v, "%d", &i)
		return i
	}
	return def
}