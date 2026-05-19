#!/usr/bin/env python3
"""
分层记忆管理器 - Mem0基因融合
基于Mem0的核心架构，实现短/长/工作记忆分层

Usage:
    python3 memory_manager.py add "memory content" --importance 0.8
    python3 memory_manager.py retrieve "query"
    python3 memory_manager.py consolidate
"""

import json
import os
import sys
import time
import hashlib
from pathlib import Path

MEMORY_DIR = Path(__file__).parent / "state" / "layered_memory"
SHORT_TERM_FILE = MEMORY_DIR / "short_term.jsonl"
LONG_TERM_FILE = MEMORY_DIR / "long_term.jsonl"
WORKING_FILE = MEMORY_DIR / "working.jsonl"

IMPORTANCE_THRESHOLD_LONG = 0.7
IMPORTANCE_THRESHOLD_SHORT = 0.4
CONSOLIDATION_INTERVAL = 10  # 每10次操作触发整合

class LayeredMemory:
    def __init__(self):
        self.operations = 0
        self._ensure_files()
    
    def _ensure_files(self):
        MEMORY_DIR.mkdir(parents=True, exist_ok=True)
        for f in [SHORT_TERM_FILE, LONG_TERM_FILE, WORKING_FILE]:
            if not f.exists():
                f.write_text("")
    
    def _compute_importance(self, content, context_score=0.5):
        """计算记忆重要性评分"""
        # 基础评分
        base = 0.5
        # 内容长度因子
        length_factor = min(len(content) / 500.0, 1.0) * 0.2
        # 上下文因子
        context_factor = context_score * 0.3
        return min(1.0, base + length_factor + context_factor)
    
    def add(self, content, importance_score=None, tags=None):
        """添加记忆"""
        if importance_score is None:
            importance_score = self._compute_importance(content)
        
        memory = {
            "id": hashlib.md5(content.encode()).hexdigest()[:12],
            "content": content,
            "importance": importance_score,
            "tags": tags or [],
            "created_at": time.time(),
            "access_count": 0,
            "last_access": time.time()
        }
        
        # 根据重要性存入不同层级
        if importance_score >= IMPORTANCE_THRESHOLD_LONG:
            self._save_to_long_term(memory)
        elif importance_score >= IMPORTANCE_THRESHOLD_SHORT:
            self._save_to_short_term(memory)
        else:
            self._save_to_working(memory)
        
        self.operations += 1
        
        # 定期整合
        if self.operations % CONSOLIDATION_INTERVAL == 0:
            self.consolidate()
        
        return memory["id"]
    
    def _save_to_short_term(self, memory):
        with open(SHORT_TERM_FILE, "a") as f:
            f.write(json.dumps(memory, ensure_ascii=False) + "\n")
    
    def _save_to_long_term(self, memory):
        with open(LONG_TERM_FILE, "a") as f:
            f.write(json.dumps(memory, ensure_ascii=False) + "\n")
    
    def _save_to_working(self, memory):
        with open(WORKING_FILE, "a") as f:
            f.write(json.dumps(memory, ensure_ascii=False) + "\n")
    
    def retrieve(self, query, top_k=5):
        """基于相关性检索记忆"""
        all_memories = []
        
        # 读取所有层
        for f in [SHORT_TERM_FILE, LONG_TERM_FILE, WORKING_FILE]:
            with open(f, "r") as fp:
                for line in fp:
                    if line.strip():
                        try:
                            m = json.loads(line)
                            # 简单的相似度计算
                            m["score"] = self._similarity(query, m["content"])
                            all_memories.append(m)
                        except:
                            pass
        
        # 更新访问计数
        for m in all_memories:
            m["access_count"] += 1
            m["last_access"] = time.time()
        
        # 返回top_k
        return sorted(all_memories, key=lambda x: x["score"], reverse=True)[:top_k]
    
    def _similarity(self, query, content):
        """简化的相似度计算"""
        query_words = set(query.lower().split())
        content_words = set(content.lower().split())
        if not query_words:
            return 0.0
        return len(query_words & content_words) / len(query_words)
    
    def consolidate(self):
        """记忆整合：将短期记忆晋升到长期"""
        print("[Memory] 执行记忆整合...")
        
        # 读取短期记忆
        short_memories = []
        with open(SHORT_TERM_FILE, "r") as f:
            for line in f:
                if line.strip():
                    short_memories.append(json.loads(line))
        
        promoted = []
        demoted = []
        
        for m in short_memories:
            # 增加访问权重
            if m["access_count"] > 3:
                promoted.append(m)
            else:
                demoted.append(m)
        
        # 晋升到长期
        if promoted:
            with open(LONG_TERM_FILE, "a") as f:
                for m in promoted:
                    f.write(json.dumps(m, ensure_ascii=False) + "\n")
            print(f"[Memory] 晋升 {len(promoted)} 条到长期记忆")
        
        # 清理短期记忆
        with open(SHORT_TERM_FILE, "w") as f:
            for m in demoted:
                f.write(json.dumps(m, ensure_ascii=False) + "\n")
        
        print(f"[Memory] 整合完成: 晋升{len(promoted)}条, 保留{len(demoted)}条")
    
    def get_memory_summary(self):
        """获取记忆摘要"""
        summary = {"short_term": 0, "long_term": 0, "working": 0}
        
        for f, key in [(SHORT_TERM_FILE, "short_term"), 
                       (LONG_TERM_FILE, "long_term"), 
                       (WORKING_FILE, "working")]:
            with open(f, "r") as fp:
                summary[key] = sum(1 for _ in fp if _.strip())
        
        return summary
    
    def get_importance_score(self):
        """计算当前记忆的重要性评分（用于Ψ_self）"""
        summary = self.get_memory_summary()
        
        # 基础分数：有记忆就给分（解决0记忆时的0分问题）
        base_score = 0.2  # 有基本记忆就给20%
        
        # 长期记忆权重最高（每个0.15）
        long_term_score = summary["long_term"] * 0.15
        
        # 短期记忆次之（每个0.08）
        short_term_score = summary["short_term"] * 0.08
        
        # 工作记忆辅助（每个0.03）
        working_score = summary["working"] * 0.03
        
        # 额外bonus：有多种记忆类型时加分
        diversity_bonus = 0.0
        if summary["long_term"] > 0:
            diversity_bonus += 0.1
        if summary["short_term"] > 0:
            diversity_bonus += 0.05
        if summary["working"] > 0:
            diversity_bonus += 0.02
        
        score = base_score + long_term_score + short_term_score + working_score + diversity_bonus
        return min(1.0, score)  # 直接归一化到0-1


if __name__ == "__main__":
    memory = LayeredMemory()
    
    if len(sys.argv) < 2:
        print("Usage: memory_manager.py [add|retrieve|consolidate|summary]")
        sys.exit(1)
    
    cmd = sys.argv[1]
    
    if cmd == "add":
        content = sys.argv[2] if len(sys.argv) > 2 else ""
        importance = float(sys.argv[3]) if len(sys.argv) > 3 else None
        memory_id = memory.add(content, importance)
        print(f"Added memory: {memory_id}")
    
    elif cmd == "retrieve":
        query = sys.argv[2] if len(sys.argv) > 2 else ""
        results = memory.retrieve(query)
        for r in results:
            print(f"[{r['score']:.2f}] {r['content'][:80]}...")
    
    elif cmd == "consolidate":
        memory.consolidate()
    
    elif cmd == "summary":
        print(json.dumps(memory.get_memory_summary(), indent=2))
    
    elif cmd == "importance":
        print(f"Importance score: {memory.get_importance_score():.3f}")
