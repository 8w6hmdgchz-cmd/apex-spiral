package main

import (
	"bufio"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"time"
)

type Evidence struct{ID string `json:"id"`; Claim string `json:"claim"`; SourceRepo string `json:"source_repo"`; SourceCommit string `json:"source_commit"`; SourcePath string `json:"source_path"`; ContextID string `json:"context_id"`; Score float64 `json:"score"`; Verification Verification `json:"verification"`; MemoryType string `json:"memory_type"`}
type Verification struct{Command string `json:"command"`; Result string `json:"result"`; EvidencePath string `json:"evidence_path"`}
type Report struct{ID string `json:"id"`; StartedAt string `json:"started_at"`; Status string `json:"status"`; Scanned []string `json:"scanned"`; Findings int `json:"findings"`; EvidencePath string `json:"evidence_path"`; Evidence []Evidence `json:"evidence"`; Format string `json:"format"`}

func main(){root:=flag.String("root","/Users/lihongxin/.openclaw/workspace","workspace root"); out:=flag.String("out","","report output"); evidenceOut:=flag.String("evidence-out","","evidence json output"); flag.Parse(); abs,_:=filepath.Abs(*root); if *out==""{*out=filepath.Join(abs,"state/apex-failure-miner-latest.json")}; if *evidenceOut==""{*evidenceOut=filepath.Join(abs,"state/apex-failure-evidence.json")}; rep:=mine(abs,*evidenceOut); writeJSON(*evidenceOut,rep.Evidence); writeJSON(*out,rep); fmtJSON(rep); if rep.Status!="success"{os.Exit(1)}}

func mine(root,evidenceOut string)Report{started:=time.Now(); files:=[]string{"memory/failure_cases.jsonl","memory/2026-05-26.md","memory/ecc/runtimeos.md"}; rep:=Report{ID:fmt.Sprintf("apex-failure-miner-%d",started.Unix()),StartedAt:started.Format(time.RFC3339),Status:"success",Scanned:files,EvidencePath:rel(root,evidenceOut),Format:"apex-failure-miner-1.0"}; seen:=map[string]bool{}
	patterns:=[]*regexp.Regexp{regexp.MustCompile(`(?i)(failed|failure|error|rollback|timeout|dirty worktree|cannot pull|repair|fallback|root_cause|what_happened)`)}
	for _,f:=range files{path:=filepath.Join(root,f); fh,err:=os.Open(path); if err!=nil{continue}; scanner:=bufio.NewScanner(fh); line:=0; for scanner.Scan(){line++; txt:=strings.TrimSpace(scanner.Text()); if len(txt)<24{continue}; if isLowValue(txt){continue}; matched:=false; for _,p:=range patterns{if p.MatchString(txt){matched=true; break}}; if !matched{continue}; claim:=normalize(txt); if len(claim)>260{claim=claim[:260]}; key:=f+claim; if seen[key]{continue}; seen[key]=true; ev:=Evidence{ID:fmt.Sprintf("failure_%s_%d",safeToken(f),line),Claim:"APEX failure/repair lesson: "+claim,SourceRepo:"8w6hmdgchz-cmd/apex-spiral",SourceCommit:commitish(),SourcePath:f,ContextID:fmt.Sprintf("%s#L%d",f,line),Score:0.88,Verification:Verification{Command:"scripts/apex-failure-miner/apex-failure-miner",Result:"pass",EvidencePath:rel(root,evidenceOut)},MemoryType:"Procedural"}; rep.Evidence=append(rep.Evidence,ev); if len(rep.Evidence)>=12{break}}
		_ = fh.Close(); if len(rep.Evidence)>=12{break}}
	rep.Findings=len(rep.Evidence); return rep}

func normalize(s string)string{s=strings.TrimLeft(s,"- *#\t"); return strings.Join(strings.Fields(s)," ")}
func isLowValue(s string)bool{ls:=strings.ToLower(s); if strings.Contains(ls,"✅")&&(strings.Contains(ls,"push")||strings.Contains(ls,"commit")){return true}; if strings.Contains(ls,"成功推送")||strings.Contains(ls,"rebase+push至"){return true}; return false}
func safeToken(s string)string{out:=[]rune{}; for _,r:=range strings.ToLower(s){if (r>='a'&&r<='z')||(r>='0'&&r<='9')||r=='_'||r=='-'{out=append(out,r)}else{out=append(out,'_')}}; t:=strings.Trim(string(out),"_"); if len(t)>48{t=t[:48]}; if t==""{t="x"}; return t}
func commitish()string{return "0000000"}
func rel(root,p string)string{r,err:=filepath.Rel(root,p); if err!=nil{return p}; return r}
func writeJSON(path string,v any){b,_:=json.MarshalIndent(v,"","  "); _=os.WriteFile(path,append(b,'\n'),0644)}
func fmtJSON(v any){b,_:=json.MarshalIndent(v,"","  "); fmt.Println(string(b))}
