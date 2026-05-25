package main

import (
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

type BridgeRequest struct {
	Protocol string            `json:"protocol"`
	Action   string            `json:"action"`
	Tool     string            `json:"tool"`
	Args     []string          `json:"args"`
	Cwd      string            `json:"cwd"`
	Timeout  int               `json:"timeout_seconds"`
	Env      map[string]string `json:"env,omitempty"`
}

type BridgeResponse struct {
	OK        bool     `json:"ok"`
	Action    string   `json:"action"`
	Tool      string   `json:"tool"`
	Cwd       string   `json:"cwd"`
	ExitCode  int      `json:"exit_code"`
	Stdout    string   `json:"stdout"`
	Stderr    string   `json:"stderr"`
	StartedAt string   `json:"started_at"`
	EndedAt   string   `json:"ended_at"`
	Duration  string   `json:"duration"`
	Error     string   `json:"error,omitempty"`
	Evidence  []string `json:"evidence"`
}

var allowedTools = map[string]bool{
	"git": true, "go": true, "cargo": true, "python3": true, "node": true, "npm": true,
	"ls": true, "find": true, "grep": true, "sed": true, "cat": true, "wc": true,
	"bash": true,
}

var blockedTokens = []string{"rm", "shutdown", "reboot", "mkfs", "dd", "sudo", "chmod 777", "chown", "curl |", "wget |"}

func main() {
	mode := flag.String("mode", "run", "run|schema|selftest")
	reqPath := flag.String("request", "", "JSON request path; stdin when empty")
	flag.Parse()

	switch *mode {
	case "schema":
		printSchema()
	case "selftest":
		selftest()
	case "run":
		req, err := readRequest(*reqPath)
		if err != nil {
			emitError("read", err)
			os.Exit(2)
		}
		resp := run(req)
		emit(resp)
		if !resp.OK {
			os.Exit(1)
		}
	default:
		fmt.Fprintf(os.Stderr, "unknown mode: %s\n", *mode)
		os.Exit(2)
	}
}

func readRequest(path string) (BridgeRequest, error) {
	var b []byte
	var err error
	if path == "" {
		b, err = os.ReadFile("/dev/stdin")
	} else {
		b, err = os.ReadFile(path)
	}
	if err != nil {
		return BridgeRequest{}, err
	}
	var req BridgeRequest
	if err := json.Unmarshal(b, &req); err != nil {
		return BridgeRequest{}, err
	}
	if req.Protocol == "" {
		req.Protocol = "apex-cli-mcp/v1"
	}
	if req.Action == "" {
		req.Action = "sandbox.exec"
	}
	if req.Timeout <= 0 {
		req.Timeout = 30
	}
	return req, nil
}

func run(req BridgeRequest) BridgeResponse {
	start := time.Now()
	resp := BridgeResponse{Action: req.Action, Tool: req.Tool, Cwd: req.Cwd, StartedAt: start.Format(time.RFC3339), Evidence: []string{"request_protocol=" + req.Protocol}}
	if err := validate(req); err != nil {
		resp.EndedAt = time.Now().Format(time.RFC3339)
		resp.Duration = time.Since(start).String()
		resp.Error = err.Error()
		return resp
	}
	ctx, cancel := context.WithTimeout(context.Background(), time.Duration(req.Timeout)*time.Second)
	defer cancel()
	cmd := exec.CommandContext(ctx, req.Tool, req.Args...)
	cmd.Dir = req.Cwd
	cmd.Env = os.Environ()
	for k, v := range req.Env {
		cmd.Env = append(cmd.Env, k+"="+v)
	}
	out, err := cmd.Output()
	resp.Stdout = string(out)
	if err != nil {
		if ee, ok := err.(*exec.ExitError); ok {
			resp.Stderr = string(ee.Stderr)
			resp.ExitCode = ee.ExitCode()
		} else {
			resp.Error = err.Error()
			resp.ExitCode = -1
		}
	} else {
		resp.OK = true
		resp.ExitCode = 0
	}
	if ctx.Err() == context.DeadlineExceeded {
		resp.Error = "timeout"
		resp.ExitCode = -1
		resp.OK = false
	}
	resp.EndedAt = time.Now().Format(time.RFC3339)
	resp.Duration = time.Since(start).String()
	resp.Evidence = append(resp.Evidence, "cwd="+req.Cwd, "tool="+req.Tool)
	return resp
}

func validate(req BridgeRequest) error {
	if req.Action != "sandbox.exec" && req.Action != "mcp.exec" {
		return fmt.Errorf("unsupported action: %s", req.Action)
	}
	if !allowedTools[req.Tool] {
		return fmt.Errorf("tool not allowed: %s", req.Tool)
	}
	cwd := req.Cwd
	if cwd == "" {
		return errors.New("cwd required")
	}
	abs, err := filepath.Abs(cwd)
	if err != nil {
		return err
	}
	root := "/Users/lihongxin/.openclaw/workspace"
	if !strings.HasPrefix(abs, root) {
		return fmt.Errorf("cwd outside workspace: %s", abs)
	}
	joined := req.Tool + " " + strings.Join(req.Args, " ")
	for _, b := range blockedTokens {
		if strings.Contains(joined, b) {
			return fmt.Errorf("blocked token: %s", b)
		}
	}
	return nil
}

func printSchema() {
	emit(map[string]any{"protocol": "apex-cli-mcp/v1", "actions": []string{"sandbox.exec", "mcp.exec"}, "allowed_tools": keys(allowedTools), "root": "/Users/lihongxin/.openclaw/workspace"})
}

func selftest() {
	req := BridgeRequest{Protocol: "apex-cli-mcp/v1", Action: "sandbox.exec", Tool: "go", Args: []string{"version"}, Cwd: "/Users/lihongxin/.openclaw/workspace", Timeout: 10}
	emit(run(req))
}

func emit(v any) { b, _ := json.MarshalIndent(v, "", "  "); fmt.Println(string(b)) }
func emitError(action string, err error) {
	emit(BridgeResponse{OK: false, Action: action, ExitCode: -1, Error: err.Error(), EndedAt: time.Now().Format(time.RFC3339)})
}
func keys(m map[string]bool) []string {
	ks := make([]string, 0, len(m))
	for k := range m {
		ks = append(ks, k)
	}
	return ks
}
