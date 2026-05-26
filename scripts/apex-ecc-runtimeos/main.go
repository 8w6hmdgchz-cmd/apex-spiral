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

type Domain struct {
	Name        string   `json:"name"`
	Native      []string `json:"native"`
	Target      []string `json:"target"`
	Gate        []string `json:"gate"`
	Status      string   `json:"status"`
	Evidence    []string `json:"evidence"`
}

type RuntimeReport struct {
	ID          string   `json:"id"`
	StartedAt   string   `json:"started_at"`
	Mode        string   `json:"mode"`
	Status      string   `json:"status"`
	Root        string   `json:"root"`
	Domains     []Domain `json:"domains"`
	FusionOK    bool     `json:"fusion_ok"`
	EvidenceOK  bool     `json:"evidence_ok"`
	SecurityOK  bool     `json:"security_ok"`
	Governance  []string `json:"governance"`
	NextAction   string   `json:"next_action"`
	Format      string   `json:"format"`
}

func main(){
	mode:=flag.String("mode","audit","audit|plan|cycle|selftest")
	root:=flag.String("root","/Users/lihongxin/.openclaw/workspace","workspace root")
	out:=flag.String("out","","output json")
	flag.Parse()
	abs,_:=filepath.Abs(*root)
	if *out==""{*out=filepath.Join(abs,"state/apex-ecc-runtimeos-latest.json")}
	rep:=run(abs,*mode)
	writeJSON(*out,rep)
	fmtJSON(rep)
	if (*mode=="selftest"||*mode=="cycle") && rep.Status!="success"{os.Exit(1)}
}

func run(root,mode string) RuntimeReport{
	rep:=RuntimeReport{ID:fmt.Sprintf("apex-ecc-%d",time.Now().Unix()),StartedAt:time.Now().Format(time.RFC3339),Mode:mode,Status:"success",Root:root,Format:"apex-ecc-runtimeos-1.0"}
	rep.Domains=[]Domain{
		{"Skills",[]string{"skills/*/SKILL.md","scripts/*"},[]string{"plugin registry","skill contract","buildable helpers"},[]string{"skill docs exist","helper builds"},"planned",[]string{"skills/apex-praison-chain/SKILL.md"}},
		{"Memory",[]string{"MEMORY.md","memory/*.md","state/sigma_memory.json"},[]string{"evidence admission","working/procedural density"},[]string{"apex-evidence-validator","sigma_memory update"},"planned",[]string{"state/sigma_memory.json"}},
		{"Hooks",[]string{"cron","auto_reflux.sh","phi_tracker.sh"},[]string{"nightly controlled evolution","no tight loops"},[]string{"cron job enabled","full_mirror PHI"},"planned",[]string{"scripts/auto_reflux.sh"}},
		{"Rules",[]string{"AGENTS.md","SOUL.md","TOOLS.md"},[]string{"no fake data","approval gates","destructive-action pause"},[]string{"superpowers gate","evidence validator"},"planned",[]string{"skills/apex-ecc-runtimeos/SKILL.md"}},
		{"Multi-agent",[]string{"sessions_spawn","apex-praison-chain"},[]string{"role graph","critic separation"},[]string{"praison activation"},"planned",[]string{"state/apex-praison-activation.json"}},
		{"Session State",[]string{"state/*.json","trajectory logs"},[]string{"resume-safe artifacts","bounded context"},[]string{"fusion engine report"},"planned",[]string{"state/apex-fusion-engine-latest.json"}},
		{"Security",[]string{"apex-secrets-gate","apex-hygiene","harness bridge"},[]string{"safe command boundary","secret scan","workspace bound"},[]string{"security gate pass"},"planned",[]string{"scripts/apex-secrets-gate/apex-secrets-gate"}},
		{"Observability",[]string{"task_runs.jsonl","phi_history.jsonl","fusion reports"},[]string{"traceable runs","metrics from artifacts"},[]string{"append-only task run"},"planned",[]string{"memory/metrics/task_runs.jsonl"}},
		{"Governance",[]string{"evidence schema","superpowers gate"},[]string{"human approval for risky actions","quality gates"},[]string{"evidence validator"},"planned",[]string{"schemas/apex-evidence.schema.json"}},
		{"Learning",[]string{"evolver","autoresearch","devour ledgers"},[]string{"devour->distill->reimplement->verify->admit"},[]string{"fusion gate"},"planned",[]string{"scripts/apex-evolver-core/apex-evolver-core"}},
	}
	if mode=="plan"||mode=="audit"{rep.NextAction="run_cycle_with_fusion_and_evidence"; return rep}
	fusion:=cmd(root, filepath.Join(root,"scripts/apex-fusion-engine/apex-fusion-engine"),"--mode","selftest","--root",root,"--out",filepath.Join(root,"state/apex-fusion-engine-latest.json"))
	rep.FusionOK=fusion
	evidence:=cmd(root, filepath.Join(root,"scripts/apex-evidence-validator/apex-evidence-validator"),"--mode","validate","--input",filepath.Join(root,"state/apex-fusion-evidence.json"),"--out",filepath.Join(root,"state/apex-fusion-evidence-report.json"))
	rep.EvidenceOK=evidence
	sec:=cmd(root, filepath.Join(root,"scripts/apex-hygiene/apex-hygiene"),"--root",root,"--out",filepath.Join(root,"state/apex-hygiene-latest.json"))
	rep.SecurityOK=sec
	twelve:=cmd(root, filepath.Join(root,"scripts/apex-12factor-agent/apex-12factor-agent"),"--mode","selftest","--root",root,"--out",filepath.Join(root,"state/apex-12factor-agent-latest.json"))
	if fusion&&evidence&&sec&&twelve{for i:=range rep.Domains{rep.Domains[i].Status="active"}; rep.NextAction="nightly_incremental_refactor"}else{rep.Status="failed"; rep.NextAction="repair_failed_gate"}
	rep.Governance=[]string{"No destructive operations without explicit approval.","No fabricated metrics; PHI must read full_mirror artifacts.","Every upgrade must pass fusion + evidence + hygiene gates.","Nightly work must be incremental, committed, and reversible."}
	return rep
}

func cmd(root string,args ...string)bool{c:=exec.Command(args[0],args[1:]...); c.Dir=root; return c.Run()==nil}
func writeJSON(path string,v any){b,_:=json.MarshalIndent(v,"","  "); _=os.WriteFile(path,append(b,'\n'),0644)}
func fmtJSON(v any){b,_:=json.MarshalIndent(v,"","  "); fmt.Println(string(b))}
