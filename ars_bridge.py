#!/usr/bin/env python3
"""
ARS (Auto-Reflux System) Bridge
连接 OpenClaw APEX Hook 与 nanoGPT-claw 的自进化闭环
"""

import http.server
import json
import socketserver
import subprocess
import time
import os
from datetime import datetime

PORT = 18522  # ARS Bridge 端口

APEX_SERVICE = "http://127.0.0.1:18521"
NANOGPT_BINARY = os.path.expanduser("~/.openclaw/workspace/nanoGPT-claw/target/release/nano-gpt-claw")

class ARSHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, format, *args):
        print(f"[ARS] {datetime.now().strftime('%H:%M:%S')} {format % args}")
    
    def send_json(self, data, status=200):
        self.send_response(status)
        self.send_header("Content-type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(data, indent=2).encode())
    
    def do_GET(self):
        if self.path == "/status":
            # 获取APEX状态
            try:
                import urllib.request
                with urllib.request.urlopen(f"{APEX_SERVICE}/apex/status", timeout=3) as resp:
                    apex = json.loads(resp.read().decode())
            except Exception as e:
                apex = {"error": str(e)}
            
            # 获取nanoGPT-claw状态
            nano_status = "not_found"
            nano_tasks = []
            if os.path.exists(NANOGPT_BINARY):
                nano_status = "available"
                try:
                    result = subprocess.run(
                        [NANOGPT_BINARY, "status"],
                        capture_output=True,
                        text=True,
                        timeout=5
                    )
                    if result.returncode == 0:
                        nano_status = "running"
                except:
                    pass
                
                # 获取任务列表
                try:
                    result = subprocess.run(
                        [NANOGPT_BINARY, "task", "list"],
                        capture_output=True,
                        text=True,
                        timeout=5
                    )
                    if result.returncode == 0:
                        nano_tasks = result.stdout[:500]
                except:
                    pass
            
            self.send_json({
                "ars": "active",
                "apex": apex,
                "nanogpt": {
                    "status": nano_status,
                    "tasks": nano_tasks
                },
                "timestamp": datetime.now().isoformat()
            })
            
        elif self.path == "/health":
            self.send_json({"status": "ok", "service": "ars-bridge"})
        else:
            self.send_error(404)
    
    def do_POST(self):
        if self.path == "/evolve":
            # 触发APEX进化
            try:
                import urllib.request
                req = urllib.request.Request(f"{APEX_SERVICE}/apex/evolve", method="POST")
                with urllib.request.urlopen(req, timeout=10) as resp:
                    apex_result = json.loads(resp.read().decode())
                
                delta_g = apex_result.get("after", {}).get("delta_g", 0)
                
                # 使用nanoGPT-claw记录任务
                nano_result = {"task_id": None}
                if os.path.exists(NANOGPT_BINARY):
                    try:
                        # 添加一个benchmark任务
                        task_desc = f"APEX进化优化 - ΔG={delta_g:.4f}"
                        result = subprocess.run(
                            [NANOGPT_BINARY, "task", "add", "benchmark", task_desc],
                            capture_output=True,
                            text=True,
                            timeout=10
                        )
                        if result.returncode == 0:
                            nano_result = {"output": result.stdout[:200]}
                    except Exception as e:
                        nano_result = {"error": str(e)[:100]}
                
                self.send_json({
                    "apex_evolve": apex_result,
                    "nanogpt_task": nano_result,
                    "ars闭环": "完成",
                    "timestamp": datetime.now().isoformat()
                })
            except Exception as e:
                self.send_json({"error": str(e)}, 500)
        else:
            self.send_error(404)

print(f"[ARS] Bridge 启动...")
print(f"[ARS] 端口: {PORT}")
print(f"[ARS] APEX: {APEX_SERVICE}")
print(f"[ARS] nanoGPT: {NANOGPT_BINARY}")

with socketserver.TCPServer(("", PORT), ARSHandler) as httpd:
    print(f"[ARS] ✅ ARS Bridge 就绪!")
    httpd.serve_forever()
