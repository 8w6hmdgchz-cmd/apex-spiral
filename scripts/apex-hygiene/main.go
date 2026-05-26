package main

import (
	"bufio"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

type Entry struct {
	Status   string `json:"status"`
	Path     string `json:"path"`
	Category string `json:"category"`
	Real     bool   `json:"real"`
}

type Report struct {
	Status         string         `json:"status"`
	Total          int            `json:"total"`
	RealDirty      int            `json:"real_dirty"`
	ManagedDirty   int            `json:"managed_dirty"`
	TransientDirty int            `json:"transient_dirty"`
	VendorDirty    int            `json:"vendor_dirty"`
	ByCategory     map[string]int `json:"by_category"`
	Entries        []Entry        `json:"entries"`
	Format         string         `json:"format"`
}

func category(path string) string {
	switch {
	case strings.HasPrefix(path, "vendor/"):
		return "vendor"
	case strings.HasSuffix(path, ".log"), strings.Contains(path, "hub-sync-stderr.log"), path == "auto_reflux.log":
		return "transient"
	case strings.HasPrefix(path, "state/a2a-hunt-"):
		return "transient"
	case strings.HasPrefix(path, "state/apex-eval-harness/"):
		return "managed_evidence"
	case strings.HasPrefix(path, "state/apex-") && strings.HasSuffix(path, ".json"):
		return "managed_evidence"
	case strings.HasPrefix(path, "memory/metrics/"):
		return "managed_memory"
	case strings.HasPrefix(path, "memory/") && strings.HasSuffix(path, ".md"):
		return "managed_memory"
	default:
		return "source"
	}
}

func real(cat string) bool {
	return cat == "source"
}

func parsePorcelain(line string) (Entry, bool) {
	if len(line) < 4 {
		return Entry{}, false
	}
	status := strings.TrimSpace(line[:2])
	path := strings.TrimSpace(line[3:])
	if idx := strings.Index(path, " -> "); idx >= 0 {
		path = path[idx+4:]
	}
	cat := category(path)
	return Entry{Status: status, Path: path, Category: cat, Real: real(cat)}, true
}

func gitStatus(root string) ([]Entry, error) {
	cmd := exec.Command("git", "status", "--porcelain")
	cmd.Dir = root
	out, err := cmd.Output()
	if err != nil {
		return nil, err
	}
	scanner := bufio.NewScanner(strings.NewReader(string(out)))
	var entries []Entry
	for scanner.Scan() {
		if e, ok := parsePorcelain(scanner.Text()); ok {
			entries = append(entries, e)
		}
	}
	return entries, scanner.Err()
}

func buildReport(entries []Entry) Report {
	rep := Report{Status: "success", ByCategory: map[string]int{}, Entries: entries, Format: "apex-hygiene-1.0"}
	for _, e := range entries {
		rep.Total++
		rep.ByCategory[e.Category]++
		switch e.Category {
		case "vendor":
			rep.VendorDirty++
		case "transient":
			rep.TransientDirty++
		case "managed_memory", "managed_evidence":
			rep.ManagedDirty++
		}
		if e.Real {
			rep.RealDirty++
		}
	}
	return rep
}

func main() {
	root := flag.String("root", ".", "workspace root")
	out := flag.String("out", "", "write JSON report")
	mode := flag.String("mode", "status", "status|real-count")
	flag.Parse()

	abs, _ := filepath.Abs(*root)
	entries, err := gitStatus(abs)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	rep := buildReport(entries)
	if *mode == "real-count" {
		fmt.Println(rep.RealDirty)
		return
	}
	b, _ := json.MarshalIndent(rep, "", "  ")
	if *out != "" {
		_ = os.WriteFile(*out, b, 0644)
	}
	fmt.Println(string(b))
}
