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
  "time"
)

type Task struct { ID string `json:"id"`; Command string `json:"command"`; ExpectContains string `json:"expect_contains"`; Timeout int `json:"timeout"` }
type SampleLog struct { Task Task `json:"task"`; Status string `json:"status"`; Score float64 `json:"score"`; Output string `json:"output"`; Error string `json:"error,omitempty"`; StartedAt string `json:"started_at"`; DurationMs int64 `json:"duration_ms"`; Trajectory string `json:"trajectory,omitempty"` }
type EvalLog struct { EvalID string `json:"eval_id"`; Status string `json:"status"`; StartedAt string `json:"started_at"`; Samples []SampleLog `json:"samples"`; Metrics map[string]float64 `json:"metrics"`; Format string `json:"format"` }

func loadTasks(path string) ([]Task,error){
  if path=="" { return []Task{{ID:"selftest_pwd",Command:"pwd",ExpectContains:"/Users",Timeout:10},{ID:"selftest_python",Command:"python3 -c 'print(2+2)'",ExpectContains:"4",Timeout:10}},nil }
  b,err:=os.ReadFile(path); if err!=nil { return nil,err }
  var tasks []Task
  if strings.HasSuffix(path,".json") { err=json.Unmarshal(b,&tasks); return tasks,err }
  sc:=bufio.NewScanner(strings.NewReader(string(b)))
  for sc.Scan(){ line:=strings.TrimSpace(sc.Text()); if line==""||strings.HasPrefix(line,"#"){continue}; parts:=strings.SplitN(line,"|",4); if len(parts)<3 {return nil,fmt.Errorf("bad task line: %s",line)}; t:=10; if len(parts)==4 {fmt.Sscanf(parts[3],"%d",&t)}; tasks=append(tasks,Task{ID:parts[0],Command:parts[1],ExpectContains:parts[2],Timeout:t}) }
  return tasks,sc.Err()
}

func runTask(ws, executor string, t Task, outdir string) SampleLog{
  start:=time.Now(); traj:=filepath.Join(outdir,t.ID+".traj.json")
  log:=SampleLog{Task:t,StartedAt:start.Format(time.RFC3339),Trajectory:traj}
  cmd:=exec.Command(executor,"--mode","run","--workspace",ws,"--cmd",t.Command,"--timeout",fmt.Sprint(t.Timeout),"--out",traj)
  b,err:=cmd.CombinedOutput(); log.Output=string(b); log.DurationMs=time.Since(start).Milliseconds()
  if err!=nil { log.Status="error"; log.Error=err.Error(); return log }
  if strings.Contains(log.Output,t.ExpectContains) { log.Status="success"; log.Score=1 } else { log.Status="failed"; log.Score=0; log.Error="expected substring not found" }
  return log
}

func main(){
  mode:=flag.String("mode","selftest","selftest|eval")
  ws:=flag.String("workspace",".","workspace")
  tasksPath:=flag.String("tasks","","json or pipe tasks: id|cmd|expect|timeout")
  out:=flag.String("out","","eval log output")
  flag.Parse()
  abs,_:=filepath.Abs(*ws)
  executor:=filepath.Join(abs,"scripts","apex-mini-executor","apex-mini-executor")
  if _,err:=os.Stat(executor); err!=nil { fmt.Fprintln(os.Stderr,"missing executor:",executor); os.Exit(1) }
  outdir:=filepath.Join(abs,"state","apex-eval-harness")
  _=os.MkdirAll(outdir,0755)
  tasks,err:=loadTasks(*tasksPath); if err!=nil { fmt.Fprintln(os.Stderr,err); os.Exit(2) }
  log:=EvalLog{EvalID:fmt.Sprintf("apex-eval-%d",time.Now().Unix()),StartedAt:time.Now().Format(time.RFC3339),Metrics:map[string]float64{},Format:"apex-eval-harness-1.0"}
  pass:=0
  for _,t:= range tasks { s:=runTask(abs,executor,t,outdir); log.Samples=append(log.Samples,s); if s.Score==1 {pass++} }
  if len(tasks)>0 { log.Metrics["accuracy"]=float64(pass)/float64(len(tasks)) }
  if pass==len(tasks) { log.Status="success" } else { log.Status="failed" }
  outPath:=*out; if outPath=="" { outPath=filepath.Join(abs,"state","apex-eval-harness-latest.json") }
  b,_:=json.MarshalIndent(log,"","  "); _=os.WriteFile(outPath,b,0644); fmt.Println(string(b))
  if *mode=="selftest" && log.Status!="success" { os.Exit(1) }
}
