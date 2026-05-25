#!/usr/bin/env python3
"""Cron Jobs 持久化修复"""
import json
import fcntl
from pathlib import Path
from datetime import datetime
from typing import List, Dict

JOBS_FILE = Path.home() / ".hermes" / "cron" / "jobs.json"
JOBS_FILE.parent.mkdir(parents=True, exist_ok=True)

class PersistentCronManager:
    def __init__(self, jobs_file=JOBS_FILE):
        self.jobs_file = jobs_file
        self._ensure_file()
    
    def _ensure_file(self):
        if not self.jobs_file.exists():
            self._write({"jobs": [], "updated_at": datetime.now().isoformat()})
        else:
            try:
                with open(self.jobs_file) as f:
                    json.load(f)
            except:
                self._write({"jobs": [], "updated_at": datetime.now().isoformat()})
    
    def _read(self) -> Dict:
        with open(self.jobs_file, 'r') as f:
            fcntl.flock(f.fileno(), fcntl.LOCK_SH)
            try:
                return json.load(f)
            finally:
                fcntl.flock(f.fileno(), fcntl.LOCK_UN)
    
    def _write(self, data: Dict):
        with open(self.jobs_file, 'w') as f:
            fcntl.flock(f.fileno(), fcntl.LOCK_EX)
            try:
                json.dump(data, f, indent=2, ensure_ascii=False)
            finally:
                fcntl.flock(f.fileno(), fcntl.LOCK_UN)
    
    def list_jobs(self) -> List[Dict]:
        return self._read().get("jobs", [])
    
    def add_job(self, job: Dict) -> str:
        data = self._read()
        job_id = job.get("id") or f"job_{datetime.now().strftime('%Y%m%d%H%M%S')}"
        job["id"] = job_id
        job["created_at"] = datetime.now().isoformat()
        data["jobs"].append(job)
        data["updated_at"] = datetime.now().isoformat()
        self._write(data)
        return job_id
    
    def remove_job(self, job_id: str) -> bool:
        data = self._read()
        original_len = len(data["jobs"])
        data["jobs"] = [j for j in data["jobs"] if j.get("id") != job_id]
        if len(data["jobs"]) < original_len:
            data["updated_at"] = datetime.now().isoformat()
            self._write(data)
            return True
        return False

CRON_JOBS = [
    {
        "name": "开智V2自进化",
        "id": "082fe67eada1",
        "schedule": "*/15 * * * *",
        "prompt": "执行开智V2自进化循环：curl -s http://localhost:8090/api/v1/loop/run",
        "skills": ["gene-kaizhi-evolution"],
        "enabled": True,
        "deliver": ["local", "origin"],
    },
    {
        "name": "APEX Tracker",
        "id": "apex_tracker_hourly",
        "schedule": "0 * * * *",
        "prompt": "执行APEX Tracker检查：读取evolution_tracker_status.json，评估ΔG趋势",
        "skills": ["apex-tiangong-matrix"],
        "enabled": True,
        "deliver": ["local", "origin"],
    }
]

def rebuild_crons():
    manager = PersistentCronManager()
    existing = {j.get("id") for j in manager.list_jobs()}
    
    for job in CRON_JOBS:
        if job["id"] not in existing:
            manager.add_job(job)
            print(f"✅ Created: {job['name']} ({job['id']})")
        else:
            print(f"⏭️  Already exists: {job['name']}")
    
    print(f"\n总计 {len(manager.list_jobs())} jobs")

if __name__ == "__main__":
    rebuild_crons()
