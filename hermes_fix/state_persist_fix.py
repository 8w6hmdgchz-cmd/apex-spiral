#!/usr/bin/env python3
"""Hermes状态持久化修复"""
import json
import fcntl
from pathlib import Path
from datetime import datetime
from typing import Dict, Any

STATE_DIR = Path.home() / ".hermes" / "state"
STATE_FILE = STATE_DIR / "apex_state.json"
STATE_DIR.mkdir(parents=True, exist_ok=True)

class PersistentState:
    def __init__(self, state_file=STATE_FILE):
        self.state_file = state_file
        self._ensure_state()
    
    def _ensure_state(self):
        if not self.state_file.exists():
            self._write({
                "version": "2.2",
                "delta_g": 0.0,
                "params": {},
                "steps": 0,
                "history": [],
                "genes": [],
                "skills": [],
                "last_update": datetime.now().isoformat()
            })
    
    def _read(self) -> Dict:
        with open(self.state_file, 'r') as f:
            fcntl.flock(f.fileno(), fcntl.LOCK_SH)
            try:
                return json.load(f)
            finally:
                fcntl.flock(f.fileno(), fcntl.LOCK_UN)
    
    def _write(self, data: Dict):
        with open(self.state_file, 'w') as f:
            fcntl.flock(f.fileno(), fcntl.LOCK_EX)
            try:
                json.dump(data, f, indent=2, ensure_ascii=False)
            finally:
                fcntl.flock(f.fileno(), fcntl.LOCK_UN)
    
    def get(self, key: str, default: Any = None) -> Any:
        return self._read().get(key, default)
    
    def set(self, key: str, value: Any):
        state = self._read()
        state[key] = value
        state["last_update"] = datetime.now().isoformat()
        self._write(state)
    
    def append_history(self, entry: Dict):
        state = self._read()
        history = state.get("history", [])
        history.append(entry)
        if len(history) > 1000:
            history = history[-1000:]
        state["history"] = history
        state["last_update"] = datetime.now().isoformat()
        self._write(state)
    
    def get_delta_g(self) -> float:
        return self.get("delta_g", 0.0)
    
    def set_delta_g(self, delta_g: float):
        self.set("delta_g", delta_g)
    
    def increment_steps(self) -> int:
        state = self._read()
        steps = state.get("steps", 0) + 1
        state["steps"] = steps
        state["last_update"] = datetime.now().isoformat()
        self._write(state)
        return steps

def ensure_evolution_dir():
    evo = Path.home() / ".hermes" / "evolution"
    evo.mkdir(parents=True, exist_ok=True)
    (evo / "genes").mkdir(exist_ok=True)
    (evo / "history").mkdir(exist_ok=True)
    (evo / "snapshots").mkdir(exist_ok=True)
    print(f"✅ Evolution目录: {evo}")

if __name__ == "__main__":
    ensure_evolution_dir()
    state = PersistentState()
    print(f"✅ State文件: {STATE_FILE}")
    print(f"   DeltaG: {state.get_delta_g()}")
    print(f"   Steps: {state.get('steps', 0)}")
