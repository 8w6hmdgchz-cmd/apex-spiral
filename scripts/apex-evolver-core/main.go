package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

type Finding struct {
	Path     string `json:"path"`
	Signal   string `json:"signal"`
	Severity string `json:"severity"`
}
type EvolverState struct {
	ID        string    `json:"id"`
	CreatedAt string    `json:"created_at"`
	Root      string    `json:"root"`
	Phase     string    `json:"phase"`
	Findings  []Finding `json:"findings"`
	PatchPlan []string  `json:"patch_plan"`
	Verify    []string  `json:"verify"`
	Next      string    `json:"next"`
	Evidence  []string  `json:"evidence"`
}

func main() {
	mode := flag.String("mode", "observe", "observe|diagnose|plan|verify|cycle")
	root := flag.String("root", "/Users/lihongxin/.openclaw/workspace", "workspace root")
	out := flag.String("out", "", "optional json output")
	flag.Parse()
	st := cycle(*mode, *root)
	emit(st, *out)
	if *mode == "verify" && len(st.Findings) > 0 {
		os.Exit(1)
	}
}
func cycle(mode, root string) EvolverState {
	root = cleanRoot(root)
	st := EvolverState{ID: "apex-evolver-" + hash(root+time.Now().String()), CreatedAt: time.Now().Format(time.RFC3339), Root: root, Phase: mode, Evidence: []string{"root=" + root}}
	st.Findings = observe(root)
	if mode == "observe" {
		st.Next = "diagnose"
		return st
	}
	st.PatchPlan = diagnose(st.Findings)
	if mode == "diagnose" || mode == "plan" {
		st.Next = "verify"
		return st
	}
	st.Verify = verify(root)
	st.Next = "archive"
	return st
}
func observe(root string) []Finding {
	var fs []Finding
	targets := []string{"scripts", "skills", "apex-ene", "memory/devour"}
	for _, t := range targets {
		filepath.WalkDir(filepath.Join(root, t), func(p string, d os.DirEntry, err error) error {
			if err != nil || d.IsDir() {
				return nil
			}
			if shouldSkipPath(p) {
				return nil
			}
			b, err := os.ReadFile(p)
			if err != nil || !isProbablyText(b) {
				return nil
			}
			s := string(b)
			rel, _ := filepath.Rel(root, p)
			if hasTodoMarker(s) {
				fs = append(fs, Finding{rel, "todo_marker", "medium"})
			}
			if hasPortableTimeoutRisk(s) {
				fs = append(fs, Finding{rel, "portable_timeout_risk", "medium"})
			}
			if hasVirtualDataMarker(s) {
				fs = append(fs, Finding{rel, "possible_virtual_data_marker", "low"})
			}
			return nil
		})
	}
	sort.Slice(fs, func(i, j int) bool { return fs[i].Path < fs[j].Path })
	if len(fs) > 25 {
		return fs[:25]
	}
	return fs
}

func shouldSkipPath(p string) bool {
	if strings.Contains(p, "/target/") || strings.Contains(p, "/.git/") || strings.Contains(p, "/vendor/") || strings.Contains(p, "/third_party/") {
		return true
	}
	ext := filepath.Ext(p)
	if ext == "" {
		info, err := os.Stat(p)
		return err == nil && info.Mode()&0111 != 0
	}
	allowed := map[string]bool{".go": true, ".rs": true, ".sh": true, ".md": true, ".json": true, ".yaml": true, ".yml": true, ".toml": true, ".py": true}
	return !allowed[ext]
}

func isProbablyText(b []byte) bool {
	if len(b) == 0 {
		return true
	}
	limit := len(b)
	if limit > 4096 {
		limit = 4096
	}
	for _, c := range b[:limit] {
		if c == 0 {
			return false
		}
	}
	return true
}

func hasTodoMarker(s string) bool {
	for _, line := range strings.Split(s, "\n") {
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(trimmed, "TODO:") || strings.HasPrefix(trimmed, "FIXME:") || strings.Contains(trimmed, " TODO:") || strings.Contains(trimmed, " FIXME:") {
			return true
		}
	}
	return false
}

func hasPortableTimeoutRisk(s string) bool {
	return strings.Contains(s, "timeout ") && strings.Contains(s, "bash") && !strings.Contains(s, "perl -e 'alarm")
}

func hasVirtualDataMarker(s string) bool {
	lower := strings.ToLower(s)
	return strings.Contains(lower, "fake benchmark") || strings.Contains(lower, "mock result") || strings.Contains(lower, "virtual data")
}

func diagnose(fs []Finding) []string {
	if len(fs) == 0 {
		return []string{"no_patch_needed"}
	}
	var ps []string
	for _, f := range fs {
		switch f.Signal {
		case "todo_marker":
			ps = append(ps, "resolve_or_document_todo:"+f.Path)
		case "portable_timeout_risk":
			ps = append(ps, "replace_timeout_with_perl_alarm:"+f.Path)
		case "possible_virtual_data_marker":
			ps = append(ps, "audit_no_fake_data_claim:"+f.Path)
		}
	}
	return compact(ps)
}
func verify(root string) []string {
	checks := []string{}
	if _, err := os.Stat(filepath.Join(root, "scripts/apex-harness-bridge/apex-harness-bridge")); err == nil {
		checks = append(checks, "harness_bridge_present")
	}
	if out, err := exec.Command("git", "-C", root, "status", "--short").Output(); err == nil {
		checks = append(checks, "git_status_lines="+fmt.Sprint(len(strings.Split(strings.TrimSpace(string(out)), "\n"))))
	}
	return checks
}
func cleanRoot(r string) string {
	abs, err := filepath.Abs(r)
	if err != nil {
		panic(err)
	}
	if !strings.HasPrefix(abs, "/Users/lihongxin/.openclaw/workspace") {
		panic("root outside workspace")
	}
	return abs
}
func compact(xs []string) []string {
	m := map[string]bool{}
	var out []string
	for _, x := range xs {
		if !m[x] {
			m[x] = true
			out = append(out, x)
		}
	}
	return out
}
func emit(v any, out string) {
	b, _ := json.MarshalIndent(v, "", "  ")
	if out != "" {
		os.WriteFile(out, append(b, '\n'), 0644)
	}
	fmt.Println(string(b))
}
func hash(s string) string { h := sha256.Sum256([]byte(s)); return hex.EncodeToString(h[:])[:10] }
