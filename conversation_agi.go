// conversation_agi.go — 对话AGI服务
// 每次对话都经过APEX基因选择 + GPT-5.5回答 + 反思记忆

package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/http"
	"time"
)

const (
	APEX_URL   = "http://localhost:8092"
	GPT55_API  = "https://api.freemodel.dev/v1/chat/completions"
	GPT55_KEY  = "Bearer fe_oa_2ef1df35ba1d091f99212ba121aeb5b4fd35edf8baaba7a9"
)

type Request struct {
	Query string `json:"query"`
}

type Response struct {
	Answer   string  `json:"answer"`
	GeneUsed string  `json:"gene_used"`
	DeltaG   float64 `json:"delta_g"`
	Insights []string `json:"insights"`
}

func main() {
	http.HandleFunc("/chat", chatHandler)
	http.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
	})

	fmt.Println("[对话AGI] 端口 :8102")
	fmt.Println("  /chat   - 智能对话")
	fmt.Println("  /health - 健康检查")
	http.ListenAndServe(":8102", nil)
}

func chatHandler(w http.ResponseWriter, r *http.Request) {
	var req Request
	json.NewDecoder(r.Body).Decode(&req)

	insights := []string{}

	// Step 1: APEX基因选择
	gene, deltaG := selectGene(req.Query)
	insights = append(insights, fmt.Sprintf("APEX选择基因: %s (ΔG=%.3f)", gene, deltaG))

	// Step 2: GPT-5.5生成回答
	answer := generateWithGPT55(req.Query, gene, deltaG)
	insights = append(insights, "GPT-5.5生成回答")

	// Step 3: 存入记忆
	go storeMemory(req.Query, answer, gene, deltaG)

	resp := Response{
		Answer:   answer,
		GeneUsed: gene,
		DeltaG:   deltaG,
		Insights: insights,
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}

func selectGene(query string) (string, float64) {
	reqBody := map[string]interface{}{
		"query":    query,
		"use_bio":  true,
		"use_evm":  true,
	}

	data, _ := json.Marshal(reqBody)
	resp, err := http.Post(APEX_URL+"/api/v1/gene/select", "application/json", bytes.NewBuffer(data))
	if err != nil {
		return "default", 3.0
	}
	defer resp.Body.Close()

	var result map[string]interface{}
	json.NewDecoder(resp.Body).Decode(&result)

	gene := "default"
	var deltaG float64 = 3.0

	if sg, ok := result["selected_gene"].(map[string]interface{}); ok {
		if n, ok := sg["name"].(string); ok {
			gene = n
		}
	}
	if dg, ok := result["delta_g"].(float64); ok {
		deltaG = dg
	}

	return gene, deltaG
}

func generateWithGPT55(query, gene string, deltaG float64) string {
	systemPrompt := fmt.Sprintf(`你是一个有帮助的AI助手。用户会问问题，你需要用自然语言回答。

当前上下文：
- 选择的基因策略: %s
- 决策质量ΔG: %.3f (越高表示越匹配)

请根据基因策略和ΔG，选择最合适的回答方式。
- ΔG高(>4.5)：可以更自信地给出确定答案
- ΔG中(3.5-4.5)：给出答案但保持开放讨论
- ΔG低(<3.5)：谨慎回答，建议用户多方验证

回答要求：
1. 直接回答问题，不要重复用户的问题
2. 如果使用了某个基因策略，可以简单提及
3. 保持回答简洁有力`, gene, deltaG)

	reqBody := map[string]interface{}{
		"model": "gpt-5.5",
		"messages": []map[string]string{
			{"role": "system", "content": systemPrompt},
			{"role": "user", "content": query},
		},
		"max_tokens": 1000,
	}

	data, _ := json.Marshal(reqBody)
	req, _ := http.NewRequest("POST", GPT55_API, bytes.NewBuffer(data))
	req.Header.Set("Authorization", GPT55_KEY)
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 25 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return fallbackAnswer(query, gene)
	}
	defer resp.Body.Close()

	body, _ := ioutil.ReadAll(resp.Body)

	var result map[string]interface{}
	json.Unmarshal(body, &result)

	if choices, ok := result["choices"].([]interface{}); ok && len(choices) > 0 {
		if msg, ok := choices[0].(map[string]interface{}); ok {
			if content, ok := msg["message"].(map[string]interface{}); ok {
				if text, ok := content["content"].(string); ok {
					return text
				}
			}
		}
	}

	return fallbackAnswer(query, gene)
}

func fallbackAnswer(query, gene string) string {
	return fmt.Sprintf("我需要思考一下这个问题。根据[%s]策略，我会从多个角度分析...", gene)
}

func storeMemory(query, answer, gene string, deltaG float64) {
	// 简单打印，实际可存入持久记忆
	fmt.Printf("[记忆] %s -> %s (ΔG=%.3f)\n", gene, query[:min(30, len(query))], deltaG)
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
