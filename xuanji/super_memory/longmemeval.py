#!/usr/bin/env python3
"""
LongMemEval Components - LongMemEval基准对齐组件 V3.0
=====================================================

此模块将SuperMemory对齐至LongMemEval基准要求:

LongMemEval核心评估维度:
1. Memory Consolidation - 记忆整合算法
2. Temporal Update - 时间更新处理
3. Conflict Resolution - 冲突检测与解决
4. Retrieval Metrics - MRR/NDCG/Recall@k
5. Multi-session Continuity - 多会话连续性

对比mem0:
- mem0: 无内置评估指标, 无整合算法, 无冲突解决
- SuperMemory V3.0: 完整LongMemEval对齐

Author: 璇玑 Xuanji-58
"""

import math
import re
import json
import time
from typing import List, Dict, Optional, Tuple, Set
from datetime import datetime, timedelta
from collections import Counter, defaultdict
from dataclasses import dataclass, field


# ============== LongMemEval评估指标 ==============

class MemoryEvaluator:
    """
    LongMemEval检索质量评估器
    ==========================
    
    计算MRR, NDCG, Recall@k等LongMemEval标准指标
    
    LongMemEval评估流程:
    1. 记忆系统处理长对话序列
    2. 系统根据查询检索记忆
    3. 计算检索结果与ground-truth的匹配度
    
    指标定义:
    - MRR (Mean Reciprocal Rank): 首个相关结果排名的倒数
    - NDCG@k (Normalized Discounted Cumulative Gain): 归一化折损累积增益
    - Recall@k: 相关结果出现在top-k的比例
    """
    
    def __init__(self):
        self.queries = []
        self.results = []
        self.ground_truth = []
    
    def add_query_result(self, query: str, retrieved_ids: List[str], 
                        relevant_ids: List[str], k: int = 10):
        """
        添加查询结果用于评估
        
        Args:
            query: 查询文本
            retrieved_ids: 系统返回的记忆ID列表 (按相关性排序)
            relevant_ids: 实际相关的记忆ID列表 (ground truth)
            k: 评估深度
        """
        self.queries.append(query)
        self.results.append(retrieved_ids[:k])
        self.ground_truth.append(set(relevant_ids))
    
    def compute_mrr(self, k: int = 10) -> float:
        """
        计算Mean Reciprocal Rank
        
        MRR = (1/|Q|) * Σ(1/rank_i)
        
        其中rank_i是首个相关结果在返回列表中的位置
        
        Args:
            k: 最大评估深度
        
        Returns:
            MRR分数 (0.0 - 1.0)
        """
        if not self.queries:
            return 0.0
        
        reciprocal_ranks = []
        for retrieved in self.results:
            relevant = self.ground_truth[self.results.index(retrieved)]
            
            for i, doc_id in enumerate(retrieved[:k]):
                if doc_id in relevant:
                    reciprocal_ranks.append(1.0 / (i + 1))
                    break
            else:
                reciprocal_ranks.append(0.0)
        
        return sum(reciprocal_ranks) / len(reciprocal_ranks)
    
    def compute_ndcg(self, k: int = 10) -> float:
        """
        计算Normalized Discounted Cumulative Gain
        
        DCG@k = Σ(i=1 to k) rel_i / log2(i+1)
        NDCG@k = DCG@k / IDCG@k
        
        其中rel_i是第i个结果的相关性分数
        
        Args:
            k: 最大评估深度
        
        Returns:
            NDCG分数 (0.0 - 1.0)
        """
        if not self.queries:
            return 0.0
        
        ndcg_scores = []
        
        for retrieved, relevant_set in zip(self.results, self.ground_truth):
            # 计算DCG
            dcg = 0.0
            for i, doc_id in enumerate(retrieved[:k]):
                # 相关性: 2^(rel) - 1, rel=1 if relevant else 0
                rel = 1.0 if doc_id in relevant_set else 0.0
                dcg += rel / math.log2(i + 2)  # i+2因为log2(1)=0
            
            # 计算IDCG (理想DCG)
            ideal_retrieved = list(relevant_set)[:k]
            idcg = 0.0
            for i, doc_id in enumerate(ideal_retrieved):
                rel = 1.0
                idcg += rel / math.log2(i + 2)
            
            # 归一化
            if idcg > 0:
                ndcg_scores.append(dcg / idcg)
            else:
                ndcg_scores.append(0.0)
        
        return sum(ndcg_scores) / len(ndcg_scores)
    
    def compute_recall(self, k: int = 10) -> float:
        """
        计算Recall@k
        
        Recall@k = |relevant ∩ retrieved_top_k| / |relevant|
        
        Args:
            k: 评估深度
        
        Returns:
            Recall分数 (0.0 - 1.0)
        """
        if not self.queries:
            return 0.0
        
        recall_scores = []
        for retrieved, relevant_set in zip(self.results, self.ground_truth):
            if not relevant_set:
                recall_scores.append(0.0)
                continue
            
            retrieved_set = set(retrieved[:k])
            intersection = retrieved_set & relevant_set
            recall = len(intersection) / len(relevant_set)
            recall_scores.append(recall)
        
        return sum(recall_scores) / len(recall_scores)
    
    def compute_precision(self, k: int = 10) -> float:
        """
        计算Precision@k
        
        Precision@k = |relevant ∩ retrieved_top_k| / k
        
        Args:
            k: 评估深度
        
        Returns:
            Precision分数 (0.0 - 1.0)
        """
        if not self.queries:
            return 0.0
        
        precision_scores = []
        for retrieved, relevant_set in zip(self.results, self.ground_truth):
            retrieved_set = set(retrieved[:k])
            intersection = retrieved_set & relevant_set
            precision = len(intersection) / k
            precision_scores.append(precision)
        
        return sum(precision_scores) / len(precision_scores)
    
    def compute_all_metrics(self, k: int = 10) -> Dict:
        """
        计算所有LongMemEval指标
        
        Returns:
            {mrr, ndcg, recall@k, precision@k}
        """
        return {
            'mrr': round(self.compute_mrr(k), 4),
            'ndcg': round(self.compute_ndcg(k), 4),
            'recall@{}'.format(k): round(self.compute_recall(k), 4),
            'precision@{}'.format(k): round(self.compute_precision(k), 4),
            'num_queries': len(self.queries)
        }
    
    def reset(self):
        """重置评估器"""
        self.queries = []
        self.results = []
        self.ground_truth = []


# ============== 记忆整合 (Memory Consolidation) ==============

class MemoryConsolidator:
    """
    记忆整合器 V3.0
    ===============
    
    LongMemEval要求记忆系统具备整合能力:
    - 根据重要性/访问频率保留关键记忆
    - 根据时间衰减淘汰低价值记忆
    - 合并相似记忆减少冗余
    
    整合策略:
    1. Importance-based retention: importance * access_count * temporal_score
    2. Decay-based eviction: 定期淘汰decay_factor低的记忆
    3. Semantic merge: 合并语义相似的事实
    """
    
    # 整合阈值
    MIN_IMPORTANCE = 0.2      # 最低重要性阈值
    MIN_ACCESS_COUNT = 3     # 最低访问次数
    DECAY_RATE = 0.95         # 时间衰减率
    CONSOLIDATION_INTERVAL = 86400  # 整合间隔 (24小时)
    
    def __init__(self, graph, decay_rate: float = 0.95):
        self.graph = graph
        self.decay_rate = decay_rate
        self._last_consolidation = 0
    
    def should_consolidate(self) -> bool:
        """检查是否需要整合"""
        return (time.time() - self._last_consolidation) > self.CONSOLIDATION_INTERVAL
    
    def calculate_composite_score(self, memory_item: Dict) -> float:
        """
        计算记忆综合分数
        
        Score = importance * (1 + log(access_count + 1)) * temporal_score * decay_factor
        
        LongMemEval评估:
        - 高分数记忆应该被保留
        - 低分数记忆应该被整合(合并或删除)
        """
        importance = memory_item.get('importance', 1.0)
        access_count = memory_item.get('access_count', 0)
        temporal_score = memory_item.get('temporal_score', 0.5)
        decay_factor = memory_item.get('decay_factor', 1.0)
        
        # 访问次数对数加成 (边际递减)
        access_bonus = 1.0 + math.log(access_count + 1)
        
        score = importance * access_bonus * temporal_score * decay_factor
        return score
    
    def apply_decay(self, memory_item: Dict, days_elapsed: float) -> Dict:
        """
        应用时间衰减
        
        decay_factor = decay_rate ^ (days_elapsed)
        
        Args:
            memory_item: 记忆条目
            days_elapsed: 自上次更新以来的天数
        
        Returns:
            更新后的记忆(带新的decay_factor)
        """
        new_decay = math.pow(self.decay_rate, days_elapsed)
        memory_item['decay_factor'] = new_decay
        return memory_item
    
    def find_mergeable_pairs(self, memories: List[Dict], 
                           similarity_threshold: float = 0.85) -> List[Tuple[str, str]]:
        """
        查找可合并的记忆对
        
        合并条件:
        1. 语义相似度 > threshold
        2. 同一user_id/agent_id
        3. 时间间隔 < 7天
        
        Args:
            memories: 记忆列表
            similarity_threshold: 相似度阈值
        
        Returns:
            [(memory_id_a, memory_id_b), ...]
        """
        mergeable = []
        memories_by_entity = defaultdict(list)
        
        # 按实体分组
        for m in memories:
            key = (m.get('user_id'), m.get('agent_id'))
            memories_by_entity[key].append(m)
        
        for key, entity_memories in memories_by_entity.items():
            for i, m1 in enumerate(entity_memories):
                for m2 in entity_memories[i+1:]:
                    # 时间检查 (7天内)
                    try:
                        t1 = datetime.fromisoformat(m1.get('created_at', ''))
                        t2 = datetime.fromisoformat(m2.get('created_at', ''))
                        days_diff = abs((t1 - t2).days)
                        if days_diff > 7:
                            continue
                    except:
                        pass
                    
                    # 相似度检查 (简单基于共同字符)
                    sim = self._compute_similarity(m1.get('content', ''), 
                                                   m2.get('content', ''))
                    if sim >= similarity_threshold:
                        mergeable.append((m1['id'], m2['id']))
        
        return mergeable
    
    def _compute_similarity(self, text1: str, text2: str) -> float:
        """
        计算简单文本相似度
        
        使用Jaccard系数 (词级别)
        """
        words1 = set(re.findall(r'\b\w+\b', text1.lower()))
        words2 = set(re.findall(r'\b\w+\b', text2.lower()))
        
        if not words1 or not words2:
            return 0.0
        
        intersection = words1 & words2
        union = words1 | words2
        
        return len(intersection) / len(union)
    
    def consolidate(self, user_id: str = None, agent_id: str = None,
                    dry_run: bool = True) -> Dict:
        """
        执行记忆整合
        
        Args:
            user_id: 用户ID (过滤)
            agent_id: Agent ID (过滤)
            dry_run: True=仅分析不执行, False=执行整合
        
        Returns:
            {candidates, to_merge, to_delete, scores}
        """
        # 获取所有记忆
        filters = {}
        if user_id:
            filters['user_id'] = user_id
        if agent_id:
            filters['agent_id'] = agent_id
        
        memories = self.graph.get_all_nodes(filters, limit=10000)
        
        # 计算分数
        candidates = []
        scores = {}
        for m in memories:
            m_dict = m.to_dict() if hasattr(m, 'to_dict') else dict(m)
            score = self.calculate_composite_score(m_dict)
            m_dict['consolidation_score'] = score
            scores[m_dict['id']] = score
            candidates.append(m_dict)
        
        # 找出需要删除的低分记忆
        to_delete = [c['id'] for c in candidates 
                     if c['consolidation_score'] < self.MIN_IMPORTANCE 
                     and c.get('access_count', 0) < self.MIN_ACCESS_COUNT]
        
        # 找出可合并的记忆对
        to_merge = self.find_mergeable_pairs(memories)
        
        if not dry_run:
            # 执行删除
            for mid in to_delete:
                self.graph.soft_delete(mid)
            
            # 执行合并 (保留分数高的,更新分数低的)
            for id_a, id_b in to_merge:
                score_a = scores.get(id_a, 0)
                score_b = scores.get(id_b, 0)
                winner = id_a if score_a >= score_b else id_b
                loser = id_b if score_a >= score_b else id_a
                
                # loser内容合并到winner
                winner_node = self.graph.get_node(winner)
                loser_node = self.graph.get_node(loser)
                if winner_node and loser_node:
                    merged_content = winner_node.content + "\n---\n" + loser_node.content
                    self.graph.update_node(winner, content=merged_content)
                    self.graph.soft_delete(loser)
        
        self._last_consolidation = time.time()
        
        return {
            'candidates': len(candidates),
            'to_merge': len(to_merge),
            'to_delete': len(to_delete),
            'avg_score': sum(scores.values()) / len(scores) if scores else 0,
            'scores': scores,
            'dry_run': dry_run
        }


# ============== 冲突检测与解决 ==============

class ConflictResolver:
    """
    冲突解决器 V3.0
    ==============
    
    LongMemEval关键要求: 检测并解决冲突记忆
    
    冲突类型:
    1. 事实冲突: "X是Y" vs "X是Z"
    2. 偏好冲突: "喜欢A" vs "喜欢B"
    3. 时间冲突: "2024年在NYC" vs "2025年在LA"
    
    解决策略:
    1. Temporal precedence: 新时间戳优先
    2. Importance-based: 高重要性优先
    3. Source credibility: 可信来源优先
    """
    
    # 冲突模式正则
    CONFLICT_PATTERNS = [
        (r'(\w+)\s+(?:lives?|resides?)\s+in\s+([A-Za-z\s]+)', 'location'),  # 修复: 捕获多词地名
        (r'(\w+)\s+(?:works?|employed)\s+(?:at|as)\s+([A-Za-z\s]+)', 'work'),  # 修复: 捕获多词职位
        (r'(\w+)\s+is\s+(?:a|an)\s+(\w+)', 'identity'),
        (r'(\w+)\s+(?:likes?|prefers?)\s+(\w+)', 'preference'),
        (r'(\w+)\s+(?:hates?|dislikes?)\s+(\w+)', 'preference_negative'),
    ]
    
    def __init__(self, graph):
        self.graph = graph
    
    def detect_conflicts(self, user_id: str = None) -> List[Dict]:
        """
        检测冲突记忆
        
        Args:
            user_id: 用户ID (过滤)
        
        Returns:
            [{conflict_type, memory_a, memory_b, resolution}, ...]
        """
        filters = {}
        if user_id:
            filters['user_id'] = user_id
        
        memories = self.graph.get_all_nodes(filters, limit=10000)
        
        # 按实体提取事实
        facts_by_entity = defaultdict(list)
        for m in memories:
            m_dict = m.to_dict() if hasattr(m, 'to_dict') else dict(m)
            content = m_dict.get('content', '')
            
            for pattern, fact_type in self.CONFLICT_PATTERNS:
                match = re.search(pattern, content, re.IGNORECASE)
                if match:
                    entity = match.group(1)
                    value = match.group(2)
                    facts_by_entity[(entity, fact_type)].append({
                        'memory_id': m_dict['id'],
                        'content': content,
                        'value': value,
                        'created_at': m_dict.get('created_at'),
                        'importance': m_dict.get('importance', 1.0),
                        'user_id': m_dict.get('user_id'),
                    })
        
        # 检测同一实体-类型的多重事实
        conflicts = []
        for (entity, fact_type), facts in facts_by_entity.items():
            if len(facts) > 1:
                # 检查是否有冲突值
                unique_values = set(f['value'].lower() for f in facts)
                if len(unique_values) > 1:
                    # 按时间排序，取最新的作为winner
                    sorted_facts = sorted(facts, 
                                        key=lambda x: x.get('created_at', ''),
                                        reverse=True)
                    
                    conflicts.append({
                        'entity': entity,
                        'conflict_type': fact_type,
                        'conflicting_values': list(unique_values),
                        'memories': [f['memory_id'] for f in facts],
                        'resolution': sorted_facts[0]['memory_id'],  # 最新事实获胜
                        'resolved_value': sorted_facts[0]['value'],
                    })
        
        return conflicts
    
    def resolve_conflict(self, conflict: Dict, strategy: str = 'temporal') -> str:
        """
        解决冲突
        
        Args:
            conflict: 冲突描述
            strategy: 解决策略 (temporal | importance | manual)
        
        Returns:
            获胜的记忆ID
        """
        if strategy == 'temporal':
            # 时间策略: 最新优先
            memories = conflict.get('memories', [])
            if not memories:
                return None
            
            # 获取每个记忆的时间戳
            memory_times = []
            for mid in memories:
                node = self.graph.get_node(mid)
                if node:
                    memory_times.append((mid, node.created_at))
            
            if memory_times:
                return sorted(memory_times, key=lambda x: x[1], reverse=True)[0][0]
            return memories[0]
        
        elif strategy == 'importance':
            # 重要性策略: 高重要性优先
            memories = conflict.get('memories', [])
            if not memories:
                return None
            
            memory_importance = []
            for mid in memories:
                node = self.graph.get_node(mid)
                if node:
                    memory_importance.append((mid, node.importance))
            
            if memory_importance:
                return sorted(memory_importance, key=lambda x: x[1], reverse=True)[0][0]
            return memories[0]
        
        else:
            return conflict.get('resolution')
    
    def auto_resolve(self, user_id: str = None) -> Dict:
        """
        自动解决所有冲突
        
        Args:
            user_id: 用户ID (过滤)
        
        Returns:
            {conflicts_found, conflicts_resolved, resolutions}
        """
        conflicts = self.detect_conflicts(user_id)
        resolutions = []
        
        for conflict in conflicts:
            winner_id = self.resolve_conflict(conflict, strategy='temporal')
            if winner_id:
                resolutions.append({
                    'conflict': conflict,
                    'winner_id': winner_id
                })
                
                # 标记失败者 (软删除)
                for mid in conflict['memories']:
                    if mid != winner_id:
                        self.graph.soft_delete(mid)
        
        return {
            'conflicts_found': len(conflicts),
            'conflicts_resolved': len(resolutions),
            'resolutions': resolutions
        }


# ============== 时间更新处理器 ==============

class TemporalUpdater:
    """
    时间更新处理器 V3.0
    ===================
    
    LongMemEval要求: 新信息自动覆盖旧信息
    
    规则:
    1. 同实体的事实更新时，自动标记旧记忆为过时
    2. 时间戳更近的记忆优先级更高
    3. 过时记忆保留用于冲突检测但不用于检索
    """
    
    def __init__(self, graph):
        self.graph = graph
    
    def process_update(self, memory_id: str, new_content: str,
                      user_id: str = None) -> Dict:
        """
        处理记忆更新
        
        检测相关旧记忆并标记为过时
        
        Args:
            memory_id: 新记忆ID
            new_content: 新内容
            user_id: 用户ID
        
        Returns:
            {updated, marked_outdated, old_memory_ids}
        """
        node = self.graph.get_node(memory_id, user_id)
        if not node:
            return {'updated': False}
        
        # 查找相关的旧记忆
        related = self._find_related_old_memories(node, user_id)
        
        marked_outdated = []
        for old_id in related:
            # 添加过时标记
            old_node = self.graph.get_node(old_id, user_id)
            if old_node:
                new_metadata = old_node.metadata or {}
                new_metadata['outdated'] = True
                new_metadata['superseded_by'] = memory_id
                new_metadata['superseded_at'] = datetime.now().isoformat()
                self.graph.update_node(old_id, metadata=new_metadata)
                marked_outdated.append(old_id)
        
        return {
            'updated': True,
            'marked_outdated': len(marked_outdated),
            'old_memory_ids': marked_outdated
        }
    
    def _find_related_old_memories(self, reference_node, 
                                   user_id: str = None) -> List[str]:
        """查找相关旧记忆"""
        related_ids = []
        
        # 1. 相同实体的记忆
        if reference_node.entity_type:
            neighbors = self.graph.get_neighbors(reference_node.id, depth=1, user_id=user_id)
            related_ids.extend(neighbors)
        
        # 2. 图关系中的记忆
        relation_results = self.graph.search_by_relation(
            reference_node.id, 'related_to', user_id
        )
        related_ids.extend(relation_results)
        
        # 3. 去除自身
        related_ids = [rid for rid in related_ids if rid != reference_node.id]
        
        return list(set(related_ids))
    
    def get_active_memories(self, user_id: str = None) -> List[Dict]:
        """
        获取活跃记忆(未过时的)
        
        Args:
            user_id: 用户ID (过滤)
        
        Returns:
            活跃记忆列表
        """
        filters = {}
        if user_id:
            filters['user_id'] = user_id
        
        all_memories = self.graph.get_all_nodes(filters, limit=10000)
        
        active = []
        for m in all_memories:
            m_dict = m.to_dict() if hasattr(m, 'to_dict') else dict(m)
            metadata = m_dict.get('metadata', {})

            # Handle metadata being a JSON string
            if isinstance(metadata, str):
                try:
                    metadata = json.loads(metadata)
                except (json.JSONDecodeError, TypeError):
                    metadata = {}

            # Skip outdated memories
            if metadata.get('outdated'):
                continue

            active.append(m_dict)

        return active
    
    def get_outdated_memories(self, user_id: str = None) -> List[Dict]:
        """
        获取过时记忆(用于冲突检测参考)
        
        Args:
            user_id: 用户ID (过滤)
        
        Returns:
            过时记忆列表
        """
        filters = {}
        if user_id:
            filters['user_id'] = user_id
        
        all_memories = self.graph.get_all_nodes(filters, limit=10000)
        
        outdated = []
        for m in all_memories:
            m_dict = m.to_dict() if hasattr(m, 'to_dict') else dict(m)
            metadata = m_dict.get('metadata', {})

            # Handle metadata being a JSON string
            if isinstance(metadata, str):
                try:
                    metadata = json.loads(metadata)
                except (json.JSONDecodeError, TypeError):
                    metadata = {}

            if metadata.get('outdated'):
                outdated.append(m_dict)

        return outdated