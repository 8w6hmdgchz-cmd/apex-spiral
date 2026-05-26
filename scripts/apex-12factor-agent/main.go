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

type Factor struct {
	ID       string   `json:"id"`
	Name     string   `json:"name"`
	APEXRule string   `json:"apex_rule"`
	Checks   []string `json:"checks"`
	Evidence []string `json:"evidence"`
	Score    float64  `json:"score"`
	Status   string   `json:"status"`
	Missing  []string `json:"missing,omitempty"`
}

type Report struct {
	ID        string   `json:"id"`
	StartedAt string   `json:"started_at"`
	Formula   string   `json:"formula"`
	Root      string   `json:"root"`
	Status    string   `json:"status"`
	Score     float64  `json:"score"`
	Passed    int      `json:"passed"`
	Total     int      `json:"total"`
	Factors   []Factor `json:"factors"`
	Next      string   `json:"next"`
	Format    string   `json:"format"`
}

func main(){
	mode:=flag.String("mode","audit","audit|selftest")
	root:=flag.String("root","/Users/lihongxin/.openclaw/workspace","workspace root")
	out:=flag.String("out","","json output")
	flag.Parse()
	abs,_:=filepath.Abs(*root)
	if *out==""{*out=filepath.Join(abs,"state/apex-12factor-agent-latest.json")}
	rep:=audit(abs)
	writeJSON(*out,rep)
	fmtJSON(rep)
	if *mode=="selftest" && rep.Status!="success"{os.Exit(1)}
}

func audit(root string) Report{
	factors:=[]Factor{
		mk("F01","Codebase","one tracked codebase, many runtime deploys",[]string{".git","scripts","skills"}),
		mk("F02","Dependencies","dependencies explicit and buildable",[]string{"scripts/apex-ecc-runtimeos/go.mod","scripts/apex-fusion-engine/go.mod","scripts/apex-praison-chain/go.mod"}),
		mk("F03","Config","config/state separated from code",[]string{"state","schemas/apex-evidence.schema.json"}),
		mk("F04","Backing Services","tools/services attached through declared bridges",[]string{"scripts/apex-harness-bridge/apex-harness-bridge","scripts/apex-evidence-validator/apex-evidence-validator"}),
		mk("F05","Build Release Run","build artifacts are generated before run",[]string{"scripts/apex-ecc-runtimeos/apex-ecc-runtimeos","scripts/apex-fusion-engine/apex-fusion-engine"}),
		mk("F06","Processes","agent work is stateless/replayable via state artifacts",[]string{"state/apex-ecc-runtimeos-latest.json","state/apex-fusion-engine-latest.json"}),
		mk("F07","Port Binding","runtime APIs are exposed via CLI/MCP boundaries",[]string{"scripts/apex-harness-bridge/apex-harness-bridge"}),
		mk("F08","Concurrency","multi-agent orchestration is explicit and bounded",[]string{"scripts/apex-praison-chain/apex-praison-chain","state/apex-praison-activation.json"}),
		mk("F09","Disposability","cycles can start/stop safely with evidence",[]string{"scripts/apex_ecc_nightly.sh","scripts/apex-hygiene/apex-hygiene"}),
		mk("F10","Dev Prod Parity","manual and cron cycles use same gates",[]string{"scripts/apex_ecc_nightly.sh","state/apex-ecc-runtimeos-latest.json"}),
		mk("F11","Logs","logs and metrics are append-only observability",[]string{"memory/metrics/task_runs.jsonl","state/phi_history.jsonl"}),
		mk("F12","Admin Processes","admin/nightly tasks run as one-off governed processes",[]string{"scripts/apex_ecc_nightly.sh"}),
	}
	pass:=0
	for i:=range factors{
		for _,p:=range factors[i].Checks{ if exists(root,p){ factors[i].Evidence=append(factors[i].Evidence,p) } else { factors[i].Missing=append(factors[i].Missing,p) } }
		if len(factors[i].Missing)==0{ factors[i].Status="pass"; factors[i].Score=1; pass++ } else { factors[i].Status="fail"; factors[i].Score=float64(len(factors[i].Evidence))/float64(len(factors[i].Checks)) }
	}
	status:="success"; next:="admit_12factor_evidence"
	if pass!=len(factors){status="failed"; next="repair_missing_factor"}
	return Report{ID:fmt.Sprintf("apex-12factor-%d",time.Now().Unix()),StartedAt:time.Now().Format(time.RFC3339),Formula:"Apex_agent = ΔG ⊙ Π(F1..F12)",Root:root,Status:status,Score:round(float64(pass)/float64(len(factors))),Passed:pass,Total:len(factors),Factors:factors,Next:next,Format:"apex-12factor-agent-1.0"}
}

func mk(id,name,rule string,checks []string)Factor{return Factor{ID:id,Name:name,APEXRule:rule,Checks:checks}}
func exists(root,p string)bool{_,err:=os.Stat(filepath.Join(root,p)); return err==nil}
func round(x float64)float64{return float64(int(x*1000+0.5))/1000}
func writeJSON(path string,v any){b,_:=json.MarshalIndent(v,"","  "); _=os.WriteFile(path,append(b,'\n'),0644)}
func fmtJSON(v any){b,_:=json.MarshalIndent(v,"","  "); fmt.Println(string(b))}
func currentCommit(root string)string{b,err:=exec.Command("git","-C",root,"rev-parse","--short=12","HEAD").Output(); if err!=nil{return "0000000"}; return strings.TrimSpace(string(b))}
