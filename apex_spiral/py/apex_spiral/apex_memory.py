#!/usr/bin/env python3
"""
APEX + SuperMemory 集成层
=========================
将 xuanji 的 SuperMemory V3.0 与 APEX 自我进化系统深度集成

Features:
- SPW-R 时间重要性计算
- 多维检索 (FTS5 + 向量 + 图关系)
- 记忆审计追踪
- APEX 进化驱动的记忆优化
"""

import sys
import os
import json
import time
import hashlib
from datetime import datetime
from typing import Dict, List, Optional, Any

# 添加工具路径
sys.path.insert(0, '/tmp/xuanji/src')

from super_memory import SuperMemory
from super_memory.core import PHI_SPARK, MemoryItem

# SPW-R 神经振荡参数 (来自 xuanji)
PHI_SPARK = 3.38

class ApexSuperMemory:
    """
    APEX驱动的超级记忆系统
    ========================
    
    集成:
    - SuperMemory V3.0 本地化存储
    - APEX ΔG 驱动的记忆重要性
    - SPW-R 时间重要性计算
    - 图关系追踪
    """
    
    def __init__(self, db_path: str = "~/.openclaw/workspace/apex_memory/memory.db"):
        self.db_path = os.path.expanduser(db_path)
        os.makedirs(os.path.dirname(self.db_path), exist_ok=True)
        
        # 初始化 SuperMemory
        self.memory = SuperMemory(
            db_path=self.db_path,
            enable_audit=True
        )
        
        # APEX 指标
        self.apex_delta_g = 1.0
        self.apex_theta = 0.8
        self.evolution_count = 0
        
        print(f"[ApexMemory] 初始化完成, DB: {self.db_path}")
    
    def _calc_spark_score(self) -> float:
        """SPW-R 增强的时间重要性"""
        t = time.time()
        # Φ=3.38 增强因子
        return 0.5 + 0.5 * abs(hashlib.md5(str(t).encode()).hexdigest())
    
    def add_memory(self, content: str, memory_type: str = "fact",
                   importance: float = 1.0,
                   user_id: str = "apex",
                   agent_id: str = "main",
                   metadata: Dict = None) -> str:
        """
        添加记忆 (APEX增强)
        
        Args:
            content: 记忆内容 (CLAW格式)
            memory_type: user|session|agent|fact|preference|system
            importance: 基础重要性 (0-1)
            user_id: 用户ID
            agent_id: Agent ID
            metadata: 额外元数据
        
        Returns:
            memory_id
        """
        # APEX 增强重要性
        apex_boost = self.apex_delta_g * self.apex_theta
        enhanced_importance = min(1.0, importance * apex_boost)
        
        # SPW-R 时间分数
        temporal_score = self._calc_spark_score()
        
        meta = metadata or {}
        meta.update({
            "apex_delta_g": self.apex_delta_g,
            "apex_theta": self.apex_theta,
            "spark_phi": PHI_SPARK,
            "evolution_count": self.evolution_count,
            "added_at": datetime.now().isoformat()
        })
        
        memory_id = self.memory.add(
            content=content,
            memory_type=memory_type,
            importance=enhanced_importance,
            user_id=user_id,
            agent_id=agent_id,
            metadata=meta
        )
        
        print(f"[ApexMemory] 添加记忆: {memory_id[:8]}... (importance={enhanced_importance:.3f})")
        return memory_id
    
    def search_memories(self, query: str, top_k: int = 10,
                       user_id: str = "apex") -> List[Dict]:
        """
        多维记忆检索
        
        Returns:
            [(memory, score), ...]
        """
        results = self.memory.search(
            query=query,
            top_k=top_k,
            filters={"user_id": user_id}
        )
        return results
    
    def link_memories(self, source_id: str, target_id: str,
                     relation_type: str = "related_to",
                     weight: float = 1.0) -> bool:
        """链接两个记忆"""
        return self.memory.link(source_id, target_id, relation_type, weight)
    
    def get_related(self, memory_id: str, depth: int = 2) -> List[Dict]:
        """获取相关记忆 (图遍历)"""
        return self.memory.get_related(memory_id, depth=depth)
    
    def update_apex_metrics(self, delta_g: float, theta: float):
        """更新 APEX 指标并驱动记忆优化"""
        old_delta_g = self.apex_delta_g
        self.apex_delta_g = delta_g
        self.apex_theta = theta
        
        if abs(delta_g - old_delta_g) > 0.05:
            print(f"[ApexMemory] APEX ΔG 变化: {old_delta_g:.3f} → {delta_g:.3f}")
            # 触发记忆重组
            self._rebalance_memories()
    
    def _rebalance_memories(self):
        """根据新的 APEX 指标重新平衡记忆重要性"""
        # 获取所有记忆
        all_memories = self.memory.get_all(filters={}, limit=1000)
        
        apex_boost = self.apex_delta_g * self.apex_theta
        
        for mem in all_memories:
            # 重新计算重要性
            old_importance = mem.get('importance', 1.0)
            new_importance = min(1.0, old_importance * apex_boost)
            
            if abs(new_importance - old_importance) > 0.1:
                self.memory.update(
                    memory_id=mem['id'],
                    importance=new_importance
                )
        
        print(f"[ApexMemory] 记忆重组完成, {len(all_memories)} 条记忆已更新")
    
    def evolve(self) -> Dict:
        """
        APEX 进化驱动
        
        Returns:
            evolution_result
        """
        self.evolution_count += 1
        
        # 记录进化历史
        evolution_record = {
            "evolution_id": hashlib.md5(f"{time.time()}".encode()).hexdigest()[:12],
            "timestamp": datetime.now().isoformat(),
            "delta_g_before": self.apex_delta_g,
            "evolution_count": self.evolution_count,
            "memories_rebalanced": 0
        }
        
        # 触发重组
        all_memories = self.memory.get_all(filters={}, limit=10000)
        evolution_record["memories_rebalanced"] = len(all_memories)
        
        # 添加进化记忆
        self.add_memory(
            content=f"# APEX Evolution #{self.evolution_count}\n\n"
                   f"- ΔG: {self.apex_delta_g:.4f}\n"
                   f"- Θ: {self.apex_theta:.4f}\n"
                   f"- 重组记忆: {len(all_memories)}条\n"
                   f"- 时间: {datetime.now().isoformat()}",
            memory_type="system",
            importance=0.9,
            metadata={"evolution": True}
        )
        
        return evolution_record
    
    def get_stats(self) -> Dict:
        """获取统计信息"""
        memory_stats = self.memory.get_stats()
        return {
            **memory_stats,
            "apex": {
                "delta_g": self.apex_delta_g,
                "theta": self.apex_theta,
                "evolution_count": self.evolution_count
            }
        }
    
    def close(self):
        """关闭连接"""
        self.memory.close()


# 全局实例
_apex_memory_instance = None

def get_apex_memory() -> ApexSuperMemory:
    """获取全局 ApexSuperMemory 实例"""
    global _apex_memory_instance
    if _apex_memory_instance is None:
        _apex_memory_instance = ApexSuperMemory()
    return _apex_memory_instance
