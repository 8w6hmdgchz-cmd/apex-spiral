#!/usr/bin/env python3
"""
MultiSignal Retriever - 多维检索引擎 V2.0
=========================================
安全 · 精准 · 高性能

功能对比mem0:
- mem0: 语义 + BM25 + 实体增强
- SuperMemory: 语义 + BM25 + 图关系 + 时间 × SPW-R

安全增强:
- Entity scoping (user_id/agent_id/run_id)
- SQL injection prevention
- Input sanitization

Author: 璇玑 Xuanji-58
"""

import math
import re
import json
import sqlite3
from typing import List, Dict, Optional, Tuple, Any
from datetime import datetime, timedelta
from collections import Counter, defaultdict
import hashlib

# SPW-R参数
PHI_SPARK = 3.38

# 安全验证器
SECRET_FIELDS = {
    'api_key', 'secret_key', 'password', 'token', 'credentials',
    'secret', 'passwd', 'pwd', 'private_key', 'access_token'
}


class SecurityValidator:
    """安全验证器"""
    
    @staticmethod
    def sanitize_query(query: str) -> str:
        """清理查询"""
        if not query or not isinstance(query, str):
            return ""
        
        # 限制长度
        if len(query) > 10000:
            query = query[:10000]
        
        return query.strip()
    
    @staticmethod
    def validate_filters(filters: Dict) -> Dict:
        """验证过滤条件"""
        validated = {}
        
        for key in ['user_id', 'agent_id', 'run_id']:
            if key in filters and filters[key]:
                value = str(filters[key]).strip()
                if value and len(value) <= 256:
                    validated[key] = value
        
        if 'memory_type' in filters:
            mt = str(filters['memory_type'])
            if mt in {'user', 'session', 'agent', 'fact', 'preference', 'system'}:
                validated['memory_type'] = mt
        
        return validated
    
    @staticmethod
    def check_sql_injection(text: str) -> bool:
        """检测SQL注入"""
        if not text:
            return False
        dangerous = [
            r'(\bUNION\b|\bSELECT\b|\bINSERT\b|\bUPDATE\b|\bDELETE\b|\bDROP\b)',
            r'[\'\"\;]\s*(OR|AND)\s+',
            r'\-\-',
            r'/\*.*\*/',
        ]
        text_upper = text.upper()
        for pattern in dangerous:
            if re.search(pattern, text_upper):
                return True
        return False


class BM25:
    """
    BM25关键词检索算法
    ==================
    
    对比mem0:
    - mem0使用fastembed/bm25 via Qdrant
    - SuperMemory使用纯Python实现，无需外部依赖
    
    参数:
    - k1: 词频饱和参数 (default 1.5)
    - b: 文档长度归一化 (default 0.75)
    """
    
    def __init__(self, k1: float = 1.5, b: float = 0.75):
        self.k1 = k1
        self.b = b
        self.doc_freqs = {}
        self.doc_lengths = []
        self.avgdl = 0
        self.N = 0
        self.doc_ids = []
        self.doc_contents = []
    
    def index(self, documents: List[Tuple[str, str]]):
        """
        建立索引: [(doc_id, content), ...]
        
        对比mem0:
        - mem0: 在Qdrant中建立BM25索引
        - SuperMemory: 纯本地SQLite + Python实现
        """
        self.N = len(documents)
        self.doc_freqs = {}
        total_len = 0
        
        self.doc_ids = []
        self.doc_contents = []
        
        for doc_id, content in documents:
            self.doc_ids.append(doc_id)
            self.doc_contents.append(content)
            
            words = self._tokenize(content)
            self.doc_lengths.append(len(words))
            total_len += len(words)
            
            unique_words = set(words)
            for word in unique_words:
                self.doc_freqs[word] = self.doc_freqs.get(word, 0) + 1
        
        self.avgdl = total_len / self.N if self.N > 0 else 1
    
    def _tokenize(self, text: str) -> List[str]:
        """分词"""
        text = text.lower()
        words = re.findall(r'\b\w+\b', text)
        return words
    
    def _calc_idf(self, word: str) -> float:
        """计算IDF"""
        df = self.doc_freqs.get(word, 0)
        if df == 0:
            return 0.0
        return math.log((self.N - df + 0.5) / (df + 0.5) + 1)
    
    def score(self, query: str, doc_index: int) -> float:
        """计算单个文档的BM25分数"""
        words = self._tokenize(query)
        doc_words = self._tokenize(self.doc_contents[doc_index])
        doc_len = self.doc_lengths[doc_index]
        doc_word_counts = Counter(doc_words)
        
        score = 0.0
        for word in words:
            if word not in self.doc_freqs:
                continue
            
            idf = self._calc_idf(word)
            tf = doc_word_counts.get(word, 0)
            
            numerator = tf * (self.k1 + 1)
            denominator = tf + self.k1 * (1 - self.b + self.b * doc_len / self.avgdl)
            
            score += idf * numerator / denominator
        
        return score
    
    def search(self, query: str, top_k: int = 10) -> List[Tuple[str, float]]:
        """搜索top_k结果"""
        if self.N == 0:
            return []
        
        scores = []
        for i in range(self.N):
            s = self.score(query, i)
            if s > 0:
                scores.append((self.doc_ids[i], s))
        
        scores.sort(key=lambda x: x[1], reverse=True)
        return scores[:top_k]


class EmbeddingEncoder:
    """
    Embedding编码器
    ==============
    
    支持多种编码方式:
    1. TF-IDF (无需外部依赖)
    2. 词袋模型
    3. 自定义向量输入
    
    对比mem0:
    - mem0: 支持13种embedding provider (OpenAI, Ollama, etc.)
    - SuperMemory: 本地TF-IDF实现，零外部依赖
    """
    
    def __init__(self, dimension: int = 384):
        self.dimension = dimension
        self.vocabulary = {}
        self.idf = {}
        self.doc_count = 0
    
    def fit(self, documents: List[str]):
        """训练TF-IDF模型"""
        self.doc_count = 0
        self.vocabulary = {}
        df = defaultdict(int)
        
        all_tokens = []
        
        for doc in documents:
            tokens = self._tokenize(doc)
            all_tokens.append(tokens)
            
            unique_tokens = set(tokens)
            for token in unique_tokens:
                df[token] += 1
            
            self.doc_count += 1
        
        # 构建词典
        for token, freq in df.items():
            if freq >= 2:  # 至少出现2次
                self.vocabulary[token] = len(self.vocabulary)
        
        # 计算IDF
        for token, freq in df.items():
            self.idf[token] = math.log(self.doc_count / (freq + 1)) + 1
    
    def _tokenize(self, text: str) -> List[str]:
        """分词"""
        text = text.lower()
        words = re.findall(r'\b\w+\b', text)
        return words
    
    def encode(self, text: str) -> List[float]:
        """编码为向量"""
        tokens = self._tokenize(text)
        
        vector = [0.0] * min(len(self.vocabulary), self.dimension)
        
        token_counts = Counter(tokens)
        total_tokens = len(tokens) if tokens else 1
        
        for token, count in token_counts.items():
            if token in self.vocabulary:
                idx = self.vocabulary[token]
                if idx < self.dimension:
                    tf = count / total_tokens
                    idf = self.idf.get(token, 1.0)
                    vector[idx] = tf * idf
        
        # 归一化
        magnitude = math.sqrt(sum(v * v for v in vector))
        if magnitude > 0:
            vector = [v / magnitude for v in vector]
        
        return vector
    
    def cosine_similarity(self, vec1: List[float], vec2: List[float]) -> float:
        """余弦相似度"""
        dot = sum(a * b for a, b in zip(vec1, vec2))
        mag1 = math.sqrt(sum(a * a for a in vec1))
        mag2 = math.sqrt(sum(b * b for b in vec2))
        
        if mag1 == 0 or mag2 == 0:
            return 0.0
        return dot / (mag1 * mag2)


class MultiSignalRetriever:
    """
    多维检索器 V2.0
    ===============
    
    融合四维信号:
    1. 语义相似度 (TF-IDF embedding)
    2. BM25关键词匹配
    3. 图关系增强
    4. 时间重要性 (SPW-R)
    
    对比mem0:
    - mem0 v1: 仅语义
    - mem0 v2: 语义 + BM25 + 实体 + 可选rerank
    - SuperMemory: 语义 + BM25 + 图关系 + 时间 × Φ_SPARK
    
    安全增强:
    - Entity scoping
    - SQL injection prevention
    - Input sanitization
    """
    
    def __init__(self, graph,
                 semantic_weight: float = 0.35,
                 bm25_weight: float = 0.30,
                 graph_weight: float = 0.20,
                 temporal_weight: float = 0.15,
                 embedding_dim: int = 384):
        self.graph = graph
        self.semantic_weight = semantic_weight
        self.bm25_weight = bm25_weight
        self.graph_weight = graph_weight
        self.temporal_weight = temporal_weight
        self.embedding_dim = embedding_dim
        
        self.bm25 = BM25()
        self.embedding_encoder = EmbeddingEncoder(dimension=embedding_dim)
        self._indexed = False
        self._lock = threading.Lock()
    
    def _ensure_index(self, filters: Dict = None):
        """确保索引存在"""
        if self._indexed:
            return
        
        with self._lock:
            if self._indexed:
                return
            
            conn = self.graph._get_conn()
            cursor = conn.cursor()
            
            sql = "SELECT id, content FROM memory_nodes WHERE is_deleted = 0"
            params = []
            
            # Entity scoping
            for key in ['user_id', 'agent_id', 'run_id']:
                if key in filters and filters[key]:
                    sql += f" AND {key} = ?"
                    params.append(filters[key])
            
            cursor.execute(sql, params)
            docs = [(row['id'], row['content']) for row in cursor.fetchall()]
            
            if docs:
                # 索引BM25
                self.bm25.index(docs)
                
                # 训练embedding
                contents = [d[1] for d in docs]
                self.embedding_encoder.fit(contents)
                
                self._indexed = True
    
    def _refresh_index(self):
        """刷新索引"""
        self._indexed = False
        self.bm25 = BM25()
        self.embedding_encoder = EmbeddingEncoder(dimension=self.embedding_dim)
    
    def search(self, query: str, top_k: int = 10,
               filters: Optional[Dict] = None,
               include_deleted: bool = False,
               rerank: bool = False) -> List[Dict]:
        """
        多维检索
        
        Args:
            query: 查询文本
            top_k: 返回数量
            filters: 过滤条件
            include_deleted: 是否包含已删除
            rerank: 是否重排
        
        Returns:
            [{id, content, score, semantic_score, bm25_score, graph_score, temporal_score}, ...]
        
        对比mem0:
        - mem0: 语义 + BM25 + 实体boost + 可选rerank
        - SuperMemory: 语义 + BM25 + 图关系 + 时间 × Φ_SPARK
        """
        # 安全验证
        query = SecurityValidator.sanitize_query(query)
        filters = SecurityValidator.validate_filters(filters or {})
        
        self._ensure_index(filters)
        
        conn = self.graph._get_conn()
        cursor = conn.cursor()
        
        # 构建查询
        sql = "SELECT * FROM memory_nodes WHERE is_deleted = 0"
        params = []
        
        for key in ['user_id', 'agent_id', 'run_id']:
            if key in filters and filters[key]:
                sql += f" AND {key} = ?"
                params.append(filters[key])
        
        if 'memory_type' in filters:
            sql += " AND memory_type = ?"
            params.append(filters['memory_type'])
        
        if not include_deleted:
            sql += " AND is_deleted = 0"
        
        cursor.execute(sql, params)
        nodes = [dict(row) for row in cursor.fetchall()]
        
        if not nodes:
            return []
        
        # 查询embedding
        query_vector = self.embedding_encoder.encode(query)
        
        # 计算各维度分数
        results = []
        for node in nodes:
            node_id = node['id']
            
            # 1. 语义相似度
            node_vector = self.embedding_encoder.encode(node['content'])
            semantic_score = self.embedding_encoder.cosine_similarity(query_vector, node_vector)
            
            # 2. BM25 (归一化)
            bm25_score = self.bm25.score(query, 
                self.bm25.doc_ids.index(node_id) if node_id in self.bm25.doc_ids else 0)
            bm25_norm = min(bm25_score / 100.0, 1.0) if bm25_score > 0 else 0
            
            # 3. 图关系分数
            graph_score = self._calc_graph_score(node_id, query, filters)
            
            # 4. 时间分数 (SPW-R)
            temporal_score = self._calc_temporal_score(node)
            
            # 加权融合 × SPW-R增强
            combined = (
                self.semantic_weight * semantic_score +
                self.bm25_weight * bm25_norm +
                self.graph_weight * graph_score +
                self.temporal_weight * temporal_score
            ) * PHI_SPARK
            
            results.append({
                'id': node_id,
                'content': node['content'],
                'memory_type': node['memory_type'],
                'score': combined,
                'semantic_score': semantic_score,
                'bm25_score': bm25_score,
                'graph_score': graph_score,
                'temporal_score': temporal_score,
                'created_at': node['created_at'],
                'importance': node.get('importance', 1.0),
                'access_count': node.get('access_count', 0),
            })
        
        # 排序
        results.sort(key=lambda x: x['score'], reverse=True)
        
        # 可选重排 (基于重要性)
        if rerank:
            results = self._rerank(results)
        
        return results[:top_k]
    
    def _calc_graph_score(self, node_id: str, query: str, filters: Dict) -> float:
        """计算图关系分数"""
        keywords = set(re.findall(r'\b\w+\b', query.lower()))
        
        neighbors = self.graph.get_neighbors(node_id, depth=2, 
                                             user_id=filters.get('user_id'))
        
        if not neighbors:
            return 0.0
        
        conn = self.graph._get_conn()
        cursor = conn.cursor()
        
        placeholders = ','.join('?' * len(neighbors))
        cursor.execute(f"""
            SELECT content FROM memory_nodes 
            WHERE id IN ({placeholders}) AND is_deleted = 0
        """, neighbors)
        
        score = 0.0
        for row in cursor.fetchall():
            content_lower = row[0].lower()
            matches = sum(1 for kw in keywords if kw in content_lower)
            score += matches * 0.1
        
        # 邻居数量加分
        score += min(len(neighbors) * 0.05, 0.5)
        
        return min(score, 1.0)
    
    def _calc_temporal_score(self, node: Dict) -> float:
        """
        SPW-R时间重要性分数
        =================
        
        模拟海马体Sharp Wave Ripples的时间编码:
        - 最近访问的记忆更重要
        - 高访问频率的记忆更重要
        - 高重要性的记忆更重要
        
        SPW-R增强: Φ_SPARK = 3.38
        """
        now = datetime.now()
        
        # 访问频率分数
        access_count = node.get('access_count', 0)
        access_score = min(access_count / 100, 1.0)
        
        # 最近访问
        try:
            updated = datetime.fromisoformat(node.get('updated_at', node.get('created_at', '')))
            days_since = (now - updated).days
            recency_score = math.exp(-days_since / 30)  # 指数衰减
        except:
            recency_score = 0.5
        
        # 重要性
        importance = node.get('importance', 1.0)
        
        # 时间衰减
        decay_factor = node.get('decay_factor', 1.0)
        
        # SPW-R融合
        temporal = (
            access_score * 0.30 +
            recency_score * 0.40 +
            importance * 0.30
        ) * decay_factor
        
        # SPW-R神经振荡增强
        temporal *= PHI_SPARK / 3.0
        
        return min(temporal, 1.0)
    
    def _rerank(self, results: List[Dict]) -> List[Dict]:
        """重排结果"""
        # 基于多信号重新评分
        reranked = []
        
        for r in results:
            # 重要性加权
            importance_weight = r.get('importance', 1.0)
            access_weight = min(r.get('access_count', 0) / 50, 1.0)
            
            new_score = r['score'] * (1 + importance_weight * 0.5 + access_weight * 0.3)
            r['score'] = new_score
            reranked.append(r)
        
        reranked.sort(key=lambda x: x['score'], reverse=True)
        return reranked
    
    def get_relevant_context(self, node_id: str, depth: int = 2,
                             user_id: str = None) -> str:
        """获取相关上下文"""
        neighbors = self.graph.get_neighbors(node_id, depth=depth, user_id=user_id)
        
        if not neighbors:
            return ""
        
        conn = self.graph._get_conn()
        cursor = conn.cursor()
        
        placeholders = ','.join('?' * len(neighbors))
        cursor.execute(f"""
            SELECT content FROM memory_nodes 
            WHERE id IN ({placeholders}) AND is_deleted = 0
            ORDER BY temporal_score DESC, created_at DESC
            LIMIT 20
        """, neighbors)
        
        contents = [row[0] for row in cursor.fetchall()]
        
        return "\n\n---\n\n".join(contents)
    
    def extract_entities(self, text: str) -> List[Dict]:
        """提取实体"""
        entities = []
        
        # 命名实体识别 (简单版)
        patterns = [
            (r'\b([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)\b', 'person'),
            (r'\b([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)\b', 'concept'),
        ]
        
        for pattern, entity_type in patterns:
            matches = re.findall(pattern, text)
            for match in matches:
                if len(match) > 2:
                    entities.append({
                        'text': match,
                        'type': entity_type,
                        'start': text.find(match),
                        'end': text.find(match) + len(match),
                    })
        
        return entities
    
    def keyword_search(self, query: str, limit: int = 100) -> List[Dict]:
        """关键词搜索 (FTS5)"""
        conn = self.graph._get_conn()
        cursor = conn.cursor()
        
        # FTS5搜索
        cursor.execute("""
            SELECT m.* FROM memory_nodes m
            JOIN memory_fts fts ON m.id = fts.id
            WHERE memory_fts MATCH ?
            AND m.is_deleted = 0
            ORDER BY rank
            LIMIT ?
        """, (query, limit))
        
        return [dict(row) for row in cursor.fetchall()]


# 线程锁
import threading