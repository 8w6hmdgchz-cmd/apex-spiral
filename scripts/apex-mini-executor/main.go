package main

import (
  "crypto/sha256"
  "encoding/hex"
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

type Event struct {
  RunID string `json:"run_id"`
  Step int `json:"step"`
  Role string `json:"role"`
  Command string `json:"command,omitempty"`
  Output string `json:"output,omitempty"`
  ReturnCode int `json:"returncode,omitempty"`
  Error string `json:"error,omitempty"`
  StartedAt string `json:"started_at"`
  DurationMs int64 `json:"duration_ms"`
  InputHash string `json:"input_hash,omitempty"`
  OutputHash string `json:"output_hash,omitempty"`
}

type Trajectory struct {
  Info map[string]any `json:"info"`
  Messages []Event `json:"messages"`
  TrajectoryFormat string `json:"trajectory_format"`
}

func hash(s string) string { h:=sha256.Sum256([]byte(s)); return hex.EncodeToString(h[:])[:16] }

func inWorkspace(ws, path string) bool {
  absWs, _ := filepath.Abs(ws); absPath, _ := filepath.Abs(path)
  rel, err := filepath.Rel(absWs, absPath)
  return err==nil && rel != ".." && !strings.HasPrefix(rel, "../") && !filepath.IsAbs(rel)
}

func validateCommand(cmd string) error {
  banned := []string{"rm -rf", "sudo", "mkfs", ":(){", "dd if=", "> /dev/", "curl ", "wget ", "ssh ", "scp ", "rsync ", "chmod -R 777"}
  low := strings.ToLower(cmd)
  for _, b := range banned { if strings.Contains(low,b) { return fmt.Errorf("banned token: %s", b) } }
  allowedPrefixes := []string{"echo ", "printf ", "pwd", "ls", "cat ", "grep ", "rg ", "find ", "python3 ", "go ", "cargo ", "git status", "git diff", "test ", "[ ", "true", "false"}
  trimmed := strings.TrimSpace(cmd)
  for _, p := range allowedPrefixes { if trimmed==strings.TrimSpace(p) || strings.HasPrefix(trimmed,p) { return nil } }
  return fmt.Errorf("command not allowlisted: %s", trimmed)
}

func runCommand(runID string, step int, ws, command string, timeout time.Duration, dry bool) Event {
  start:=time.Now(); ev:=Event{RunID:runID, Step:step, Role:"observation", Command:command, StartedAt:start.Format(time.RFC3339), InputHash:hash(command)}
  if err:=validateCommand(command); err!=nil { ev.ReturnCode=-1; ev.Error=err.Error(); ev.DurationMs=time.Since(start).Milliseconds(); return ev }
  if dry { ev.Output="DRY_RUN: command validated, not executed"; ev.ReturnCode=0; ev.DurationMs=time.Since(start).Milliseconds(); ev.OutputHash=hash(ev.Output); return ev }
  if !inWorkspace(ws, ws) { ev.ReturnCode=-1; ev.Error="workspace path invalid"; return ev }
  shell := exec.Command("/bin/zsh", "-lc", command)
  shell.Dir = ws
  shell.Env = os.Environ()
  timer := time.AfterFunc(timeout, func(){ _ = shell.Process.Kill() })
  out, err := shell.CombinedOutput(); timer.Stop()
  ev.Output=string(out); ev.OutputHash=hash(ev.Output); ev.DurationMs=time.Since(start).Milliseconds()
  if err!=nil { ev.ReturnCode=1; ev.Error=err.Error() } else { ev.ReturnCode=0 }
  if ev.DurationMs >= timeout.Milliseconds() { ev.Error="timeout"; ev.ReturnCode=-1 }
  return ev
}

func writeJSON(path string, v any) error { b,_:=json.MarshalIndent(v,"","  "); return os.WriteFile(path,b,0644) }

func selftest(ws string) error {
  out:=filepath.Join(ws,"state","apex-mini-executor-selftest.traj.json")
  traj, err := execute(ws, []string{"pwd", "printf 'ok\\n'", "python3 -c 'print(2+2)'"}, false, 10*time.Second)
  if err!=nil { return err }
  if len(traj.Messages)!=3 { return errors.New("expected 3 messages") }
  if traj.Messages[2].Output != "4\n" { return fmt.Errorf("unexpected python output: %q", traj.Messages[2].Output) }
  return writeJSON(out,traj)
}

func execute(ws string, commands []string, dry bool, timeout time.Duration) (Trajectory,error) {
  runID:=fmt.Sprintf("apex-mini-%d", time.Now().Unix())
  traj:=Trajectory{Info:map[string]any{"run_id":runID,"workspace":ws,"dry_run":dry,"started_at":time.Now().Format(time.RFC3339)},TrajectoryFormat:"apex-mini-executor-1.0"}
  for i,c:=range commands { traj.Messages=append(traj.Messages, runCommand(runID,i+1,ws,c,timeout,dry)) }
  return traj,nil
}

func main(){
  mode:=flag.String("mode","selftest","selftest|run")
  ws:=flag.String("workspace",".","workspace directory")
  cmd:=flag.String("cmd","pwd","command for run mode")
  out:=flag.String("out","","trajectory output path")
  dry:=flag.Bool("dry-run",false,"validate without executing")
  timeout:=flag.Int("timeout",30,"timeout seconds")
  flag.Parse()
  abs,_:=filepath.Abs(*ws)
  switch *mode{
  case "selftest": if err:=selftest(abs); err!=nil { fmt.Fprintln(os.Stderr,err); os.Exit(1)}; fmt.Println("selftest_ok")
  case "run": traj,err:=execute(abs,[]string{*cmd},*dry,time.Duration(*timeout)*time.Second); if err!=nil {fmt.Fprintln(os.Stderr,err); os.Exit(1)}; if *out!="" { _=writeJSON(*out,traj)}; b,_:=json.MarshalIndent(traj,"","  "); fmt.Println(string(b))
  default: fmt.Fprintln(os.Stderr,"unknown mode"); os.Exit(2)
  }
}
