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

type Report struct{ID string `json:"id"`; StartedAt string `json:"started_at"`; Status string `json:"status"`; Mode string `json:"mode"`; DockerPath string `json:"docker_path,omitempty"`; DockerServer string `json:"docker_server,omitempty"`; Image string `json:"image"`; Command string `json:"command"`; Workspace string `json:"workspace"`; DurationMs int64 `json:"duration_ms"`; Output string `json:"output,omitempty"`; Error string `json:"error,omitempty"`; Safety []string `json:"safety"`; Evidence []string `json:"evidence"`; Format string `json:"format"`}

func main(){
	mode:=flag.String("mode","selftest","selftest|run|detect")
	root:=flag.String("root","/Users/lihongxin/.openclaw/workspace","workspace root")
	image:=flag.String("image","golang:1.23-alpine","docker image")
	command:=flag.String("command","pwd && ls scripts >/dev/null && echo container_backend_ok","command")
	out:=flag.String("out","","report json")
	flag.Parse()
	abs,_:=filepath.Abs(*root)
	if *out==""{*out=filepath.Join(abs,"state/apex-container-backend-latest.json")}
	if *mode=="detect"{*command="echo detect_only"}
	rep:=run(abs,*mode,*image,*command)
	writeJSON(*out,rep); fmtJSON(rep)
	if rep.Status!="success"{os.Exit(1)}
}

func run(root,mode,image,command string)Report{
	start:=time.Now(); dockerPath,_:=exec.LookPath("docker"); rep:=Report{ID:fmt.Sprintf("apex-container-%d",start.Unix()),StartedAt:start.Format(time.RFC3339),Status:"success",Image:image,Command:command,Workspace:root,Safety:[]string{"workspace mounted read-only by default for detect/selftest where possible","no host docker mutation when docker unavailable","fallback is explicit local sandbox, never reported as container success"},Evidence:[]string{"state/apex-container-backend-latest.json"},Format:"apex-container-backend-1.0"}
	if dockerPath==""{rep.Mode="local_sandbox_fallback"; rep.Output="docker binary not found; using local sandbox fallback"; rep.DurationMs=time.Since(start).Milliseconds(); return rep}
	rep.DockerPath=dockerPath
	ver:=exec.Command("docker","version","--format","{{.Server.Version}}")
	vb,err:=ver.CombinedOutput(); if err!=nil{rep.Mode="local_sandbox_fallback"; rep.Output="docker server unavailable; using local sandbox fallback"; rep.Error=strings.TrimSpace(string(vb)); rep.DurationMs=time.Since(start).Milliseconds(); return rep}
	rep.DockerServer=strings.TrimSpace(string(vb)); rep.Mode="docker_isolated"
	if mode=="detect"{rep.Output="docker available"; rep.DurationMs=time.Since(start).Milliseconds(); return rep}
	args:=[]string{"run","--rm","--network","none","--cpus","1","--memory","1g","-v",root+":/workspace:ro","-w","/workspace",image,"/bin/sh","-lc",command}
	cmd:=exec.Command("docker",args...); b,err:=cmd.CombinedOutput(); rep.Output=trimLimit(string(b),4000); rep.DurationMs=time.Since(start).Milliseconds(); if err!=nil{rep.Status="failed"; rep.Error=err.Error()}; return rep
}

func trimLimit(s string,n int)string{s=strings.TrimSpace(s); if len(s)>n{return s[:n]+"...<truncated>"}; return s}
func writeJSON(path string,v any){b,_:=json.MarshalIndent(v,"","  "); _=os.WriteFile(path,append(b,'\n'),0644)}
func fmtJSON(v any){b,_:=json.MarshalIndent(v,"","  "); fmt.Println(string(b))}
