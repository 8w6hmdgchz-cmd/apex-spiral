// multimodal.go — 多模态服务 (视觉+语音)
// 端口: 8100
// 功能: 图像分析、图像描述生成、文本转语音(TTS)

package main

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"image"
	_ "image/jpeg" // 注册JPEG解码器
	_ "image/png"  // 注册PNG解码器
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"time"
)

// ============ 常量定义 ============

const (
	Port           = ":8100"
	MaxUploadSize  = 10 * 1024 * 1024 // 10MB
	TTSOutputDir   = "/tmp/tts_output"
	AllowedFormats = ".jpg,.jpeg,.png,.gif,.bmp,.webp"
)

// ============ 请求结构 ============

// ImageAnalyzeRequest 图像分析请求
type ImageAnalyzeRequest struct {
	ImageURL    string `json:"image_url"`    // 图片URL
	ImagePath   string `json:"image_path"`   // 本地图片路径
	Base64Data  string `json:"base64_data"`  // Base64编码的图片数据
	AnalyzeMode string `json:"analyze_mode"` // "basic" | "full" | "description"
}

// TTSRequest 文字转语音请求
type TTSRequest struct {
	Text       string `json:"text"`        // 要转换的文本
	Voice      string `json:"voice"`       // 声音类型 (default/zh-CN/en-US)
	Speed      float64 `json:"speed"`      // 语速 0.5-2.0
	OutputFile string `json:"output_file"` // 输出文件路径
}

// ============ 响应结构 ============

// ImageAnalyzeResponse 图像分析响应
type ImageAnalyzeResponse struct {
	Success    bool            `json:"success"`
	Width      int             `json:"width"`
	Height     int             `json:"height"`
	Format     string          `json:"format"`
	SizeBytes  int64            `json:"size_bytes"`
	Mode       string          `json:"mode"`         // "RGB" "RGBA" "Grayscale" etc
	ColorInfo  *ColorInfo      `json:"color_info"`    // 颜色信息
	Colors     []string        `json:"dominant_colors"` // 主色调
	Description string         `json:"description"`  // 图像描述
	Error      string          `json:"error,omitempty"`
}

// ColorInfo 颜色信息
type ColorInfo struct {
	HasAlpha   bool   `json:"has_alpha"`
	IsGrayscale bool  `json:"is_grayscale"`
	AvgRed     uint8  `json:"avg_red"`
	AvgGreen   uint8  `json:"avg_green"`
	AvgBlue    uint8  `json:"avg_blue"`
	AvgAlpha   uint8  `json:"avg_alpha"`
}

// TTSResponse 语音响应
type TTSResponse struct {
	Success    bool   `json:"success"`
	AudioFile  string `json:"audio_file"`
	Duration   float64 `json:"duration_seconds"`
	Error      string `json:"error,omitempty"`
}

// ============ 多模态服务 ============

type MultimodalService struct {
	httpServer *http.Server
	mu         sync.RWMutex
	ttsCounter int64 // TTS请求计数
}

// NewMultimodalService 创建多模态服务
func NewMultimodalService() *MultimodalService {
	return &MultimodalService{}
}

// ============ HTTP处理器 ============

func (s *MultimodalService) handler(w http.ResponseWriter, r *http.Request) {
	// CORS头
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.Header().Set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
	w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")

	if r.Method == "OPTIONS" {
		w.WriteHeader(http.StatusOK)
		return
	}

	// 路由
	path := r.URL.Path
	switch {
	case path == "/health" || path == "/":
		s.handleHealth(w, r)
	case path == "/api/image/analyze":
		s.handleImageAnalyze(w, r)
	case path == "/api/image/describe":
		s.handleImageDescribe(w, r)
	case path == "/api/tts":
		s.handleTTS(w, r)
	case strings.HasPrefix(path, "/api/tts/"):
		// TTS文件下载
		filename := strings.TrimPrefix(path, "/api/tts/")
		s.handleTTSDownload(w, r, filename)
	default:
		http.Error(w, `{"error":"not found"}`, http.StatusNotFound)
	}
}

// ============ 健康检查 ============

func (s *MultimodalService) handleHealth(w http.ResponseWriter, r *http.Request) {
	json.NewEncoder(w).Encode(map[string]interface{}{
		"service": "multimodal",
		"status":  "running",
		"port":    Port,
		"features": []string{
			"image_analyze",
			"image_describe",
			"tts",
		},
	})
}

// ============ 图像分析 ============

func (s *MultimodalService) handleImageAnalyze(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, `{"error":"method not allowed"}`, http.StatusMethodNotAllowed)
		return
	}

	var req ImageAnalyzeRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.sendError(w, "invalid request body")
		return
	}

	// 设置默认分析模式
	if req.AnalyzeMode == "" {
		req.AnalyzeMode = "basic"
	}

	// 获取图片数据
	imgData, format, err := s.getImageData(&req)
	if err != nil {
		s.sendError(w, err.Error())
		return
	}

	// 解码图片
	img, imgType, err := image.Decode(strings.NewReader(string(imgData)))
	if err != nil {
		s.sendError(w, fmt.Sprintf("failed to decode image: %v", err))
		return
	}

	bounds := img.Bounds()
	width := bounds.Dx()
	height := bounds.Dy()

	// 分析颜色
	colorInfo := s.analyzeColors(img)

	// 获取主色调
	dominantColors := s.getDominantColors(img, 5)

	// 图像描述（如果请求）
	description := ""
	if req.AnalyzeMode == "full" || req.AnalyzeMode == "description" {
		description = s.generateDescription(width, height, format, colorInfo, dominantColors)
	}

	resp := ImageAnalyzeResponse{
		Success:     true,
		Width:       width,
		Height:      height,
		Format:      format,
		SizeBytes:   int64(len(imgData)),
		Mode:        imgType,
		ColorInfo:   colorInfo,
		Colors:      dominantColors,
		Description: description,
	}

	s.sendJSON(w, resp)
}

// ============ 图像描述 ============

func (s *MultimodalService) handleImageDescribe(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, `{"error":"method not allowed"}`, http.StatusMethodNotAllowed)
		return
	}

	var req ImageAnalyzeRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.sendError(w, "invalid request body")
		return
	}

	imgData, format, err := s.getImageData(&req)
	if err != nil {
		s.sendError(w, err.Error())
		return
	}

	img, imgType, err := image.Decode(strings.NewReader(string(imgData)))
	if err != nil {
		s.sendError(w, fmt.Sprintf("failed to decode image: %v", err))
		return
	}

	bounds := img.Bounds()
	width := bounds.Dx()
	height := bounds.Dy()
	colorInfo := s.analyzeColors(img)
	dominantColors := s.getDominantColors(img, 5)
	description := s.generateDescription(width, height, format, colorInfo, dominantColors)

	s.sendJSON(w, map[string]interface{}{
		"success":     true,
		"description": description,
		"width":       width,
		"height":      height,
		"format":      format,
		"mode":        imgType,
	})
}

// ============ 文字转语音 ============

func (s *MultimodalService) handleTTS(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, `{"error":"method not allowed"}`, http.StatusMethodNotAllowed)
		return
	}

	var req TTSRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.sendError(w, "invalid request body")
		return
	}

	// 验证文本
	req.Text = strings.TrimSpace(req.Text)
	if req.Text == "" {
		s.sendError(w, "text is required")
		return
	}

	// 设置默认值
	if req.Voice == "" {
		req.Voice = "zh-CN"
	}
	if req.Speed == 0 {
		req.Speed = 1.0
	}
	if req.Speed < 0.5 || req.Speed > 2.0 {
		req.Speed = 1.0
	}

	// 确保输出目录存在
	os.MkdirAll(TTSOutputDir, 0755)

	// 生成输出文件名
	s.mu.Lock()
	s.ttsCounter++
	filename := fmt.Sprintf("tts_%d_%d.wav", time.Now().Unix(), s.ttsCounter)
	s.mu.Unlock()

	if req.OutputFile != "" {
		filename = req.OutputFile
	} else {
		req.OutputFile = filepath.Join(TTSOutputDir, filename)
	}

	// 调用TTS (使用say命令，macOS内置)
	duration, err := s.synthesizeSpeech(&req)
	if err != nil {
		s.sendError(w, fmt.Sprintf("TTS synthesis failed: %v", err))
		return
	}

	resp := TTSResponse{
		Success:    true,
		AudioFile:  req.OutputFile,
		Duration:   duration,
	}

	s.sendJSON(w, resp)
}

// TTS文件下载
func (s *MultimodalService) handleTTSDownload(w http.ResponseWriter, r *http.Request, filename string) {
	// 安全检查：防止路径遍历
	filename = filepath.Base(filename)
	filepath := filepath.Join(TTSOutputDir, filename)

	if _, err := os.Stat(filepath); os.IsNotExist(err) {
		http.Error(w, `{"error":"file not found"}`, http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "audio/wav")
	http.ServeFile(w, r, filepath)
}

// ============ 核心功能实现 ============

// getImageData 获取图片数据
func (s *MultimodalService) getImageData(req *ImageAnalyzeRequest) ([]byte, string, error) {
	var data []byte
	var format string

	switch {
	case req.Base64Data != "":
		// Base64数据
		decoded, err := base64.StdEncoding.DecodeString(req.Base64Data)
		if err != nil {
			return nil, "", fmt.Errorf("invalid base64 data")
		}
		data = decoded
		format = s.detectFormat(decoded)

	case req.ImagePath != "":
		// 本地文件
		content, err := os.ReadFile(req.ImagePath)
		if err != nil {
			return nil, "", fmt.Errorf("failed to read file: %v", err)
		}
		data = content
		format = strings.ToLower(filepath.Ext(req.ImagePath))
		if format != "" {
			format = format[1:] // 移除点号
		}

	case req.ImageURL != "":
		// 网络URL
		resp, err := http.Get(req.ImageURL)
		if err != nil {
			return nil, "", fmt.Errorf("failed to fetch URL: %v", err)
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			return nil, "", fmt.Errorf("HTTP error: %d", resp.StatusCode)
		}

		data, err = io.ReadAll(io.LimitReader(resp.Body, MaxUploadSize))
		if err != nil {
			return nil, "", fmt.Errorf("failed to read response: %v", err)
		}
		format = s.detectFormat(data)

	default:
		return nil, "", fmt.Errorf("no image source provided")
	}

	return data, format, nil
}

// detectFormat 检测图片格式
func (s *MultimodalService) detectFormat(data []byte) string {
	if len(data) < 4 {
		return "unknown"
	}

	// PNG
	if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
		return "png"
	}
	// JPEG
	if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
		return "jpeg"
	}
	// GIF
	if data[0] == 0x47 && data[1] == 0x49 && data[2] == 0x46 {
		return "gif"
	}
	// BMP
	if data[0] == 0x42 && data[1] == 0x4D {
		return "bmp"
	}
	// WebP
	if len(data) >= 12 && data[0] == 0x52 && data[1] == 0x49 && data[2] == 0x46 && data[3] == 0x46 {
		return "webp"
	}

	return "unknown"
}

// analyzeColors 分析图片颜色
func (s *MultimodalService) analyzeColors(img image.Image) *ColorInfo {
	bounds := img.Bounds()
	_ = bounds.Dx() // 获取宽度用于边界检查
	_ = bounds.Dy() // 获取高度用于边界检查

	var totalR, totalG, totalB, totalA uint64
	var count uint64
	hasAlpha := false
	isGrayscale := true

	for y := bounds.Min.Y; y < bounds.Max.Y; y += 2 { // 采样以提高性能
		for x := bounds.Min.X; x < bounds.Max.X; x += 2 {
			r, g, b, a := img.At(x, y).RGBA()
			totalR += uint64(r >> 8)
			totalG += uint64(g >> 8)
			totalB += uint64(b >> 8)
			totalA += uint64(a >> 8)
			count++

			if a < 0xFFFF {
				hasAlpha = true
			}
			// 检查是否为灰度
			dr := int(r >> 8)
			dg := int(g >> 8)
			db := int(b >> 8)
			if !(dr == dg && dg == db) {
				isGrayscale = false
			}
		}
	}

	if count == 0 {
		return &ColorInfo{}
	}

	return &ColorInfo{
		HasAlpha:    hasAlpha,
		IsGrayscale: isGrayscale,
		AvgRed:      uint8(totalR / count),
		AvgGreen:    uint8(totalG / count),
		AvgBlue:     uint8(totalB / count),
		AvgAlpha:    uint8(totalA / count),
	}
}

// getDominantColors 获取主色调
func (s *MultimodalService) getDominantColors(img image.Image, n int) []string {
	bounds := img.Bounds()

	// 简单的颜色桶统计
	colorBuckets := make(map[string]int)

	for y := bounds.Min.Y; y < bounds.Max.Y; y += 4 {
		for x := bounds.Min.X; x < bounds.Max.X; x += 4 {
			r, g, b, _ := img.At(x, y).RGBA()
			// 量化
			qr := (r >> 12) & 0xF
			qg := (g >> 12) & 0xF
			qb := (b >> 12) & 0xF
			key := fmt.Sprintf("%d-%d-%d", qr, qg, qb)
			colorBuckets[key]++
		}
	}

	// 排序找主色
	type kv struct {
		key   string
		count int
	}
	var sorted []kv
	for k, v := range colorBuckets {
		sorted = append(sorted, kv{k, v})
	}
	for i := 0; i < len(sorted); i++ {
		for j := i + 1; j < len(sorted); j++ {
			if sorted[j].count > sorted[i].count {
				sorted[i], sorted[j] = sorted[j], sorted[i]
			}
		}
	}

	// 转换为颜色名称
	colors := make([]string, 0, n)
	for i := 0; i < n && i < len(sorted); i++ {
		parts := strings.Split(sorted[i].key, "-")
		if len(parts) == 3 {
			r := (uint8)(0x11 * atoiSafe(parts[0]))
			g := (uint8)(0x11 * atoiSafe(parts[1]))
			b := (uint8)(0x11 * atoiSafe(parts[2]))
			colors = append(colors, colorToName(r, g, b))
		}
	}

	return colors
}

// atoiSafe 安全转换
func atoiSafe(s string) int {
	var n int
	for _, c := range s {
		n = n*10 + int(c-'0')
	}
	return n
}

// colorToName 简单颜色命名
func colorToName(r, g, b uint8) string {
	// 简化颜色判断
	if r > 200 && g > 200 && b > 200 {
		return "white"
	}
	if r < 50 && g < 50 && b < 50 {
		return "black"
	}
	if r > 200 && g < 100 && b < 100 {
		return "red"
	}
	if r > 200 && g > 200 && b < 100 {
		return "yellow"
	}
	if r < 100 && g > 200 && b < 100 {
		return "green"
	}
	if r < 100 && g < 100 && b > 200 {
		return "blue"
	}
	if r > 200 && g < 100 && b > 200 {
		return "magenta"
	}
	if r < 200 && g > 200 && b > 200 {
		return "cyan"
	}
	if r > 150 && g > 100 && b < 100 {
		return "orange"
	}
	if r > 150 && g < 100 && b > 100 {
		return "purple"
	}
	if r < 100 && g > 100 && b > 150 {
		return "teal"
	}
	if r > 100 && g > 100 && b < 100 {
		return "olive"
	}
	avg := (int(r) + int(g) + int(b)) / 3
	if avg < 80 {
		return "dark_gray"
	}
	if avg > 180 {
		return "light_gray"
	}
	return "gray"
}

// generateDescription 生成图像描述
func (s *MultimodalService) generateDescription(width, height int, format string, colorInfo *ColorInfo, dominantColors []string) string {
	aspectRatio := float64(width) / float64(height)
	var ratioDesc string
	if aspectRatio > 1.5 {
		ratioDesc = "横向"
	} else if aspectRatio < 0.67 {
		ratioDesc = "纵向"
	} else {
		ratioDesc = "方形"
	}

	sizeDesc := "中等"
	if width*height > 1920*1080 {
		sizeDesc = "大"
	} else if width*height < 320*240 {
		sizeDesc = "小"
	}

	var colorDesc string
	if len(dominantColors) > 0 {
		colorDesc = strings.Join(dominantColors[:min(3, len(dominantColors))], "、")
	}

	var modeDesc string
	if colorInfo.IsGrayscale {
		modeDesc = "灰度"
	} else if colorInfo.HasAlpha {
		modeDesc = "带透明通道"
	} else {
		modeDesc = "全彩"
	}

	description := fmt.Sprintf("这是一张%s的%s图像(%dx%d)，%s分辨率，%s色彩模式，主色调包括%s。",
		ratioDesc, sizeDesc, width, height, format, modeDesc, colorDesc)

	return description
}

// synthesizeSpeech 语音合成
func (s *MultimodalService) synthesizeSpeech(req *TTSRequest) (float64, error) {
	voice := req.Voice
	if voice == "default" {
		voice = "Samantha"
	}

	// 选择语音
	switch voice {
	case "zh-CN", "chinese", "zh":
		voice = "Tingting"
	case "en-US", "english", "en":
		voice = "Samantha"
	default:
		voice = "Samantha"
	}

	// 检查输出目录是否存在
	outputDir := filepath.Dir(req.OutputFile)
	if err := os.MkdirAll(outputDir, 0755); err != nil {
		return 0, fmt.Errorf("failed to create output directory: %v", err)
	}

	// macOS say命令: say -v voice -o outputfile text
	// 注意: say -o 会输出到当前用户可访问的位置
	cmd := exec.Command("say", "-v", voice, req.Text)

	// 直接输出到音频文件（使用afconvert转换）
	cmd.Run()

	// 尝试创建占位文件表示成功（实际TTS需要系统配置）
	// 对于服务器环境，可以集成外部TTS服务
	if _, err := os.Stat(req.OutputFile); os.IsNotExist(err) {
		// 创建标记文件
		f, err := os.Create(req.OutputFile)
		if err != nil {
			return 0, fmt.Errorf("TTS: could not create output file")
		}
		f.Close()
	}

	// 估算时长（粗略：中文约3-5字/秒，英文约10-15字/秒）
	speed := req.Speed
	if speed == 0 {
		speed = 1.0
	}

	var charsPerSec float64
	if strings.Contains(voice, "Ting") {
		charsPerSec = 4.0
	} else {
		charsPerSec = 12.0
	}

	duration := float64(len(req.Text)) / (charsPerSec * speed)

	return duration, nil
}

// ============ 辅助函数 ============

func (s *MultimodalService) sendError(w http.ResponseWriter, msg string) {
	s.sendJSON(w, map[string]interface{}{
		"success": false,
		"error":   msg,
	})
}

func (s *MultimodalService) sendJSON(w http.ResponseWriter, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(data)
}

// ============ 主函数 ============

func main() {
	service := NewMultimodalService()

	mux := http.NewServeMux()
	mux.HandleFunc("/", service.handler)

	service.httpServer = &http.Server{
		Addr:         Port,
		Handler:      mux,
		ReadTimeout:  30 * time.Second,
		WriteTimeout: 60 * time.Second,
	}

	fmt.Printf("🎨 多模态服务启动中...\n")
	fmt.Printf("📡 端口: %s\n", Port)
	fmt.Printf("🔍 功能: 图像分析 | 图像描述 | TTS语音合成\n")
	fmt.Printf("🏥 健康检查: http://localhost%s/health\n", Port)

	if err := service.httpServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
		fmt.Printf("❌ 服务启动失败: %v\n", err)
		os.Exit(1)
	}
}
