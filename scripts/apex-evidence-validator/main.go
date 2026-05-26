package main

import (
  "encoding/json"
  "flag"
  "fmt"
  "os"
  "path/filepath"
  "regexp"
  "strings"
)

type Verification struct { Command string `json:"command"`; Result string `json:"result"`; EvidencePath string `json:"evidence_path"` }
type Evidence struct { ID string `json:"id"`; Claim string `json:"claim"`; SourceRepo string `json:"source_repo"`; SourceCommit string `json:"source_commit"`; SourcePath string `json:"source_path"`; ContextID string `json:"context_id"`; Score float64 `json:"score"`; Verification Verification `json:"verification"`; MemoryType string `json:"memory_type,omitempty"` }
type Report struct { Status string `json:"status"`; Checked int `json:"checked"`; Passed int `json:"passed"`; Errors []string `json:"errors"`; Format string `json:"format"` }

var idRe=regexp.MustCompile(`^[A-Za-z0-9_.:-]+$`)
var shaRe=regexp.MustCompile(`^[0-9a-f]{7,40}$`)

func badPath(p string) bool { return p=="" || filepath.IsAbs(p) || strings.Contains(p,"..") }
func validMem(t string) bool { return t==""||t=="Working"||t=="Procedural"||t=="Semantic"||t=="Episodic" }

func validate(e Evidence, idx int) []string{
  var errs []string; prefix:=fmt.Sprintf("$[%d]",idx)
  if !idRe.MatchString(e.ID) || len(e.ID)<3 { errs=append(errs,prefix+".id invalid") }
  if len(e.Claim)<12 { errs=append(errs,prefix+".claim too short") }
  if len(e.SourceRepo)<3 { errs=append(errs,prefix+".source_repo missing") }
  if !shaRe.MatchString(e.SourceCommit) { errs=append(errs,prefix+".source_commit must be 7-40 hex") }
  if badPath(e.SourcePath) { errs=append(errs,prefix+".source_path must be relative and safe") }
  if len(e.ContextID)<3 { errs=append(errs,prefix+".context_id missing") }
  if e.Score<0 || e.Score>1 { errs=append(errs,prefix+".score outside 0..1") }
  if len(e.Verification.Command)<3 { errs=append(errs,prefix+".verification.command missing") }
  if e.Verification.Result!="pass" && e.Verification.Result!="fail" && e.Verification.Result!="blocked" { errs=append(errs,prefix+".verification.result invalid") }
  if badPath(e.Verification.EvidencePath) { errs=append(errs,prefix+".verification.evidence_path must be relative and safe") }
  if !validMem(e.MemoryType) { errs=append(errs,prefix+".memory_type invalid") }
  if e.Score < 0.7 { errs=append(errs,prefix+".score below hard gate 0.70") }
  if e.Verification.Result != "pass" { errs=append(errs,prefix+".verification.result must pass for memory admission") }
  return errs
}

func load(path string) ([]Evidence,error){
  b,err:=os.ReadFile(path); if err!=nil {return nil,err}
  var arr []Evidence
  if err:=json.Unmarshal(b,&arr); err==nil {return arr,nil}
  var one Evidence
  if err:=json.Unmarshal(b,&one); err!=nil {return nil,err}
  return []Evidence{one},nil
}

func selftest(out string) error{
  samples:=[]Evidence{{ID:"selftest:mini-exec",Claim:"APEX mini executor selftest passed with trajectory evidence.",SourceRepo:"SWE-agent/mini-swe-agent",SourceCommit:"adfe20233d456104c38c3129161b54f0fd39f2c7",SourcePath:"vendor/github/SWE-agent/mini-swe-agent/src/minisweagent/agents/default.py",ContextID:"mini-selftest-001",Score:.95,Verification:Verification{Command:"scripts/apex-mini-executor/apex-mini-executor --mode selftest",Result:"pass",EvidencePath:"state/apex-mini-executor-selftest.traj.json"},MemoryType:"Working"}}
  b,_:=json.MarshalIndent(samples,"","  "); return os.WriteFile(out,b,0644)
}

func main(){
  mode:=flag.String("mode","validate","validate|selftest")
  input:=flag.String("input","","evidence json")
  out:=flag.String("out","","report output")
  flag.Parse()
  if *mode=="selftest" { path:=*input; if path=="" {path="state/apex-evidence-selftest.json"}; if err:=selftest(path); err!=nil {fmt.Fprintln(os.Stderr,err); os.Exit(1)}; *input=path }
  evs,err:=load(*input); if err!=nil {fmt.Fprintln(os.Stderr,err); os.Exit(2)}
  rep:=Report{Checked:len(evs),Format:"apex-evidence-validator-1.0"}
  for i,e:=range evs { errs:=validate(e,i); if len(errs)==0 {rep.Passed++} else {rep.Errors=append(rep.Errors,errs...)} }
  if rep.Passed==rep.Checked {rep.Status="success"} else {rep.Status="failed"}
  b,_:=json.MarshalIndent(rep,"","  "); fmt.Println(string(b)); if *out!="" {_=os.WriteFile(*out,b,0644)}; if rep.Status!="success" {os.Exit(1)}
}
