#!/usr/bin/env python3
"""
三层记忆架构 + SWR + Gini选择
基于海马体SWRs机制设计

层次：
1. 短期记忆：RingBuffer (原样保留)
2. 中期记忆：ReplayBuffer (Gini选择高价值片段)
3. 长期记忆：LongTermMemory (知识库/摘要库)
4. 调度层：SWRMemoryManager
"""

import json, sys, time, math
from pathlib import Path
from collections import defaultdict

# ============ Gini公式 ============
def gini_impurity(labels: list) -> float:
    """Gini不纯度: Gini = 1 - Σpk²"""
    if not labels:
        return 0.0
    counts = defaultdict(int)
    for l in labels:
        counts[l] += 1
    total = len(labels)
    return 1.0 - sum((c / total) ** 2 for c in counts.values())

def gini_gain(parent_labels: list, left_labels: list, right_labels: list) -> float:
    """基尼增益: ΔGini = Gini父 - (NL/N×GiniL + NR/N×GiniR)"""
    n = len(parent_labels)
    if n == 0:
        return 0.0
    gini_parent = gini_impurity(parent_labels)
    gini_left = gini_impurity(left_labels)
    gini_right = gini_impurity(right_labels)
    nl = len(left_labels)
    nr = len(right_labels)
    return gini_parent - (nl / n * gini_left + nr / n * gini_right)

def information_gain(parent_labels: list, left_labels: list, right_labels: list) -> float:
    """信息增益: IG = H父 - Σ(Nv/N×Hv)"""
    n = len(parent_labels)
    if n == 0:
        return 0.0
    h_parent = entropy(parent_labels)
    h_left = entropy(left_labels)
    h_right = entropy(right_labels)
    nl = len(left_labels)
    nr = len(right_labels)
    return h_parent - (nl / n * h_left + nr / n * h_right)

def entropy(labels: list) -> float:
    """信息熵: H = -Σpk×log2(pk)"""
    if not labels:
        return 0.0
    counts = defaultdict(int)
    for l in labels:
        counts[l] += 1
    total = len(labels)
    h = 0.0
    for c in counts.values():
        p = c / total
        if p > 0:
            h -= p * math.log2(p)
    return h

def surprise_score(entry: dict, recent_stats: dict) -> float:
    """惊讶度：与当前分布差异(Gini/熵变化)"""
    entry_val = entry.get("fitness", 0.5)
    mean = recent_stats.get("mean", 0.5)
    std = recent_stats.get("std", 0.1)
    if std == 0:
        return 0.0
    return abs(entry_val - mean) / std

# ============ RingBuffer (短期记忆) ============
class RingBuffer:
    """O(1) RingBuffer，保留最近N条交互/状态"""
    def __init__(self, capacity: int = 128, threshold: float = 0.7):
        self.capacity = capacity
        self.threshold = threshold
        self.buffer = []  # [{skill, fitness, timestamp, data}]
        self.head = 0

    def push(self, skill: str, fitness: float, data: dict = None, timestamp: float = None) -> dict:
        """O(1)追加，fitness>=threshold才写入"""
        if fitness < self.threshold:
            return {"archived": False, "reason": f"fitness {fitness:.3f} < {self.threshold}"}
        
        ts = timestamp or time.time()
        entry = {"skill": skill, "fitness": fitness, "timestamp": ts, "data": data or {}}
        
        if len(self.buffer) < self.capacity:
            self.buffer.append(entry)
        else:
            self.buffer[self.head] = entry
        
        self.head = (self.head + 1) % self.capacity
        return {"archived": True, "entry": entry}

    def sample_window(self, window_size: int = 10) -> list:
        """采样最近window_size条"""
        if not self.buffer:
            return []
        # 逆序取最近window_size条
        samples = []
        for i in range(min(window_size, len(self.buffer))):
            idx = (self.head - 1 - i) % len(self.buffer)
            samples.append(self.buffer[idx])
        return samples

    def get_all(self) -> list:
        return self.buffer

    def stats(self) -> dict:
        if not self.buffer:
            return {"len": 0, "mean": 0, "std": 0, "max": 0, "min": 0}
        fitnesses = [e["fitness"] for e in self.buffer]
        return {
            "len": len(self.buffer),
            "mean": sum(fitnesses) / len(fitnesses),
            "std": self._std(fitnesses),
            "max": max(fitnesses),
            "min": min(fitnesses),
        }

    def _std(self, values: list) -> float:
        if not values:
            return 0.0
        mean = sum(values) / len(values)
        variance = sum((v - mean) ** 2 for v in values) / len(values)
        return math.sqrt(variance)

# ============ ReplayBuffer (中期记忆) ============
class ReplayBuffer:
    """SWR重放池，Gini选择高价值片段"""
    def __init__(self, capacity: int = 256):
        self.capacity = capacity
        self.buffer = []  # [{entry, score, gini_delta, selected_at}]
        self.gini_threshold = 0.05  # 最小Gini增益阈值

    def compute_scores(self, candidates: list, recent_stats: dict) -> list:
        """计算每条候选的综合分数"""
        scored = []
        for entry in candidates:
            surprise = surprise_score(entry, recent_stats)
            reward = entry.get("fitness", 0.5)
            # 简单综合分数
            score = 0.4 * surprise + 0.6 * reward
            scored.append((entry, score))
        return scored

    def gini_select(self, candidates: list, top_k: int = 5) -> list:
        """用Gini增益选择最优Top-K"""
        if not candidates:
            return []
        # 按fitness排序后计算分裂增益
        sorted_cands = sorted(candidates, key=lambda x: x.get("fitness", 0), reverse=True)
        
        # 二分分裂，计算每类的Gini增益
        best_gain = -1
        best_idx = 0
        
        labels = [e.get("skill", "unknown") for e in sorted_cands]
        
        for i in range(1, len(sorted_cands)):
            left_labels = labels[:i]
            right_labels = labels[i:]
            gain = gini_gain(labels, left_labels, right_labels)
            if gain > best_gain:
                best_gain = gain
                best_idx = i
        
        # 只选择Gini增益超过阈值且排序靠前的
        if best_gain < self.gini_threshold:
            # 增益不足，选择fitness最高的
            return sorted_cands[:top_k]
        
        # 返回增益最高的分裂点左半部分（高价值）
        return sorted_cands[:min(best_idx, top_k)]

    def add(self, entries: list, timestamp: float = None) -> dict:
        """添加条目到重放池"""
        ts = timestamp or time.time()
        added = 0
        for entry in entries:
            scored_entry = {
                "entry": entry,
                "score": entry.get("fitness", 0.5),
                "gini_delta": 0.0,
                "selected_at": ts,
            }
            self.buffer.append(scored_entry)
            added += 1
        
        # 超过容量，移除最低分
        while len(self.buffer) > self.capacity:
            self.buffer.sort(key=lambda x: x["score"])
            self.buffer.pop(0)
        
        return {"added": added, "buffer_len": len(self.buffer)}

    def sample(self, n: int = 5) -> list:
        """采样n条用于重放"""
        if not self.buffer:
            return []
        # 按score降序，取top n
        sorted_buf = sorted(self.buffer, key=lambda x: x["score"], reverse=True)
        return [s["entry"] for s in sorted_buf[:n]]

    def stats(self) -> dict:
        if not self.buffer:
            return {"len": 0, "mean_score": 0}
        scores = [s["score"] for s in self.buffer]
        return {
            "len": len(self.buffer),
            "mean_score": sum(scores) / len(scores),
            "max_score": max(scores),
        }

# ============ LongTermMemory (长期记忆) ============
class LongTermMemory:
    """长期记忆，知识库/摘要库"""
    def __init__(self, path: str = None):
        self.path = path or "~/.openclaw/workspace/memory/longterm.json"
        self.path = Path(self.path).expanduser()
        self.memory = self._load()

    def _load(self) -> dict:
        try:
            with open(self.path) as f:
                return json.load(f)
        except:
            return {
                "skills": {},      # skill_name -> {rule, pattern, summary}
                "patterns": [],     # 高价值模式
                "cases": [],       # 典型案例
                "summaries": [],    # 周期性总结
            }

    def _save(self):
        self.path.parent.mkdir(parents=True, exist_ok=True)
        with open(self.path, "w") as f:
            json.dump(self.memory, f, indent=2, ensure_ascii=False)

    def promote(self, entry: dict, conditions: dict = None) -> bool:
        """提升条目到长期记忆"""
        skill = entry.get("skill", "unknown")
        conditions = conditions or {}
        
        # 稳定性条件检查
        stability = conditions.get("stability", 0.8)
        if stability < 0.7:
            return False
        
        # 更新skills
        if skill not in self.memory["skills"]:
            self.memory["skills"][skill] = {"rule": "", "pattern": [], "summary": []}
        
        skill_mem = self.memory["skills"][skill]
        
        # 提取高价值信息
        if "data" in entry:
            data = entry["data"]
            if "rule" in data:
                skill_mem["rule"] = data["rule"]
            if "pattern" in data:
                skill_mem["pattern"].append(data["pattern"])
            if "summary" in data:
                skill_mem["summary"].append(data["summary"])
        
        # 限制每个skill的pattern数量
        skill_mem["pattern"] = skill_mem["pattern"][-10:]
        skill_mem["summary"] = skill_mem["summary"][-5:]
        
        self._save()
        return True

    def query(self, skill: str = None, top_k: int = 5) -> list:
        """查询长期记忆"""
        if skill:
            return [self.memory["skills"].get(skill, {})]
        
        # 返回最近的summaries
        return self.memory["summaries"][-top_k:]

    def add_summary(self, summary: str, timestamp: float = None) -> bool:
        """添加周期性总结"""
        ts = timestamp or time.time()
        self.memory["summaries"].append({
            "summary": summary,
            "timestamp": ts,
        })
        # 限制数量
        self.memory["summaries"] = self.memory["summaries"][-50:]
        self._save()
        return True

    def stats(self) -> dict:
        return {
            "skills_count": len(self.memory["skills"]),
            "patterns_count": sum(len(s["pattern"]) for s in self.memory["skills"].values()),
            "summaries_count": len(self.memory["summaries"]),
        }

# ============ SWRMemoryManager (统一调度) ============
class SWRMemoryManager:
    """三层记忆统一调度"""
    def __init__(self, 
                 ring_capacity: int = 128,
                 replay_capacity: int = 256,
                 longterm_path: str = None,
                 swr_threshold: float = 0.7,
                 gini_threshold: float = 0.05):
        self.ring = RingBuffer(capacity=ring_capacity, threshold=swr_threshold)
        self.replay = ReplayBuffer(capacity=replay_capacity)
        self.longterm = LongTermMemory(path=longterm_path)
        self.gini_threshold = gini_threshold
        self.promote_counter = 0
        self.promote_interval = 10  # 每10次SWR触发后检查一次长期记忆

    def add(self, skill: str, fitness: float, data: dict = None, trigger: str = "manual") -> dict:
        """添加记忆条目"""
        # 1. 先写入短期记忆
        result = self.ring.push(skill, fitness, data)
        
        # 2. 检查是否触发SWR
        swr_triggered = self._should_replay(trigger, fitness)
        
        if swr_triggered:
            self._execute_swr()
        
        return {
            "ring_result": result,
            "swr_triggered": swr_triggered,
            "ring_stats": self.ring.stats(),
        }

    def _should_replay(self, trigger: str, fitness: float) -> bool:
        """判断是否触发SWR"""
        triggers = {
            "task_end": True,
            "low_confidence": fitness < 0.6,
            "high_reward": fitness > 0.9,
            "low_reward": fitness < 0.3,
            "periodic": False,  # 周期性触发由外部控制
            "manual": False,
        }
        return triggers.get(trigger, False)

    def _execute_swr(self):
        """执行SWR选择流程"""
        # 1. 从RingBuffer采样候选
        candidates = self.ring.sample_window(window_size=10)
        if not candidates:
            return
        
        # 2. 获取近期统计
        recent_stats = self.ring.stats()
        
        # 3. 计算分数
        scored = self.replay.compute_scores(candidates, recent_stats)
        
        # 4. Gini选择
        selected = self.replay.gini_select(candidates, top_k=5)
        
        # 5. 添加到重放池
        if selected:
            self.replay.add(selected)
        
        # 6. 检查是否需要提升到长期记忆
        self.promote_counter += 1
        if self.promote_counter >= self.promote_interval:
            self._check_promote_to_longterm()
            self.promote_counter = 0

    def _check_promote_to_longterm(self):
        """检查是否需要提升到长期记忆"""
        # 取重放池中fitness最高的条目
        replayed = self.replay.sample(n=3)
        for entry in replayed:
            stability = entry.get("fitness", 0.5)
            self.longterm.promote(entry, {"stability": stability})

    def query(self, skill: str = None, layers: list = None) -> dict:
        """查询记忆"""
        layers = layers or ["ring", "replay", "longterm"]
        result = {}
        
        if "ring" in layers:
            result["ring"] = {
                "recent": self.ring.sample_window(5),
                "stats": self.ring.stats(),
            }
        
        if "replay" in layers:
            result["replay"] = {
                "samples": self.replay.sample(5),
                "stats": self.replay.stats(),
            }
        
        if "longterm" in layers:
            result["longterm"] = {
                "skills": self.longterm.query(skill) if skill else [],
                "stats": self.longterm.stats(),
            }
        
        return result

    def periodic_trigger(self):
        """周期性触发SWR"""
        self._execute_swr()

    def full_stats(self) -> dict:
        return {
            "ring": self.ring.stats(),
            "replay": self.replay.stats(),
            "longterm": self.longterm.stats(),
        }

# ============ CLI ============
if __name__ == "__main__":
    args = json.load(sys.stdin)
    cmd = args.get("cmd", "add")
    
    manager_path = args.get("manager_file", "")
    
    if cmd == "add":
        manager = SWRMemoryManager() if not manager_path else None
        if manager:
            result = manager.add(
                skill=args["skill"],
                fitness=args["fitness"],
                data=args.get("data"),
                trigger=args.get("trigger", "manual"),
            )
            print(json.dumps(result, indent=2, ensure_ascii=False))
    
    elif cmd == "query":
        manager = SWRMemoryManager() if not manager_path else None
        if manager:
            result = manager.query(
                skill=args.get("skill"),
                layers=args.get("layers", ["ring", "replay", "longterm"]),
            )
            print(json.dumps(result, indent=2, ensure_ascii=False))
    
    elif cmd == "stats":
        manager = SWRMemoryManager() if not manager_path else None
        if manager:
            print(json.dumps(manager.full_stats(), indent=2, ensure_ascii=False))
    
    elif cmd == "periodic":
        manager = SWRMemoryManager() if not manager_path else None
        if manager:
            manager.periodic_trigger()
            print(json.dumps({"status": "periodic_trigger_executed"}, indent=2, ensure_ascii=False))
    
    else:
        print(json.dumps({"error": f"unknown cmd: {cmd}"}), file=sys.stderr)
        sys.exit(1)
