// ppt_generator.go — PPT生成服务
// 简化版：GPT生成内容 + Python生成PPTX

package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

const (
	PPT_PORT   = ":8103"
	GPT55_API  = "https://api.freemodel.dev/v1/chat/completions"
	GPT55_KEY  = "Bearer fe_oa_2ef1df35ba1d091f99212ba121aeb5b4fd35edf8baaba7a9"
	PYTHON_BIN = "/usr/bin/python3"
)

type Request struct {
	Topic  string `json:"topic"`
	Slides int    `json:"slides"`
	Style  string `json:"style"`
}

type Response struct {
	Status   string `json:"status"`
	PPTXPath string `json:"pptx_path"`
	Slides   int    `json:"slides"`
}

func main() {
	http.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
	})
	http.HandleFunc("/generate", generateHandler)

	fmt.Println("PPT Generator :8103")
	fmt.Println("  POST /generate {topic, slides, style}")

	http.ListenAndServe(PPT_PORT, nil)
}

func generateHandler(w http.ResponseWriter, r *http.Request) {
	var req Request
	json.NewDecoder(r.Body).Decode(&req)

	if req.Slides == 0 {
		req.Slides = 5
	}
	if req.Style == "" {
		req.Style = "business"
	}

	// 生成PPT大纲
	outline := generateOutline(req.Topic, req.Slides, req.Style)

	// 用Python生成PPTX
	pptxPath, err := createPPTX(req.Topic, outline)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}

	resp := Response{
		Status:   "success",
		PPTXPath: pptxPath,
		Slides:   len(outline),
	}
	json.NewEncoder(w).Encode(resp)
}

type SlideOutline struct {
	Title   string   `json:"title"`
	Content string   `json:"content"`
	Bullets []string `json:"bullets"`
}

func generateOutline(topic string, slides int, style string) []SlideOutline {
	// 调用GPT生成大纲
	prompt := fmt.Sprintf(`为主题"%s"生成%d页PPT大纲。

JSON格式返回，slides是数组，每项包含title和bullets数组：
{"slides":[{"title":"标题","bullets":["要点1","要点2","要点3"]}]}

只返回JSON。`, topic, slides)

	reqBody := map[string]interface{}{
		"model": "gpt-5.5",
		"messages": []map[string]string{
			{"role": "user", "content": prompt},
		},
		"max_tokens": 1500,
	}

	data, _ := json.Marshal(reqBody)
	req, _ := http.NewRequest("POST", GPT55_API, bytes.NewBuffer(data))
	req.Header.Set("Authorization", GPT55_KEY)
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 25 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return fallbackOutline(topic, slides)
	}
	defer resp.Body.Close()

	body, _ := ioutil.ReadAll(resp.Body)

	var result map[string]interface{}
	json.Unmarshal(body, &result)

	if choices, ok := result["choices"].([]interface{}); ok && len(choices) > 0 {
		if msg, ok := choices[0].(map[string]interface{}); ok {
			if content, ok := msg["message"].(map[string]interface{}); ok {
				if text, ok := content["content"].(string); ok {
					// 解析JSON
					text = strings.TrimSpace(text)
					// 去掉可能的markdown code block
					text = strings.TrimPrefix(text, "```json")
					text = strings.TrimPrefix(text, "```")
					text = strings.TrimSuffix(text, "```")

					var data struct {
						Slides []SlideOutline `json:"slides"`
					}
					if json.Unmarshal([]byte(text), &data) == nil && len(data.Slides) > 0 {
						return data.Slides
					}
				}
			}
		}
	}

	return fallbackOutline(topic, slides)
}

func createPPTX(topic string, outline []SlideOutline) (string, error) {
	// 写入临时JSON数据文件
	dataPath := filepath.Join(os.TempDir(), "ppt_data.json")
	dataJSON, _ := json.Marshal(outline)
	ioutil.WriteFile(dataPath, dataJSON, 0644)

	// 输出文件
	safeName := strings.ReplaceAll(topic, " ", "_")
	safeName = strings.ReplaceAll(safeName, "/", "_")
	if len(safeName) > 15 {
		safeName = safeName[:15]
	}
	outputPath := filepath.Join("/tmp", fmt.Sprintf("%s_%d.pptx", safeName, time.Now().Unix()))

	// Python脚本
	script := fmt.Sprintf(`
import json
from pptx import Presentation
from pptx.util import Inches, Pt

with open('%s') as f:
    outline = json.load(f)

prs = Presentation()
prs.slide_width = Inches(13.333)
prs.slide_height = Inches(7.5)

for item in outline:
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    
    # 标题
    title_box = slide.shapes.add_textbox(Inches(0.5), Inches(0.3), Inches(12), Inches(1))
    tf = title_box.text_frame
    tf.text = item.get('title', '')
    tf.paragraphs[0].font.size = Pt(40)
    tf.paragraphs[0].font.bold = True
    
    # 要点
    content_box = slide.shapes.add_textbox(Inches(0.5), Inches(1.5), Inches(12), Inches(5.5))
    cf = content_box.text_frame
    cf.text = item.get('content', '')
    cf.paragraphs[0].font.size = Pt(24)
    
    for bullet in item.get('bullets', []):
        p = cf.add_paragraph()
        p.text = '• ' + bullet
        p.level = 1
        p.font.size = Pt(20)

prs.save('%s')
print('%s')
`, dataPath, outputPath, outputPath)

	// 写入脚本
	scriptPath := filepath.Join(os.TempDir(), "gen_ppt.py")
	ioutil.WriteFile(scriptPath, []byte(script), 0644)

	// 执行
	cmd := exec.Command(PYTHON_BIN, scriptPath)
	var stderr bytes.Buffer
	cmd.Stdout = &bytes.Buffer{}
	cmd.Stderr = &stderr
	err := cmd.Run()
	if err != nil {
		return "", fmt.Errorf("生成失败: %s", stderr.String())
	}

	return outputPath, nil
}

func fallbackOutline(topic string, slides int) []SlideOutline {
	outline := make([]SlideOutline, slides)
	for i := 0; i < slides; i++ {
		outline[i] = SlideOutline{
			Title:   fmt.Sprintf("%s - 第%d页", topic, i+1),
			Content: "核心内容",
			Bullets: []string{"要点1", "要点2", "要点3"},
		}
	}
	return outline
}
