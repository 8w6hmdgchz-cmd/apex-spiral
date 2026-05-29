"""
APEX Memory Stream Module - 基于论文 arXiv:2304.03442
实现时序记忆流 + 定期高层反思
"""

import time
from typing import Optional, Callable, List, Any, Dict
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum


class MemoryType(Enum):
    """记忆类型"""
    OBSERVATION = "observation"      # 观察
    REFLECTION = "reflection"        # 反思
    PLAN = "plan"                   # 规划
    EXECUTION = "execution"         # 执行
    INSIGHT = "insight"            # 高层见解


@dataclass
class Memory:
    """单条记忆"""
    id: str
    type: MemoryType
    content: str
    timestamp: str
    importance: float = 0.5        # 0-1, 重要性
    embedding: Optional[List[float]] = None  # 向量嵌入
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def age(self) -> float:
        """计算记忆年龄（小时）"""
        try:
            t = datetime.fromisoformat(self.timestamp)
            delta = datetime.now() - t
            return delta.total_seconds() / 3600
        except:
            return 0.0


@dataclass
class MemoryStreamConfig:
    """Memory Stream 配置"""
    max_size: int = 1000
    reflection_threshold: int = 20  # 积累多少条记忆后触发反思
    importance_boost_on_use: float = 0.05  # 每次使用时提升重要性


class ApexMemoryStream:
    """
    APEX 记忆流 - 模仿 Generative Agents 的 Memory Stream
    
    核心机制：
    1. 自然语言存储所有经验
    2. 基于 相关性 + 时效性 + 重要性 检索
    3. 定期合成高层见解
    """
    
    def __init__(
        self,
        embed_func: Optional[Callable[[str], List[float]]] = None,
        llm_func: Optional[Callable[[str], str]] = None,
        save_func: Optional[Callable[[str, str], None]] = None,
        load_func: Optional[Callable[[str], str]] = None,
        config: Optional[MemoryStreamConfig] = None
    ):
        """
        Args:
            embed_func: 向量化函数 (text) -> List[float]
            llm_func: LLM 函数，用于合成反思
            save_func: 持久化存储函数
            load_func: 持久化加载函数
            config: 配置
        """
        self.embed = embed_func
        self.llm = llm_func
        self.save = save_func
        self.load = load_func
        self.config = config or MemoryStreamConfig()
        
        self.memories: List[Memory] = []
        self.insights: List[Memory] = []  # 高层见解
    
    def add(
        self,
        content: str,
        memory_type: MemoryType = MemoryType.OBSERVATION,
        importance: float = 0.5,
        metadata: Optional[Dict[str, Any]] = None
    ) -> str:
        """
        添加记忆
        
        Returns:
            memory_id: 生成的记忆 ID
        """
        memory_id = f"mem_{datetime.now().timestamp()}"
        
        # 如果有 embed 函数，生成向量
        embedding = self.embed(content) if self.embed else None
        
        memory = Memory(
            id=memory_id,
            type=memory_type,
            content=content,
            timestamp=datetime.now().isoformat(),
            importance=importance,
            embedding=embedding,
            metadata=metadata or {}
        )
        
        self.memories.append(memory)
        
        # 持久化
        if self.save:
            self.save(memory_id, self._serialize(memory))
        
        # 检查是否需要触发反思
        self._maybe_synthesize_reflection()
        
        # 限制大小
        if len(self.memories) > self.config.max_size:
            self._prune_old_memories()
        
        return memory_id
    
    def retrieve(
        self,
        query: str,
        n: int = 5,
        memory_types: Optional[List[MemoryType]] = None
    ) -> List[Memory]:
        """
        检索记忆
        
        基于 相关性 + 时效性 + 重要性 综合评分
        
        Args:
            query: 查询文本
            n: 返回数量
            memory_types: 过滤的记忆类型
        """
        if not self.memories:
            return []
        
        candidates = self.memories
        if memory_types:
            candidates = [m for m in candidates if m.type in memory_types]
        
        if self.embed:
            # 向量检索
            query_embedding = self.embed(query)
            scored = []
            for mem in candidates:
                relevance = self._cosine_sim(query_embedding, mem.embedding)
                recency = 1.0 / (1.0 + mem.age() * 0.1)  # 越老越低
                importance = mem.importance
                
                # 综合评分
                score = relevance * 0.5 + recency * 0.2 + importance * 0.3
                scored.append((score, mem))
            
            scored.sort(reverse=True)
            return [m for _, m in scored[:n]]
        else:
            # 文本检索（简单匹配）
            query_lower = query.lower()
            scored = []
            for mem in candidates:
                relevance = query_lower in mem.content.lower()
                recency = 1.0 / (1.0 + mem.age() * 0.1)
                score = (1.0 if relevance else 0.0) * 0.6 + recency * 0.4 + mem.importance * 0.2
                scored.append((score, mem))
            
            scored.sort(reverse=True)
            return [m for _, m in scored[:n]]
    
    def _cosine_sim(self, a: List[float], b: List[float]) -> float:
        """计算余弦相似度"""
        if not a or not b or len(a) != len(b):
            return 0.0
        
        dot = sum(x * y for x, y in zip(a, b))
        norm_a = sum(x * x for x in a) ** 0.5
        norm_b = sum(x * x for x in b) ** 0.5
        
        if norm_a == 0 or norm_b == 0:
            return 0.0
        
        return dot / (norm_a * norm_b)
    
    def _maybe_synthesize_reflection(self) -> None:
        """检查是否需要合成高层反思"""
        if not self.llm:
            return
        
        # 积累足够多记忆后触发
        observation_count = sum(
            1 for m in self.memories 
            if m.type == MemoryType.OBSERVATION
        )
        
        if observation_count >= self.config.reflection_threshold:
            self.synthesize_insight()
    
    def synthesize_insight(self) -> Optional[Memory]:
        """
        合成高层见解 - Generative Agents 的 Reflection 机制
        
        将最近的观察合成高层洞察
        """
        if not self.llm:
            return None
        
        # 获取最近的观察
        recent = [
            m for m in self.memories[-50:]
            if m.type == MemoryType.OBSERVATION
        ]
        
        if len(recent) < 5:
            return None
        
        # 构建 prompt
        content_list = "\n".join([
            f"- {m.content} ({m.timestamp})"
            for m in recent
        ])
        
        prompt = f"""这些是最近的观察记录：

{content_list}

请分析这些观察，合成3-5个高层次的见解。
每个见解应该：
1. 超越具体事件，看到模式
2. 有洞察力，能指导未来行动
3. 用简洁的语言描述

格式：
[洞察1] ...
[洞察2] ...
..."""
        
        insight_text = self.llm(prompt)
        
        insight = Memory(
            id=f"insight_{datetime.now().timestamp()}",
            type=MemoryType.INSIGHT,
            content=insight_text,
            timestamp=datetime.now().isoformat(),
            importance=0.8,  # 高重要性
            metadata={"source_count": len(recent)}
        )
        
        self.insights.append(insight)
        self.memories.append(insight)
        
        if self.save:
            self.save(insight.id, self._serialize(insight))
        
        return insight
    
    def get_insights(self, n: int = 5) -> List[Memory]:
        """获取高层见解"""
        return self.insights[-n:]
    
    def boost_importance(self, memory_id: str) -> None:
        """提升记忆重要性"""
        for mem in self.memories:
            if mem.id == memory_id:
                mem.importance = min(1.0, mem.importance + self.config.importance_boost_on_use)
                break
    
    def _prune_old_memories(self) -> None:
        """删除最老的记忆"""
        # 保留 insights 和重要记忆
        to_keep = []
        for mem in self.memories:
            if mem.type == MemoryType.INSIGHT or mem.importance > 0.7:
                to_keep.append(mem)
        
        # 如果太多，保留最近的
        if len(to_keep) > self.config.max_size * 0.5:
            to_keep = to_keep[-int(self.config.max_size * 0.5):]
        
        self.memories = to_keep
    
    def _serialize(self, memory: Memory) -> str:
        """序列化记忆"""
        import json
        return json.dumps({
            "id": memory.id,
            "type": memory.type.value,
            "content": memory.content,
            "timestamp": memory.timestamp,
            "importance": memory.importance,
            "metadata": memory.metadata
        })
    
    def summary(self) -> dict:
        """返回记忆流摘要"""
        type_counts = {}
        for mem_type in MemoryType:
            type_counts[mem_type.value] = sum(
                1 for m in self.memories if m.type == mem_type
            )
        
        return {
            "total_memories": len(self.memories),
            "insights": len(self.insights),
            "type_counts": type_counts,
            "recent": [
                {"type": m.type.value, "content": m.content[:50]}
                for m in self.memories[-3:]
            ]
        }
