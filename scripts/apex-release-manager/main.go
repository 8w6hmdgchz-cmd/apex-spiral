package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

type Artifact struct{Path string `json:"path"`; SHA256 string `json:"sha256"`; Size int64 `json:"size"`}
type Report struct{ID string `json:"id"`; StartedAt string `json:"started_at"`; Status string `json:"status"`; Version string `json:"version"`; Commit string `json:"commit"`; PreviousCommit string `json:"previous_commit"`; Branch string `json:"branch"`; ReleaseNotes string `json:"release_notes"`; RollbackManifest string `json:"rollback_manifest"`; Artifacts []Artifact `json:"artifacts"`; Gates []string `json:"gates"`; PublishMode string `json:"publish_mode"`; Evidence []string `json:"evidence"`; Format string `json:"format"`}

func main(){mode:=flag.String("mode","prepare","prepare|selftest")
	root:=flag.String("root","/Users/lihongxin/.openclaw/workspace","workspace root")
	version:=flag.String("version","","release version")
	out:=flag.String("out","","report output")
	flag.Parse(); abs,_:=filepath.Abs(*root); if *version==""{*version="v0.0.0-apex-"+time.Now().Format("20060102-150405")}; if *out==""{*out=filepath.Join(abs,"state/apex-release-manager-latest.json")}
	rep:=prepare(abs,*version,*mode); writeJSON(*out,rep); fmtJSON(rep); if rep.Status!="success"{os.Exit(1)}}

func prepare(root,version,mode string)Report{started:=time.Now(); relDir:=filepath.Join(root,"releases",version); _=os.MkdirAll(relDir,0755)
	commit:=git(root,"rev-parse","--short=12","HEAD"); prev:=git(root,"rev-parse","--short=12","HEAD~1"); branch:=git(root,"rev-parse","--abbrev-ref","HEAD")
	gates:=[]string{"state/apex-container-backend-latest.json","state/apex-claude-code-runner-latest.json","state/apex-cmmi-delivery-latest.json","state/apex-fusion-evidence-report.json","state/apex-memory-admission-latest.json","state/phi_tracker_latest.json"}
	status:="success"; for _,g:=range gates{if _,err:=os.Stat(filepath.Join(root,g)); err!=nil{status="failed"}}
	notes:=fmt.Sprintf("# APEX Release %s\n\n- Commit: `%s`\n- Previous: `%s`\n- Branch: `%s`\n- Generated: `%s`\n\n## Gates\n\n",version,commit,prev,branch,started.Format(time.RFC3339)); for _,g:=range gates{notes+=fmt.Sprintf("- `%s`\n",g)}; notes+="\n## Publish Mode\n\nPrepared locally; GitHub Release publishing requires explicit external release permission/tooling.\n"
	rollback:=fmt.Sprintf("{\n  \"version\": \"%s\",\n  \"commit\": \"%s\",\n  \"rollback_to\": \"%s\",\n  \"command\": \"git checkout %s\",\n  \"generated_at\": \"%s\"\n}\n",version,commit,prev,prev,started.Format(time.RFC3339))
	notesPath:=filepath.Join(relDir,"RELEASE_NOTES.md"); rbPath:=filepath.Join(relDir,"ROLLBACK.json"); _=os.WriteFile(notesPath,[]byte(notes),0644); _=os.WriteFile(rbPath,[]byte(rollback),0644)
	arts:=[]Artifact{}; for _,p:=range []string{rel(root,notesPath),rel(root,rbPath)}{if a,ok:=artifact(root,p); ok{arts=append(arts,a)}}
	return Report{ID:fmt.Sprintf("apex-release-%d",started.Unix()),StartedAt:started.Format(time.RFC3339),Status:status,Version:version,Commit:commit,PreviousCommit:prev,Branch:branch,ReleaseNotes:rel(root,notesPath),RollbackManifest:rel(root,rbPath),Artifacts:arts,Gates:gates,PublishMode:"prepared_local_no_external_release",Evidence:[]string{"state/apex-release-manager-latest.json",rel(root,notesPath),rel(root,rbPath)},Format:"apex-release-manager-1.0"}}

func artifact(root,p string)(Artifact,bool){b,err:=os.ReadFile(filepath.Join(root,p)); if err!=nil{return Artifact{},false}; h:=sha256.Sum256(b); return Artifact{Path:p,SHA256:hex.EncodeToString(h[:]),Size:int64(len(b))},true}
func git(root string,args ...string)string{cmd:=exec.Command("git",append([]string{"-C",root},args...)...); b,err:=cmd.Output(); if err!=nil{return "unknown"}; return strings.TrimSpace(string(b))}
func rel(root,path string)string{r,err:=filepath.Rel(root,path); if err!=nil{return path}; return r}
func writeJSON(path string,v any){b,_:=json.MarshalIndent(v,"","  "); _=os.WriteFile(path,append(b,'\n'),0644)}
func fmtJSON(v any){b,_:=json.MarshalIndent(v,"","  "); fmt.Println(string(b))}
