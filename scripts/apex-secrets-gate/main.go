package main

import (
	"bufio"
	"encoding/json"
	"flag"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

type Finding struct {
	Path     string `json:"path"`
	Line     int    `json:"line"`
	Rule     string `json:"rule"`
	Severity string `json:"severity"`
	Evidence string `json:"evidence"`
}
type Report struct {
	OK           bool      `json:"ok"`
	Root         string    `json:"root"`
	Mode         string    `json:"mode"`
	Findings     []Finding `json:"findings"`
	ScannedFiles int       `json:"scanned_files"`
	SkippedFiles int       `json:"skipped_files"`
}
type Rule struct {
	Name     string
	Severity string
	Re       *regexp.Regexp
}

var rules = []Rule{
	{"openai_api_key", "critical", regexp.MustCompile(`sk-[A-Za-z0-9_-]{20,}`)},
	{"github_token", "critical", regexp.MustCompile(`gh[pousr]_[A-Za-z0-9_]{20,}`)},
	{"aws_access_key", "critical", regexp.MustCompile(`AKIA[0-9A-Z]{16}`)},
	{"private_key_block", "critical", regexp.MustCompile("-----BEGIN " + `(RSA |OPENSSH |EC |DSA |PRIVATE )?` + "PRIVATE KEY-----")},
	{"slack_token", "high", regexp.MustCompile(`xox[baprs]-[A-Za-z0-9-]{20,}`)},
}

func main() {
	root := flag.String("root", ".", "root/path to scan")
	mode := flag.String("mode", "path", "path|staged")
	jsonOut := flag.Bool("json", true, "emit json")
	flag.Parse()
	rep := Scan(*root, *mode)
	if *jsonOut {
		b, _ := json.MarshalIndent(rep, "", "  ")
		fmt.Println(string(b))
	} else {
		fmt.Printf("ok=%v findings=%d scanned=%d skipped=%d\n", rep.OK, len(rep.Findings), rep.ScannedFiles, rep.SkippedFiles)
	}
	if !rep.OK {
		os.Exit(1)
	}
}
func Scan(root, mode string) Report {
	abs, _ := filepath.Abs(root)
	rep := Report{OK: true, Root: abs, Mode: mode}
	if mode == "staged" {
		return scanStaged(abs, rep)
	}
	filepath.WalkDir(abs, func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			return nil
		}
		if p == abs {
			return nil
		}
		if shouldSkip(p, d) {
			if d != nil && d.IsDir() {
				return filepath.SkipDir
			}
			rep.SkippedFiles++
			return nil
		}
		if d == nil || d.IsDir() {
			return nil
		}
		fs, skip := scanFile(p)
		if skip {
			rep.SkippedFiles++
			return nil
		}
		rep.ScannedFiles++
		rep.Findings = append(rep.Findings, fs...)
		return nil
	})
	rep.OK = len(rep.Findings) == 0
	return rep
}
func scanStaged(root string, rep Report) Report {
	rep.OK = false
	rep.Findings = []Finding{{Path: "staged", Line: 0, Rule: "unsupported_mode", Severity: "medium", Evidence: "use --mode path in this runtime until git blob scanning is enabled"}}
	return rep
}
func shouldSkip(p string, d fs.DirEntry) bool {
	parts := strings.Split(filepath.ToSlash(p), "/")
	for _, x := range parts {
		if x == ".git" || x == "vendor" || x == "node_modules" || x == "target" || x == ".venv" {
			return true
		}
	}
	if d != nil && d.IsDir() {
		return false
	}
	ext := strings.ToLower(filepath.Ext(p))
	allowed := map[string]bool{".go": true, ".rs": true, ".py": true, ".sh": true, ".md": true, ".json": true, ".yaml": true, ".yml": true, ".toml": true, ".txt": true, ".env": true, ".ipynb": true}
	return !allowed[ext]
}
func scanFile(p string) ([]Finding, bool) {
	f, err := os.Open(p)
	if err != nil {
		return nil, true
	}
	defer f.Close()
	scanner := bufio.NewScanner(f)
	scanner.Buffer(make([]byte, 0, 64*1024), 1024*1024)
	out := []Finding{}
	line := 0
	for scanner.Scan() {
		line++
		text := scanner.Text()
		if strings.ContainsRune(text, '\x00') {
			return nil, true
		}
		for _, r := range rules {
			if r.Re.MatchString(text) {
				out = append(out, Finding{Path: p, Line: line, Rule: r.Name, Severity: r.Severity, Evidence: redact(text)})
			}
		}
	}
	return out, false
}
func redact(s string) string {
	s = strings.TrimSpace(s)
	if len(s) > 120 {
		s = s[:120]
	}
	for _, r := range rules {
		s = r.Re.ReplaceAllString(s, "[REDACTED]")
	}
	return s
}
