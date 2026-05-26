package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"math"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

type Evidence struct{ID string `json:"id"`; Claim string `json:"claim"`; SourceRepo string `json:"source_repo"`; SourceCommit string `json:"source_commit"`; SourcePath string `json:"source_path"`; ContextID string `json:"context_id"`; Score float64 `json:"score"`; MemoryType string `json:"memory_type"`}
type MemoryEntry struct{ID string `json:"id"`; Content string `json:"content"`; Embedding []float64 `json:"embedding"`; Timestamp int64 `json:"timestamp"`; Importance float64 `json:"importance"`; MemoryType string `json:"memory_type"`; AccessCount int `json:"access_count"`; Source string `json:"source"`}
type Sigma struct{LearnRate float64 `json:"learn_rate"`; DecayFactor float64 `json:"decay_factor"`; RetentionThreshold float64 `json:"retention_threshold"`; Entries []MemoryEntry `json:"memory_entries"`}
type Report struct{ID string `json:"id"`; StartedAt string `json:"started_at"`; Status string `json:"status"`; Input string `json:"input"`; Validated bool `json:"validated"`; Added int `json:"added"`; Skipped int `json:"skipped"`; MemoryCount int `json:"memory_count"`; SigmaMemory float64 `json:"sigma_memory"`; TypeCounts map[string]int `json:"type_counts"`; Evidence []string `json:"evidence"`; Format string `json:"format"`}

func main(){mode:=flag.String("mode","admit","admit|selftest"); root:=flag.String("root","/Users/lihongxin/.openclaw/workspace","workspace root"); input:=flag.String("input","state/apex-fusion-evidence.json","evidence json"); out:=flag.String("out","","report json"); flag.Parse(); abs,_:=filepath.Abs(*root); if *mode=="selftest"{*input="state/apex-fusion-evidence.json"}; if *out==""{*out=filepath.Join(abs,"state/apex-memory-admission-latest.json")}; rep:=admit(abs,*input); writeJSON(*out,rep); fmtJSON(rep); if rep.Status!="success"{os.Exit(1)}}

func admit(root,input string)Report{started:=time.Now(); inPath:=filepath.Join(root,input); report:=Report{ID:fmt.Sprintf("apex-memory-admit-%d",started.Unix()),StartedAt:started.Format(time.RFC3339),Status:"success",Input:input,Evidence:[]string{input,"state/sigma_memory.json"},Format:"apex-memory-admission-1.0"}
	validator:=filepath.Join(root,"scripts/apex-evidence-validator/apex-evidence-validator")
	validReport:=filepath.Join(root,"state/apex-memory-admission-evidence-report.json")
	if !cmd(root,validator,"--mode","validate","--input",inPath,"--out",validReport){report.Status="failed"; report.Evidence=append(report.Evidence,"state/apex-memory-admission-evidence-report.json"); return report}
	report.Validated=true; report.Evidence=append(report.Evidence,"state/apex-memory-admission-evidence-report.json")
	ev:=[]Evidence{}; b,err:=os.ReadFile(inPath); if err!=nil{report.Status="failed"; return report}; if err:=json.Unmarshal(b,&ev); err!=nil{report.Status="failed"; return report}
	sig:=loadSigma(filepath.Join(root,"state/sigma_memory.json")); existing:=map[string]bool{}; for _,e:=range sig.Entries{existing[e.ID]=true}
	for _,e:=range ev{id:="evidence_"+safeID(e.ID); if existing[id]{report.Skipped++; continue}; mt:=e.MemoryType; if mt==""{mt="Working"}; sig.Entries=append(sig.Entries,MemoryEntry{ID:id,Content:fmt.Sprintf("[%s] %s (source=%s@%s:%s context=%s)",mt,e.Claim,e.SourceRepo,e.SourceCommit,e.SourcePath,e.ContextID),Embedding:[]float64{0,0,0},Timestamp:started.Unix(),Importance:clamp(e.Score,.7,.99),MemoryType:mt,AccessCount:int(20+e.Score*20),Source:input}); existing[id]=true; report.Added++}
	sort.Slice(sig.Entries,func(i,j int)bool{if sig.Entries[i].Importance==sig.Entries[j].Importance{return sig.Entries[i].Timestamp>sig.Entries[j].Timestamp}; return sig.Entries[i].Importance>sig.Entries[j].Importance}); if len(sig.Entries)>500{sig.Entries=sig.Entries[:500]}; if sig.LearnRate==0{sig.LearnRate=.928}; if sig.DecayFactor==0{sig.DecayFactor=.988}; if sig.RetentionThreshold==0{sig.RetentionThreshold=.372}
	writeJSON(filepath.Join(root,"state/sigma_memory.json"),sig); report.MemoryCount=len(sig.Entries); report.TypeCounts=counts(sig.Entries); report.SigmaMemory=sigma(sig); return report}

func loadSigma(path string)Sigma{b,err:=os.ReadFile(path); if err!=nil{return Sigma{LearnRate:.928,DecayFactor:.988,RetentionThreshold:.372}}; s:=Sigma{}; _=json.Unmarshal(b,&s); return s}
func counts(es []MemoryEntry)map[string]int{m:=map[string]int{}; for _,e:=range es{m[e.MemoryType]++}; return m}
func sigma(s Sigma)float64{n:=len(s.Entries); if n==0{return 0}; c:=counts(s.Entries); H:=0.0; for _,v:=range c{p:=float64(v)/float64(n); H+=-p*math.Log2(p)}; avg:=0.0; for _,e:=range s.Entries{avg+=e.Importance}; avg/=float64(n); return round(s.LearnRate*math.Sqrt(s.RetentionThreshold*s.LearnRate)*math.Min(1,H/1.5)*(float64(len(c))/4)*s.DecayFactor*(0.4+0.6*avg))}
func safeID(s string)string{h:=sha256.Sum256([]byte(s)); return hex.EncodeToString(h[:])[:16]}
func clamp(x,a,b float64)float64{if x<a{return a}; if x>b{return b}; return x}
func round(x float64)float64{return float64(int(x*10000+0.5))/10000}
func cmd(root string,args ...string)bool{c:=exec.Command(args[0],args[1:]...); c.Dir=root; return c.Run()==nil}
func writeJSON(path string,v any){b,_:=json.MarshalIndent(v,"","  "); _=os.WriteFile(path,append(b,'\n'),0644)}
func fmtJSON(v any){b,_:=json.MarshalIndent(v,"","  "); fmt.Println(string(b))}
func _trim(s string)string{return strings.TrimSpace(s)}
