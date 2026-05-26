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
)

type Entry struct{Status string `json:"status"`; Path string `json:"path"`; Category string `json:"category"`; Real bool `json:"real"`; Policy string `json:"policy"`}
type Report struct{Status string `json:"status"`; Total int `json:"total"`; RealDirty int `json:"real_dirty"`; ManagedDirty int `json:"managed_dirty"`; TransientDirty int `json:"transient_dirty"`; VendorDirty int `json:"vendor_dirty"`; ReleaseDirty int `json:"release_dirty"`; ByCategory map[string]int `json:"by_category"`; Policies map[string]string `json:"policies"`; Entries []Entry `json:"entries"`; Format string `json:"format"`}

func category(path string) string {switch{
case strings.HasPrefix(path,"vendor/"): return "vendor"
case strings.HasSuffix(path,".log")||strings.Contains(path,"hub-sync-stderr.log")||path=="auto_reflux.log"||path==".reflux_msg.txt": return "transient"
case strings.HasPrefix(path,"state/a2a-hunt-"): return "transient"
case strings.HasPrefix(path,"releases/")&&strings.HasSuffix(path,"/RELEASE_NOTES.md"): return "managed_release"
case strings.HasPrefix(path,"releases/")&&strings.HasSuffix(path,"/ROLLBACK.json"): return "managed_release"
case strings.HasPrefix(path,"state/apex-eval-harness/"): return "managed_evidence"
case strings.HasPrefix(path,"state/apex-")&&strings.HasSuffix(path,".json"): return "managed_evidence"
case path=="state/phi_tracker_latest.json"||path=="state/phi_v10_result.json"||path=="state/phi_history.jsonl"||path=="state/sigma_memory.json": return "managed_evidence"
case strings.HasPrefix(path,"memory/metrics/"): return "managed_memory"
case strings.HasPrefix(path,"memory/")&&strings.HasSuffix(path,".md"): return "managed_memory"
default: return "source"}}
func policy(cat string)string{switch cat{case "source": return "commit_or_review"; case "transient": return "do_not_commit_or_ignore"; case "managed_evidence": return "commit_only_when_gate_snapshot_intentional"; case "managed_memory": return "commit_when_user/session_memory_relevant"; case "managed_release": return "commit_when_release_manager_prepared"; case "vendor": return "do_not_commit_snapshot_noise"}; return "review"}
func real(cat string) bool{return cat=="source"}
func parsePorcelain(line string)(Entry,bool){if len(line)<4{return Entry{},false}; status:=strings.TrimSpace(line[:2]); path:=strings.TrimSpace(line[3:]); if idx:=strings.Index(path," -> "); idx>=0{path=path[idx+4:]}; cat:=category(path); return Entry{Status:status,Path:path,Category:cat,Real:real(cat),Policy:policy(cat)},true}
func gitStatus(root string)([]Entry,error){cmd:=exec.Command("git","status","--porcelain"); cmd.Dir=root; out,err:=cmd.Output(); if err!=nil{return nil,err}; scanner:=bufio.NewScanner(strings.NewReader(string(out))); var entries []Entry; for scanner.Scan(){if e,ok:=parsePorcelain(scanner.Text()); ok{entries=append(entries,e)}}; return entries,scanner.Err()}
func buildReport(entries []Entry)Report{rep:=Report{Status:"success",ByCategory:map[string]int{},Policies:map[string]string{},Entries:entries,Format:"apex-hygiene-2.0"}; for _,e:=range entries{rep.Total++; rep.ByCategory[e.Category]++; rep.Policies[e.Category]=policy(e.Category); switch e.Category{case "vendor": rep.VendorDirty++; case "transient": rep.TransientDirty++; case "managed_memory","managed_evidence": rep.ManagedDirty++; case "managed_release": rep.ReleaseDirty++}; if e.Real{rep.RealDirty++}}; return rep}
func main(){root:=flag.String("root",".","workspace root"); out:=flag.String("out","","write JSON report"); mode:=flag.String("mode","status","status|real-count"); flag.Parse(); abs,_:=filepath.Abs(*root); entries,err:=gitStatus(abs); if err!=nil{fmt.Fprintln(os.Stderr,err); os.Exit(1)}; rep:=buildReport(entries); if *mode=="real-count"{fmt.Println(rep.RealDirty); return}; b,_:=json.MarshalIndent(rep,"","  "); if *out!=""{_ = os.WriteFile(*out,b,0644)}; fmt.Println(string(b))}
