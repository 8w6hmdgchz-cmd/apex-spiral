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
type Report struct{ID string `json:"id"`; StartedAt string `json:"started_at"`; Status string `json:"status"`; Input string `json:"input"`; Validated bool `json:"validated"`; Added int `json:"added"`; Skipped int `json:"skipped"`; Rejected int `json:"rejected"`; Deduped int `json:"deduped"`; Pruned int `json:"pruned"`; MemoryCount int `json:"memory_count"`; SigmaMemory float64 `json:"sigma_memory"`; TypeCounts map[string]int `json:"type_counts"`; RejectionReasons map[string]int `json:"rejection_reasons"`; QualityFloor float64 `json:"quality_floor"`; Capacity int `json:"capacity"`; Evidence []string `json:"evidence"`; Format string `json:"format"`}

func main(){mode:=flag.String("mode","admit","admit|selftest"); root:=flag.String("root","/Users/lihongxin/.openclaw/workspace","workspace root"); input:=flag.String("input","state/apex-fusion-evidence.json","evidence json"); out:=flag.String("out","","report json"); floor:=flag.Float64("quality-floor",0.72,"minimum score"); capacity:=flag.Int("capacity",500,"max memory entries"); flag.Parse(); abs,_:=filepath.Abs(*root); if *mode=="selftest"{*input="state/apex-fusion-evidence.json"}; if *out==""{*out=filepath.Join(abs,"state/apex-memory-admission-latest.json")}; rep:=admit(abs,*input,*floor,*capacity); writeJSON(*out,rep); fmtJSON(rep); if rep.Status!="success"{os.Exit(1)}}

func admit(root,input string,floor float64,capacity int)Report{started:=time.Now(); inPath:=filepath.Join(root,input); report:=Report{ID:fmt.Sprintf("apex-memory-admit-%d",started.Unix()),StartedAt:started.Format(time.RFC3339),Status:"success",Input:input,QualityFloor:floor,Capacity:capacity,RejectionReasons:map[string]int{},Evidence:[]string{input,"state/sigma_memory.json"},Format:"apex-memory-admission-2.0"}
	validator:=filepath.Join(root,"scripts/apex-evidence-validator/apex-evidence-validator"); validReport:=filepath.Join(root,"state/apex-memory-admission-evidence-report.json")
	if !cmd(root,validator,"--mode","validate","--input",inPath,"--out",validReport){report.Status="failed"; report.Evidence=append(report.Evidence,"state/apex-memory-admission-evidence-report.json"); return report}
	report.Validated=true; report.Evidence=append(report.Evidence,"state/apex-memory-admission-evidence-report.json")
	ev:=[]Evidence{}; b,err:=os.ReadFile(inPath); if err!=nil{report.Status="failed"; report.RejectionReasons["input_read_error"]++; return report}; if err:=json.Unmarshal(b,&ev); err!=nil{report.Status="failed"; report.RejectionReasons["input_parse_error"]++; return report}
	sig:=loadSigma(filepath.Join(root,"state/sigma_memory.json")); before:=len(sig.Entries); existingID:=map[string]bool{}; existingContent:=map[string]bool{}; for _,e:=range sig.Entries{existingID[e.ID]=true; existingContent[contentKey(e.Content)]=true}
	for _,e:=range ev{
		if e.Score<floor{report.Rejected++; report.RejectionReasons["below_quality_floor"]++; continue}
		if len(strings.TrimSpace(e.Claim))<40{report.Rejected++; report.RejectionReasons["claim_too_short"]++; continue}
		id:="evidence_"+safeID(e.ID); if existingID[id]{report.Skipped++; continue}
		mt:=normalizeType(e.MemoryType,e.Claim); content:=fmt.Sprintf("[%s] %s (source=%s@%s:%s context=%s)",mt,e.Claim,e.SourceRepo,e.SourceCommit,e.SourcePath,e.ContextID); ck:=contentKey(content); if existingContent[ck]{report.Deduped++; continue}
		imp:=quality(e,mt); sig.Entries=append(sig.Entries,MemoryEntry{ID:id,Content:content,Embedding:[]float64{0,0,0},Timestamp:started.Unix(),Importance:imp,MemoryType:mt,AccessCount:int(20+imp*25),Source:input}); existingID[id]=true; existingContent[ck]=true; report.Added++
	}
	sig.Entries=dedupeEntries(sig.Entries,&report); sort.Slice(sig.Entries,func(i,j int)bool{si:=rankScore(sig.Entries[i]); sj:=rankScore(sig.Entries[j]); if si==sj{return sig.Entries[i].Timestamp>sig.Entries[j].Timestamp}; return si>sj}); if capacity<1{capacity=500}; if len(sig.Entries)>capacity{report.Pruned+=len(sig.Entries)-capacity; sig.Entries=sig.Entries[:capacity]}; if sig.LearnRate==0{sig.LearnRate=.928}; if sig.DecayFactor==0{sig.DecayFactor=.988}; if sig.RetentionThreshold==0{sig.RetentionThreshold=.372}
	writeJSON(filepath.Join(root,"state/sigma_memory.json"),sig); report.MemoryCount=len(sig.Entries); report.TypeCounts=counts(sig.Entries); report.SigmaMemory=sigma(sig); if before==report.MemoryCount&&report.Added==0&&report.Rejected>0{report.Status="success"}; return report}

func normalizeType(mt,claim string)string{if mt==""{mt="Working"}; lc:=strings.ToLower(claim); if strings.Contains(lc,"rollback")||strings.Contains(lc,"failure")||strings.Contains(lc,"repair")||strings.Contains(lc,"failed")||strings.Contains(lc,"safe rebase"){return "Procedural"}; switch mt{case "Working","Procedural","Semantic","Episodic":return mt}; return "Working"}
func quality(e Evidence,mt string)float64{q:=clamp(e.Score,.72,.99); if e.SourceCommit!=""&&len(e.SourceCommit)>=7{q+=.01}; if e.SourcePath!=""{q+=.01}; if e.ContextID!=""{q+=.005}; if mt=="Procedural"{q+=.005}; return clamp(q,.72,.99)}
func rankScore(e MemoryEntry)float64{return e.Importance + math.Min(float64(e.AccessCount),60)/1000}
func contentKey(s string)string{s=strings.ToLower(s); if i:=strings.Index(s,"(source="); i>=0{s=s[:i]}; return safeID(strings.Join(strings.Fields(s)," "))}
func dedupeEntries(es []MemoryEntry, r *Report)[]MemoryEntry{seen:=map[string]bool{}; out:=[]MemoryEntry{}; sort.Slice(es,func(i,j int)bool{return rankScore(es[i])>rankScore(es[j])}); for _,e:=range es{k:=contentKey(e.Content); if seen[k]{r.Deduped++; continue}; seen[k]=true; out=append(out,e)}; return out}
func loadSigma(path string)Sigma{b,err:=os.ReadFile(path); if err!=nil{return Sigma{LearnRate:.928,DecayFactor:.988,RetentionThreshold:.372}}; s:=Sigma{}; _=json.Unmarshal(b,&s); return s}
func counts(es []MemoryEntry)map[string]int{m:=map[string]int{}; for _,e:=range es{m[e.MemoryType]++}; return m}
func sigma(s Sigma)float64{n:=len(s.Entries); if n==0{return 0}; c:=counts(s.Entries); H:=0.0; for _,v:=range c{p:=float64(v)/float64(n); H+=-p*math.Log2(p)}; avg:=0.0; for _,e:=range s.Entries{avg+=e.Importance}; avg/=float64(n); return round(s.LearnRate*math.Sqrt(s.RetentionThreshold*s.LearnRate)*math.Min(1,H/1.5)*(float64(len(c))/4)*s.DecayFactor*(0.4+0.6*avg))}
func safeID(s string)string{h:=sha256.Sum256([]byte(s)); return hex.EncodeToString(h[:])[:16]}
func clamp(x,a,b float64)float64{if x<a{return a}; if x>b{return b}; return x}
func round(x float64)float64{return float64(int(x*10000+0.5))/10000}
func cmd(root string,args ...string)bool{c:=exec.Command(args[0],args[1:]...); c.Dir=root; return c.Run()==nil}
func writeJSON(path string,v any){b,_:=json.MarshalIndent(v,"","  "); _=os.WriteFile(path,append(b,'\n'),0644)}
func fmtJSON(v any){b,_:=json.MarshalIndent(v,"","  "); fmt.Println(string(b))}
