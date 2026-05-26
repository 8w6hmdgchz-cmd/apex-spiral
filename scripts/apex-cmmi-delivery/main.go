package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"time"
)

type Phase struct{ID string `json:"id"`; Name string `json:"name"`; Owner string `json:"owner"`; Standard string `json:"standard"`; Gate string `json:"gate"`; Evidence []string `json:"evidence"`; Status string `json:"status"`}
type Report struct{ID string `json:"id"`; StartedAt string `json:"started_at"`; Objective string `json:"objective"`; Status string `json:"status"`; ContainerMode string `json:"container_mode"`; DockerAvailable bool `json:"docker_available"`; Formula string `json:"formula"`; Phases []Phase `json:"phases"`; Evidence []string `json:"evidence"`; Next string `json:"next"`; Format string `json:"format"`}

func main(){
	mode:=flag.String("mode","plan","plan|selftest|cycle")
	root:=flag.String("root","/Users/lihongxin/.openclaw/workspace","workspace root")
	objective:=flag.String("task","APEX CMMI industrial delivery closed loop","objective")
	out:=flag.String("out","","json output")
	flag.Parse()
	abs,_:=filepath.Abs(*root)
	if *out==""{*out=filepath.Join(abs,"state/apex-cmmi-delivery-latest.json")}
	if *mode=="selftest"{*objective="Validate APEX CMMI delivery harness with planning coding audit test release gates"}
	rep:=run(abs,*objective,*mode)
	writeJSON(*out,rep); fmtJSON(rep)
	if (*mode=="selftest"||*mode=="cycle")&&rep.Status!="success"{os.Exit(1)}
}

func run(root,obj,mode string)Report{
	docker:=hasDocker()
	container:="unavailable_local_sandbox"
	if docker{container="docker_available_preferred"}
	phases:=[]Phase{
		{"P1","Container Backend Detection","runtime platform","CMMI SAM + CM","docker isolated or explicit local fallback",[]string{"state/apex-container-backend-latest.json"},"planned"},
		{"P2","APEX Formula Planning","GPT/phasor planner","CMMI PP + IPM","phasor route + twelve-factor gate",[]string{"state/apex-phasor-llm-latest.json","state/apex-12factor-agent-latest.json"},"planned"},
		{"P3","Claude-Code Verified Implementation Slot","claude code runner","CMMI TS + PI","claude backend detection + git diff + build gate",[]string{"state/apex-claude-code-runner-latest.json","scripts/apex-agent-dispatch/main.go"},"planned"},
		{"P4","APEX PR Audit","GPT/APEX reviewer","CMMI VER + VAL","fusion + evidence validator",[]string{"state/apex-fusion-engine-latest.json","state/apex-fusion-evidence-report.json"},"planned"},
		{"P5","Automated Test Closure","harness","CMMI PPQA + CM","ECC cycle + 12factor + hygiene",[]string{"state/apex-ecc-runtimeos-latest.json","state/apex-hygiene-latest.json"},"planned"},
		{"P6","Evidence Memory Admission","memory governor","CMMI MA + PPQA","evidence validator + sigma memory admission",[]string{"state/apex-memory-admission-latest.json","state/apex-memory-admission-evidence-report.json"},"planned"},
		{"P7","GitHub Release Sync","governance","CMMI CM + DAR","safe rebase/push",[]string{"memory/metrics/task_runs.jsonl"},"planned"},
	}
	status:="success"
	if mode=="cycle"||mode=="selftest"{
		for i:=range phases{for _,e:=range phases[i].Evidence{if !exists(root,e){phases[i].Status="missing_evidence"; status="failed"; break}; phases[i].Status="pass"}}
		if !cmd(root,filepath.Join(root,"scripts/apex-container-backend/apex-container-backend"),"--mode","selftest","--root",root,"--out",filepath.Join(root,"state/apex-container-backend-latest.json")){status="failed"}
		if !cmd(root,filepath.Join(root,"scripts/apex-claude-code-runner/apex-claude-code-runner"),"--mode","selftest","--root",root,"--out",filepath.Join(root,"state/apex-claude-code-runner-latest.json")){status="failed"}
		if !cmd(root,filepath.Join(root,"scripts/apex-fusion-engine/apex-fusion-engine"),"--mode","selftest","--root",root,"--out",filepath.Join(root,"state/apex-fusion-engine-latest.json")){status="failed"}
		if !cmd(root,filepath.Join(root,"scripts/apex-evidence-validator/apex-evidence-validator"),"--mode","validate","--input",filepath.Join(root,"state/apex-fusion-evidence.json"),"--out",filepath.Join(root,"state/apex-fusion-evidence-report.json")){status="failed"}
		if !cmd(root,filepath.Join(root,"scripts/apex-memory-admission/apex-memory-admission"),"--mode","admit","--root",root,"--input","state/apex-fusion-evidence.json","--out",filepath.Join(root,"state/apex-memory-admission-latest.json")){status="failed"}
	}
	return Report{ID:fmt.Sprintf("apex-cmmi-%d",time.Now().Unix()),StartedAt:time.Now().Format(time.RFC3339),Objective:obj,Status:status,ContainerMode:container,DockerAvailable:docker,Formula:"Apex_CMMI = Apex_agent × (Container→Plan→ClaudeCode→Audit→Test→Memory→Release) × EvidenceGate",Phases:phases,Evidence:[]string{"state/apex-container-backend-latest.json","state/apex-claude-code-runner-latest.json","state/apex-cmmi-delivery-latest.json","state/apex-fusion-engine-latest.json","state/apex-fusion-evidence-report.json","state/apex-memory-admission-latest.json"},Next:"use_as_default_delivery_harness",Format:"apex-cmmi-delivery-1.0"}
}

func hasDocker()bool{p,err:=exec.LookPath("docker"); if err!=nil||p==""{return false}; return exec.Command("docker","version","--format","{{.Server.Version}}").Run()==nil}
func exists(root,p string)bool{_,err:=os.Stat(filepath.Join(root,p)); return err==nil}
func cmd(root string,args ...string)bool{c:=exec.Command(args[0],args[1:]...); c.Dir=root; return c.Run()==nil}
func writeJSON(path string,v any){b,_:=json.MarshalIndent(v,"","  "); _=os.WriteFile(path,append(b,'\n'),0644)}
func fmtJSON(v any){b,_:=json.MarshalIndent(v,"","  "); fmt.Println(string(b))}
