package main

import (
	"crypto/sha256"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"time"
)

// ========================================
// APEX Book-to-Skill
// ApexBookSkill = DoclingParse ⊗ SkillStruct ⊗ LazyLoad ⊗ MemLLM ⊗ ParallelAgent
// ========================================

// ---- 数据结构 ----

type Skill struct {
	Name         string        `json:"name"`
	SourceFile   string        `json:"source_file"`
	SourceType   string        `json:"source_type"` // docx, md, pdf, txt, json
	Chapters     []Chapter     `json:"chapters"`
	Terms        []Term        `json:"terms"`
	Paradigms    []Paradigm    `json:"paradigms"`
	Cheatsheet   []string      `json:"cheatsheet"`
	CodeExamples []CodeExample `json:"code_examples"`
	CompiledAt   string        `json:"compiled_at"`
	Hash         string        `json:"hash"`
}

type Chapter struct {
	ID      string `json:"id"`
	Title   string `json:"title"`
	Offset  int    `json:"offset"` // for lazy loading
	Length  int    `json:"length"`
	Loaded  bool   `json:"loaded"`
	Content string `json:"content,omitempty"`
}

type Term struct {
	Term       string `json:"term"`
	Definition string `json:"definition"`
	Chapter    string `json:"chapter"`
}

type Paradigm struct {
	Name        string `json:"name"`
	Description string `json:"description"`
	WhenToUse   string `json:"when_to_use"`
}

type CodeExample struct {
	Language string `json:"language"`
	Title    string `json:"title"`
	Code     string `json:"code"`
	Chapter  string `json:"chapter"`
}

// LazyLoad index
type SkillIndex struct {
	Name       string         `json:"name"`
	SourceFile string         `json:"source_file"`
	Chapters   []ChapterIndex `json:"chapters"`
	Terms      []string       `json:"terms"`
	Paradigms  []string       `json:"paradigms"`
	TotalSize  int            `json:"total_size"`
}

type ChapterIndex struct {
	ID     string `json:"id"`
	Title  string `json:"title"`
	Offset int    `json:"offset"`
	Length int    `json:"length"`
}

// MemLLM memory record
type MemoryRecord struct {
	Skill     string   `json:"skill"`
	Chapter   string   `json:"chapter"`
	Content   string   `json:"content"`
	Tags      []string `json:"tags"`
	Timestamp string   `json:"timestamp"`
	Source    string   `json:"source"`
}

func main() {
	compileFile := flag.String("compile", "", "编译文档为SKILL")
	skillName := flag.String("name", "", "SKILL名称")
	batchDir := flag.String("batch", "", "批量编译目录")
	querySkill := flag.String("query", "", "查询SKILL")
	queryChapter := flag.String("chapter", "", "查询章节")
	list := flag.Bool("list", false, "列出所有SKILL")
	search := flag.String("search", "", "搜索记忆")
	flag.Parse()

	switch {
	case *list:
		listSkills()
	case *search != "":
		searchMemory(*search)
	case *querySkill != "":
		querySkillFunc(*querySkill, *queryChapter)
	case *compileFile != "":
		compileDocument(*compileFile, *skillName)
	case *batchDir != "":
		batchCompile(*batchDir)
	default:
		flag.Usage()
	}
}

// ---- T1: DoclingParse ----

func parseDocument(path string) (string, string, error) {
	ext := strings.ToLower(filepath.Ext(path))

	switch ext {
	case ".md", ".markdown":
		content, err := os.ReadFile(path)
		return string(content), "md", err

	case ".txt":
		content, err := os.ReadFile(path)
		return string(content), "txt", err

	case ".json":
		content, err := os.ReadFile(path)
		return string(content), "json", err

	case ".docx":
		return parseDocxPython(path), "docx", nil

	case ".pdf":
		return parsePDF(path), "pdf", nil

	default:
		// Fallback: try raw read
		content, err := os.ReadFile(path)
		if err == nil {
			return string(content), ext, nil
		}
		return "", "", fmt.Errorf("unsupported format: %s", ext)
	}
}

func parseDocxPython(path string) string {
	script := fmt.Sprintf(`
import sys
try:
    from docx import Document
    doc = Document(%q)
    text = []
    for para in doc.paragraphs:
        if para.text.strip():
            text.append(para.text)
    # Extract tables
    for table in doc.tables:
        for row in table.rows:
            cells = [cell.text.strip() for cell in row.cells]
            text.append(" | ".join(cells))
    print("---DOCX_START---")
    print("\n".join(text))
    print("---DOCX_END---")
except Exception as e:
    print("ERROR: " + str(e), file=sys.stderr)
    sys.exit(1)
`, path)

	cmd := exec.Command("python3", "-c", script)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Sprintf("[docx parse error: %s]", err)
	}

	// Extract content between markers
	s := string(out)
	start := strings.Index(s, "---DOCX_START---")
	end := strings.Index(s, "---DOCX_END---")
	if start >= 0 && end > start {
		return strings.TrimSpace(s[start+15 : end])
	}
	return s
}

func parsePDF(path string) string {
	// Try pdftotext first, fallback to basic extraction
	cmd := exec.Command("pdftotext", path, "-")
	out, err := cmd.Output()
	if err == nil && len(out) > 50 {
		return string(out)
	}

	// No parser available
	return fmt.Sprintf("[PDF: %s — install pdftotext or upload .md/.docx version]", filepath.Base(path))
}

// ---- T2: SkillStruct ----

func compileToSkill(content string, sourceType string, name string) Skill {
	if name == "" {
		name = fmt.Sprintf("skill-%x", sha256.Sum256([]byte(content)))[:16]
	}

	// Auto-detect name from first heading
	lines := strings.Split(content, "\n")
	autoName := name
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if strings.HasPrefix(line, "# ") {
			autoName = strings.TrimPrefix(line, "# ")
			autoName = strings.TrimSpace(autoName)
			break
		}
		if strings.HasPrefix(line, "## ") && autoName == name {
			autoName = strings.TrimPrefix(line, "## ")
			autoName = strings.TrimSpace(autoName)
		}
	}

	skill := Skill{
		Name:       autoName,
		SourceType: sourceType,
		Chapters:   extractChapters(content),
		Terms:      extractTerms(content),
		Paradigms:  extractParadigms(content),
		Cheatsheet: extractCheatsheet(content),
		CompiledAt: time.Now().UTC().Format(time.RFC3339),
		Hash:       fmt.Sprintf("%x", sha256.Sum256([]byte(content)))[:12],
	}

	return skill
}

// Extract chapters from markdown headings
func extractChapters(content string) []Chapter {
	var chapters []Chapter
	lines := strings.Split(content, "\n")
	var currentChapter *Chapter
	var currentContent []string
	offset := 0

	flushChapter := func() {
		if currentChapter == nil {
			return
		}
		currentChapter.Content = strings.TrimRight(strings.Join(currentContent, "\n"), "\n")
		currentChapter.Length = len(currentChapter.Content)
		chapters = append(chapters, *currentChapter)
	}

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(trimmed, "## ") || strings.HasPrefix(trimmed, "### ") {
			flushChapter()
			title := strings.TrimPrefix(trimmed, "## ")
			title = strings.TrimPrefix(title, "### ")
			id := fmt.Sprintf("ch-%x", sha256.Sum256([]byte(title)))[:8]
			currentChapter = &Chapter{
				ID: id, Title: title, Offset: offset,
			}
			currentContent = []string{line}
		} else if currentChapter != nil {
			currentContent = append(currentContent, line)
		}
		offset += len(line) + 1
	}

	flushChapter()

	if len(chapters) == 0 && strings.TrimSpace(content) != "" {
		id := fmt.Sprintf("ch-%x", sha256.Sum256([]byte("Full Document")))[:8]
		chapters = append(chapters, Chapter{
			ID:      id,
			Title:   "Full Document",
			Offset:  0,
			Length:  len(content),
			Content: content,
		})
	}

	return chapters
}

// Simple term extraction (heuristic)
func extractTerms(content string) []Term {
	var terms []Term
	seen := make(map[string]bool)
	lines := strings.Split(content, "\n")

	for _, line := range lines {
		line = strings.TrimSpace(line)
		// Match "**term**: definition" or "- **term**: definition"
		if strings.Contains(line, "**:") {
			parts := strings.SplitN(line, "**:", 2)
			if len(parts) == 2 {
				term := strings.TrimLeft(parts[0], "- *")
				term = strings.TrimSpace(term)
				def := strings.TrimSpace(parts[1])
				if term != "" && !seen[term] && len(def) > 5 {
					seen[term] = true
					terms = append(terms, Term{
						Term:       term,
						Definition: def,
					})
				}
			}
		}
	}
	return terms
}

func extractParadigms(content string) []Paradigm {
	var paradigms []Paradigm
	lines := strings.Split(content, "\n")
	for idx, line := range lines {
		lower := strings.ToLower(line)
		if strings.Contains(lower, "范式") || strings.Contains(lower, "pattern") || strings.Contains(lower, "principle") {
			if idx+1 < len(lines) {
				paradigms = append(paradigms, Paradigm{
					Name:        strings.TrimSpace(strings.TrimLeft(line, "# ")),
					Description: strings.TrimSpace(lines[idx+1]),
				})
			}
		}
	}
	return paradigms
}

func extractCheatsheet(content string) []string {
	var cheatsheet []string
	lines := strings.Split(content, "\n")
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		// Extract code blocks and key formulas
		if strings.HasPrefix(trimmed, "```") || strings.Contains(trimmed, "→") || strings.Contains(trimmed, "=>") || strings.Contains(trimmed, "公式") {
			cheatsheet = append(cheatsheet, trimmed)
		}
	}
	return cheatsheet
}

// ---- T3: LazyLoad ----

func buildIndex(skill Skill) SkillIndex {
	idx := SkillIndex{
		Name:       skill.Name,
		SourceFile: skill.SourceFile,
		Chapters:   make([]ChapterIndex, len(skill.Chapters)),
	}

	for i, ch := range skill.Chapters {
		if i < len(skill.Chapters) {
			idx.Chapters[i] = ChapterIndex{
				ID:     ch.ID,
				Title:  ch.Title,
				Offset: ch.Offset,
				Length: ch.Length,
			}
		}
	}

	for _, t := range skill.Terms {
		idx.Terms = append(idx.Terms, t.Term)
	}
	for _, p := range skill.Paradigms {
		idx.Paradigms = append(idx.Paradigms, p.Name)
	}
	idx.TotalSize = len([]byte(fmt.Sprintf("%v", skill)))

	return idx
}

func lazyLoadChapter(skillDir, chapterID string) (string, error) {
	chPath := filepath.Join(skillDir, "chapters", chapterID+".md")
	content, err := os.ReadFile(chPath)
	if err != nil {
		return "", err
	}
	return string(content), nil
}

// ---- T4: MemLLM ----

func saveToMemory(skill Skill, outputDir string) error {
	// Create skill directory
	skillDir := filepath.Join(outputDir, skill.Name)
	os.MkdirAll(skillDir, 0755)
	os.MkdirAll(filepath.Join(skillDir, "chapters"), 0755)
	os.MkdirAll(filepath.Join(skillDir, "memory"), 0755)

	// Save index
	idx := buildIndex(skill)
	idxData, _ := json.MarshalIndent(idx, "", "  ")
	os.WriteFile(filepath.Join(skillDir, "index.json"), idxData, 0644)

	// Save chapters for lazy loading
	for _, ch := range skill.Chapters {
		if strings.TrimSpace(ch.Content) == "" {
			return fmt.Errorf("chapter %s (%s) has empty content; refusing to write broken lazy-load file", ch.ID, ch.Title)
		}
		chPath := filepath.Join(skillDir, "chapters", ch.ID+".md")
		if err := os.WriteFile(chPath, []byte(ch.Content), 0644); err != nil {
			return err
		}
	}

	// Save SKILL.md
	var skillMD strings.Builder
	skillMD.WriteString(fmt.Sprintf("# %s\n\n", skill.Name))
	skillMD.WriteString(fmt.Sprintf("> Source: %s | Type: %s | Compiled: %s\n\n", skill.SourceFile, skill.SourceType, skill.CompiledAt))
	skillMD.WriteString("## Chapters\n\n")
	for _, ch := range skill.Chapters {
		skillMD.WriteString(fmt.Sprintf("- [%s](#%s)\n", ch.Title, ch.ID))
	}
	skillMD.WriteString("\n## Key Terms\n\n")
	for _, t := range skill.Terms {
		skillMD.WriteString(fmt.Sprintf("- **%s**: %s\n", t.Term, t.Definition))
	}
	skillMD.WriteString("\n## Paradigms\n\n")
	for _, p := range skill.Paradigms {
		skillMD.WriteString(fmt.Sprintf("- **%s**: %s\n", p.Name, p.Description))
	}
	os.WriteFile(filepath.Join(skillDir, "SKILL.md"), []byte(skillMD.String()), 0644)

	// Memory records
	memDir := filepath.Join(skillDir, "memory")
	termsJSON, _ := json.MarshalIndent(skill.Terms, "", "  ")
	os.WriteFile(filepath.Join(memDir, "terms.json"), termsJSON, 0644)
	paradigmsJSON, _ := json.MarshalIndent(skill.Paradigms, "", "  ")
	os.WriteFile(filepath.Join(memDir, "paradigms.json"), paradigmsJSON, 0644)

	fmt.Printf("  ✅ Saved to %s\n", skillDir)
	return nil
}

// ---- T5: ParallelAgent (simulated) ----

type AgentTask struct {
	Name   string
	Work   func() interface{}
	Result interface{}
}

func runParallelAgents(skill Skill) map[string]interface{} {
	results := make(map[string]interface{})
	var mu sync.Mutex
	var wg sync.WaitGroup

	agents := []AgentTask{
		{Name: "retrieval", Work: func() interface{} {
			return len(skill.Chapters)
		}},
		{Name: "paradigm-extract", Work: func() interface{} {
			return skill.Paradigms
		}},
		{Name: "code-extract", Work: func() interface{} {
			return skill.CodeExamples
		}},
		{Name: "compile-summary", Work: func() interface{} {
			return fmt.Sprintf("%s: %d chapters, %d terms, %d paradigms",
				skill.Name, len(skill.Chapters), len(skill.Terms), len(skill.Paradigms))
		}},
	}

	for _, agent := range agents {
		wg.Add(1)
		go func(a AgentTask) {
			defer wg.Done()
			result := a.Work()
			mu.Lock()
			results[a.Name] = result
			mu.Unlock()
		}(agent)
	}

	wg.Wait()
	return results
}

// ---- CLI Commands ----

func compileDocument(path, name string) {
	fmt.Printf("📚 APEX Book-to-Skill — 编译\n")
	fmt.Printf("━━━━━━━━━━━━━━━━━━━━━━━━━━\n")
	fmt.Printf("  文件: %s\n", path)

	// [T1] Parse
	fmt.Printf("\n[T1] DoclingParse...\n")
	content, sourceType, err := parseDocument(path)
	if err != nil {
		fmt.Fprintf(os.Stderr, "  ❌ Parse error: %s\n", err)
		os.Exit(1)
	}
	fmt.Printf("  ✅ 解析完成 (%s, %d 字符)\n", sourceType, len(content))

	// [T2] Compile to Skill
	fmt.Printf("\n[T2] SkillStruct — 编译SKILL...\n")
	skill := compileToSkill(content, sourceType, name)
	skill.SourceFile = filepath.Base(path)
	fmt.Printf("  ✅ 书名: %s\n", skill.Name)
	fmt.Printf("  📖 章节: %d\n", len(skill.Chapters))
	fmt.Printf("  📝 术语: %d\n", len(skill.Terms))
	fmt.Printf("  🧠 范式: %d\n", len(skill.Paradigms))

	// [T3] Build LazyLoad index
	fmt.Printf("\n[T3] LazyLoad — 按需索引...\n")
	idx := buildIndex(skill)
	fmt.Printf("  ✅ 索引: %d 章节 (合计 %d 字节)\n", len(idx.Chapters), idx.TotalSize)
	fmt.Printf("  💡 章节标题:\n")
	for _, ch := range skill.Chapters {
		fmt.Printf("    • [%s] %s\n", ch.ID, ch.Title)
	}

	// [T5] Parallel agents
	fmt.Printf("\n[T5] ParallelAgent — 多Agent并行...\n")
	agentResults := runParallelAgents(skill)
	fmt.Printf("  ✅ 检索Agent: %v\n", agentResults["retrieval"])
	fmt.Printf("  ✅ 范式提取: %v 个\n", len(agentResults["paradigm-extract"].([]Paradigm)))
	fmt.Printf("  ✅ 编译摘要: %s\n", agentResults["compile-summary"])

	// [T4] Save to memory
	fmt.Printf("\n[T4] MemLLM — 记忆固化...\n")
	outputDir := filepath.Join(getWorkspace(), "skills", "compiled")
	os.MkdirAll(outputDir, 0755)
	if err := saveToMemory(skill, outputDir); err != nil {
		fmt.Fprintf(os.Stderr, "  ❌ Save error: %s\n", err)
		os.Exit(1)
	}

	fmt.Printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━\n")
	fmt.Printf("🏁 编译完成!\n\n")

	// Print lazy-load demo
	fmt.Printf("📋 LazyLoad 演示 — 只有查询的章节才加载:\n")
	fmt.Printf("   输入: apex-book-skill --query \"%s\" --chapter \"%s\"\n", skill.Name, skill.Chapters[0].ID)
}

func batchCompile(dir string) {
	fmt.Printf("📚 批量编译: %s\n\n", dir)
	entries, err := os.ReadDir(dir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "  ❌ %s\n", err)
		os.Exit(1)
	}

	var wg sync.WaitGroup
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		ext := strings.ToLower(filepath.Ext(entry.Name()))
		if ext == ".md" || ext == ".docx" || ext == ".txt" || ext == ".json" || ext == ".pdf" {
			wg.Add(1)
			go func(name string) {
				defer wg.Done()
				path := filepath.Join(dir, name)
				content, stype, err := parseDocument(path)
				if err != nil {
					fmt.Printf("  ❌ %s: %s\n", name, err)
					return
				}
				skill := compileToSkill(content, stype, strings.TrimSuffix(name, ext))
				skill.SourceFile = name
				outputDir := filepath.Join(getWorkspace(), "skills", "compiled")
				os.MkdirAll(outputDir, 0755)
				if err := saveToMemory(skill, outputDir); err != nil {
					fmt.Printf("  ❌ %s: save failed: %s\n", name, err)
				}
			}(entry.Name())
		}
	}
	wg.Wait()
	fmt.Printf("\n✅ 批量编译完成\n")
}

func listSkills() {
	compiledDir := filepath.Join(getWorkspace(), "skills", "compiled")
	entries, err := os.ReadDir(compiledDir)
	if err != nil {
		fmt.Println("📭 暂无已编译SKILL")
		return
	}

	fmt.Println("📚 已编译SKILL库:")
	fmt.Println("━━━━━━━━━━━━━━━━━")
	for _, entry := range entries {
		if entry.IsDir() {
			idxPath := filepath.Join(compiledDir, entry.Name(), "index.json")
			if data, err := os.ReadFile(idxPath); err == nil {
				var idx SkillIndex
				json.Unmarshal(data, &idx)
				fmt.Printf("\n  📖 %s\n", idx.Name)
				fmt.Printf("     来源: %s\n", idx.SourceFile)
				fmt.Printf("     章节: %d\n", len(idx.Chapters))
				fmt.Printf("     术语: %d\n", len(idx.Terms))
				fmt.Printf("     范式: %d\n", len(idx.Paradigms))
			} else {
				fmt.Printf("\n  📖 %s (索引加载中...)\n", entry.Name())
			}
		}
	}
}

func querySkillFunc(name, chapterID string) {
	skillDir := filepath.Join(getWorkspace(), "skills", "compiled", name)
	idxPath := filepath.Join(skillDir, "index.json")

	data, err := os.ReadFile(idxPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "❌ SKILL \"%s\" not found\n", name)
		os.Exit(1)
	}

	var idx SkillIndex
	json.Unmarshal(data, &idx)

	fmt.Printf("📖 %s\n", idx.Name)
	fmt.Printf("━━━━━━━━━━━━━━━━━\n")

	if chapterID != "" {
		// [T3] LazyLoad: only load this chapter
		fmt.Printf("\n[T3] LazyLoad — 按需加载章节...\n")
		content, err := lazyLoadChapter(skillDir, chapterID)
		if err != nil {
			fmt.Printf("  ❌ 章节 %s 未找到\n", chapterID)
		} else {
			fmt.Printf("  ✅ 加载完成 (%d 字符)\n\n", len(content))
			fmt.Println(content[:min(500, len(content))])
		}
	} else {
		fmt.Printf("  来源: %s\n", idx.SourceFile)
		fmt.Printf("  章节索引:\n")
		for _, ch := range idx.Chapters {
			fmt.Printf("    • [%s] %s\n", ch.ID, ch.Title)
		}
		fmt.Printf("\n  术语: %s\n", strings.Join(idx.Terms, ", "))
		fmt.Printf("  范式: %s\n", strings.Join(idx.Paradigms, ", "))
		fmt.Printf("\n💡 使用 --chapter <ID> 按需加载具体章节\n")
	}
}

func searchMemory(query string) {
	compiledDir := filepath.Join(getWorkspace(), "skills", "compiled")
	fmt.Printf("🔍 搜索记忆: \"%s\"\n\n", query)

	entries, _ := os.ReadDir(compiledDir)
	q := strings.ToLower(query)
	found := 0

	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}
		idxPath := filepath.Join(compiledDir, entry.Name(), "index.json")
		data, err := os.ReadFile(idxPath)
		if err != nil {
			continue
		}
		var idx SkillIndex
		json.Unmarshal(data, &idx)

		// Search index
		if strings.Contains(strings.ToLower(idx.Name), q) {
			fmt.Printf("  📖 %s (书名匹配)\n", idx.Name)
			found++
			continue
		}
		for _, t := range idx.Terms {
			if strings.Contains(strings.ToLower(t), q) {
				fmt.Printf("  📖 %s → 术语: %s\n", idx.Name, t)
				found++
				break
			}
		}
		for _, p := range idx.Paradigms {
			if strings.Contains(strings.ToLower(p), q) {
				fmt.Printf("  📖 %s → 范式: %s\n", idx.Name, p)
				found++
				break
			}
		}
		// Search chapter titles
		for _, ch := range idx.Chapters {
			if strings.Contains(strings.ToLower(ch.Title), q) {
				fmt.Printf("  📖 %s → 章节: %s [%s]\n", idx.Name, ch.Title, ch.ID)
				found++
				break
			}
		}
	}

	if found == 0 {
		fmt.Println("  ❌ 未找到匹配内容")
	}
}

func getWorkspace() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".openclaw", "workspace")
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
