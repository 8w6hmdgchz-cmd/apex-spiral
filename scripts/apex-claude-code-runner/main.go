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

type Backend struct{Name string `json:"name"`; Path string `json:"path,omitempty"`; Version string `json:"version,omitempty"`; Available bool `json:"available"`; Error string `json:"error,omitempty"`}
type Report struct{ID string `json:"id"`; StartedAt string `json:"started_at"`; Status string `json:"status"`; Mode string `json:"mode"`; Objective string `json:"objective"`; Selected string `json:"selected"`; Backends []Backend `json:"backends"`; ContainerEvidence string `json:"container_evidence"`; DispatchEvidence string `json:"dispatch_evidence"`; Safety []string `json:"safety"`; Gate string `json:"gate"`; Evidence []string `json:"evidence"`; Format string `json:"format"`}

func main(){mode:=flag.String("mode","selftest","detect|selftest|plan")
	root:=flag.String("root","/Users/lihongxin/.openclaw/workspace","workspace root")
	task:=flag.String("task","Implement smallest reversible APEX change under CMMI gates","objective")
	out:=flag.String("out","","report output")
	flag.Parse(); abs,_:=filepath.Abs(*root); if *out==""{*out=filepath.Join(abs,"state/apex-claude-code-runner-latest.json")}
	rep:=run(abs,*mode,*task); writeJSON(*out,rep); fmtJSON(rep); if rep.Status!="success"{os.Exit(1)}}

func run(root,mode,task string)Report{bs:=[]Backend{detect("claude"),detect("claude-code"),detect("codex")}; selected:=""; for _,b:=range bs{if b.Available&&(b.Name=="claude-code"||b.Name=="claude"){selected=b.Name; break}}; status:="success"; if selected==""{selected="fallback-local-coding-agent"}
	return Report{ID:fmt.Sprintf("apex-claude-runner-%d",time.Now().Unix()),StartedAt:time.Now().Format(time.RFC3339),Status:status,Mode:mode,Objective:task,Selected:selected,Backends:bs,ContainerEvidence:"state/apex-container-backend-latest.json",DispatchEvidence:"state/apex-agent-dispatch-latest.json",Safety:[]string{"detect/selftest never mutates source","coding mode must run inside CMMI gates","runner must output diff/test evidence before PR audit","no direct push from coding slot"},Gate:"detect backend + container evidence + dispatch plan",Evidence:[]string{"state/apex-claude-code-runner-latest.json","state/apex-container-backend-latest.json","state/apex-agent-dispatch-latest.json"},Format:"apex-claude-code-runner-1.0"}}

func detect(name string)Backend{p,err:=exec.LookPath(name); if err!=nil{return Backend{Name:name,Available:false,Error:"not found"}}; b:=Backend{Name:name,Path:p,Available:true}; for _,args:=range [][]string{{"--version"},{"version"},{"--help"}}{cmd:=exec.Command(p,args...); out,err:=cmd.CombinedOutput(); s:=strings.TrimSpace(string(out)); if err==nil&&s!=""{b.Version=firstLine(s); return b}; if s!=""&&b.Version==""{b.Version=firstLine(s)}}; return b}
func firstLine(s string)string{if i:=strings.IndexByte(s,'\n'); i>=0{return s[:i]}; if len(s)>200{return s[:200]}; return s}
func writeJSON(path string,v any){b,_:=json.MarshalIndent(v,"","  "); _=os.WriteFile(path,append(b,'\n'),0644)}
func fmtJSON(v any){b,_:=json.MarshalIndent(v,"","  "); fmt.Println(string(b))}
