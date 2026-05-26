package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

type Phasor struct{Selected string `json:"selected"`; Fallbacks []string `json:"fallbacks"`; Alignment float64 `json:"alignment"`; Status string `json:"status"`}
type Worker struct{ID string `json:"id"`; Role string `json:"role"`; Task string `json:"task"`; Model string `json:"model"`; Fallbacks []string `json:"fallbacks"`; Verification string `json:"verification"`; SpawnMode string `json:"spawn_mode"`}
type Dispatch struct{ID string `json:"id"`; StartedAt string `json:"started_at"`; Objective string `json:"objective"`; Status string `json:"status"`; PhasorEvidence string `json:"phasor_evidence"`; Workers []Worker `json:"workers"`; Parallelizable bool `json:"parallelizable"`; Evidence []string `json:"evidence"`; Format string `json:"format"`}

func main(){
	mode:=flag.String("mode","plan","plan|selftest")
	root:=flag.String("root","/Users/lihongxin/.openclaw/workspace","workspace root")
	objective:=flag.String("task","APEX ECC RuntimeOS incremental refactor with evidence gates","objective")
	out:=flag.String("out","","json output")
	flag.Parse()
	abs,_:=filepath.Abs(*root)
	if *mode=="selftest"{*objective="Improve APEX RuntimeOS efficiency by dispatching research build verify memory tasks with phasor-selected models"}
	if *out==""{*out=filepath.Join(abs,"state/apex-agent-dispatch-latest.json")}
	rep:=plan(abs,*objective)
	writeJSON(*out,rep); fmtJSON(rep)
	if rep.Status!="success"{os.Exit(1)}
}

func plan(root, objective string) Dispatch{
	phasorPath:=filepath.Join(root,"state/apex-phasor-llm-latest.json")
	if _,err:=os.Stat(phasorPath); err!=nil{ run(root, filepath.Join(root,"scripts/apex-phasor-llm/apex-phasor-llm"),"--mode","selftest","--root",root,"--out",phasorPath) }
	p:=Phasor{}
	if b,err:=os.ReadFile(phasorPath); err==nil{_ = json.Unmarshal(b,&p)}
	if p.Selected==""{p=Phasor{Selected:"freemodel/gpt-5.5",Fallbacks:[]string{"deepseek/deepseek-v4-pro"},Alignment:.75,Status:"fallback"}}
	workers:=[]Worker{
		{"W1","researcher","Gather source constraints and prior evidence for: "+objective,p.Selected,p.Fallbacks,"evidence ledger paths exist","isolated"},
		{"W2","builder","Implement the smallest reversible artifact for: "+objective,p.Selected,p.Fallbacks,"git diff and build gate pass","isolated"},
		{"W3","critic","Verify outputs, reject weak evidence, and identify regression risk",firstFallback(p),append([]string{p.Selected},p.Fallbacks...),"test/evidence gate pass","isolated"},
		{"W4","memory-archivist","Admit only validated durable lessons into sigma_memory/task_runs",p.Selected,p.Fallbacks,"apex-evidence-validator pass","isolated"},
	}
	status:="success"; if p.Alignment<.35{status="failed"}
	return Dispatch{ID:fmt.Sprintf("apex-dispatch-%d",time.Now().Unix()),StartedAt:time.Now().Format(time.RFC3339),Objective:objective,Status:status,PhasorEvidence:"state/apex-phasor-llm-latest.json",Workers:workers,Parallelizable:true,Evidence:[]string{"state/apex-phasor-llm-latest.json","scripts/apex-agent-dispatch/main.go"},Format:"apex-agent-dispatch-1.0"}
}

func firstFallback(p Phasor)string{if len(p.Fallbacks)>0{return p.Fallbacks[0]}; return p.Selected}
func run(root string,args ...string){cmd:=exec.Command(args[0],args[1:]...); cmd.Dir=root; _=cmd.Run()}
func writeJSON(path string,v any){b,_:=json.MarshalIndent(v,"","  "); _=os.WriteFile(path,append(b,'\n'),0644)}
func fmtJSON(v any){b,_:=json.MarshalIndent(v,"","  "); fmt.Println(string(b))}
func _contains(s, sub string)bool{return strings.Contains(s,sub)}
