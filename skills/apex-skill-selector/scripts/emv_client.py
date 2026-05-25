#!/usr/bin/env python3
"""
EMV Client - Python调用Rust EMV SkillSelector REST API
"""
import urllib.request
import json
from typing import Dict, List, Optional

class EMVClient:
    def __init__(self, base_url: str = "http://127.0.0.1:8080", timeout: float = 5.0):
        self.base_url = base_url
        self.timeout = timeout

    def select_skill(self, payload: dict) -> dict:
        """调用Rust SkillSelector选择技能"""
        data = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(
            f"{self.base_url}/select_skill",
            data=data,
            headers={"Content-Type": "application/json"},
            method="POST"
        )
        with urllib.request.urlopen(req, timeout=self.timeout) as resp:
            return json.loads(resp.read().decode("utf-8"))

    def health(self) -> dict:
        """健康检查"""
        req = urllib.request.Request(
            f"{self.base_url}/api/v1/health",
            method="GET"
        )
        with urllib.request.urlopen(req, timeout=self.timeout) as resp:
            return json.loads(resp.read().decode("utf-8"))

    def list_skills(self) -> dict:
        """列出所有技能"""
        req = urllib.request.Request(
            f"{self.base_url}/api/v1/skills",
            method="GET"
        )
        with urllib.request.urlopen(req, timeout=self.timeout) as resp:
            return json.loads(resp.read().decode("utf-8"))

def build_select_request(
    request_id: str,
    task_type: str,
    goal: str,
    features: dict,
    memory_summary: dict,
    candidates: List[dict],
    gini: float,
    delta_g: float,
    swr_trigger: bool,
    temperature: float = 0.2
) -> dict:
    """构建选择请求"""
    return {
        "request_id": request_id,
        "timestamp_ms": int(__import__("time").time() * 1000),
        "task": {
            "task_type": task_type,
            "goal": goal
        },
        "context": {
            "features": features,
            "memory_summary": memory_summary
        },
        "candidates": candidates,
        "control": {
            "gini": gini,
            "delta_g": delta_g,
            "swr_trigger": swr_trigger,
            "temperature": temperature
        }
    }

if __name__ == "__main__":
    import sys
    
    client = EMVClient()
    
    # 健康检查
    print("=== EMV Health ===")
    try:
        print(json.dumps(client.health(), indent=2))
    except Exception as e:
        print(f"Health check failed: {e}")
    
    # 测试选择
    print("\n=== EMV Select ===")
    req = build_select_request(
        request_id="test-001",
        task_type="classification",
        goal="maximize_accuracy",
        features={"f1": 0.5, "f2": 0.7},
        memory_summary={"recent_success_rate": 0.8, "recent_failure_rate": 0.2},
        candidates=[
            {"skill_id": "gini-select", "prior": 0.4},
            {"skill_id": "swrs-replay", "prior": 0.35},
            {"skill_id": "apex-repair", "prior": 0.25}
        ],
        gini=0.62,
        delta_g=0.08,
        swr_trigger=True
    )
    
    try:
        result = client.select_skill(req)
        print(json.dumps(result, indent=2))
    except Exception as e:
        print(f"Select failed: {e}")
        print("Make sure Rust EMV server is running on port 8080")
