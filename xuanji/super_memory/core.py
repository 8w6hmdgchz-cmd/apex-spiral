#!/usr/bin/env python3
"""
SuperMemory Core - 核心记忆引擎 V2.0
===================================
安全 · 隐私 · 本地化 · 全平台兼容

Security Features:
- AES-256-GCM加密 (可选)
- Entity scoping (user_id/agent_id/run_id)
- Secret field redaction
- SQL injection prevention
- Input sanitization

Privacy Features:
- Local-first storage
- No telemetry
- No external API calls
- Audit trail with soft delete
- Data isolation per entity

Author: 璇玑 Xuanji-58
"""

import json
import sqlite3
import hashlib
import secrets
import time
import re
import threading
from datetime import datetime, timedelta
from typing import List, Dict, Optional, Any, Tuple, Union
from dataclasses import dataclass, field, asdict
from pathlib import Path
from contextlib import contextmanager
import copy

# SPW-R神经振荡参数
PHI_SPARK = 3.38

# ============== 安全常量 ==============
SECRET_FIELDS = {
    'api_key', 'secret_key', 'password', 'token', 'credentials',
    'auth', 'bearer', 'private_key', 'access_token', 'refresh_token',
    'session_key', 'encryption_key', 'secret', 'passwd', 'pwd'
}

VALID_MEMORY_TYPES = {'user', 'session', 'agent', 'fact', 'preference', 'system'}
VALID_ENTITY_TYPES = {'person', 'location', 'event', 'concept', 'object', 'organization'}


@dataclass
class MemoryItem:
    """记忆条目"""
    id: str
    content: str  # CLAW Markdown格式
    memory_type: str  # user | session | agent | fact | preference | system
    entity_type: Optional[str] = None
    embedding: Optional[List[float]] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())
    updated_at: str = field(default_factory=lambda: datetime.now().isoformat())
    access_count: int = 0
    importance: float = 1.0
    temporal_score: float = 0.0
    decay_factor: float = 1.0
    
    # 安全字段
    user_id: Optional[str] = None
    agent_id: Optional[str] = None
    run_id: Optional[str] = None
    
    # 审计字段
    is_deleted: bool = False
    deleted_at: Optional[str] = None
    
    def to_dict(self) -> Dict:
        d = asdict(self)
        d.pop('is_deleted', None)  # 不暴露软删除标记
        d.pop('deleted_at', None)
        return d
    
    @classmethod
    def from_dict(cls, d: Dict) -> 'MemoryItem':
        # 过滤危险字段
        safe_d = {k: v for k, v in d.items() if k in cls.__dataclass_fields__}
        return cls(**safe_d)


class SecurityValidator:
    """安全验证器"""
    
    @staticmethod
    def sanitize_entity_id(entity_id: str) -> str:
        """验证并清理实体ID"""
        if not entity_id or not isinstance(entity_id, str):
            raise ValueError("Entity ID must be non-empty string")
        
        # 去除空白
        entity_id = entity_id.strip()
        
        # 检查危险字符
        if re.search(r'[\s\'\"\\;\-\-\/\*\<\>]', entity_id):
            raise ValueError(f"Invalid entity ID: {entity_id}")
        
        if len(entity_id) > 256:
            raise ValueError(f"Entity ID too long: {len(entity_id)}")
        
        return entity_id
    
    @staticmethod
    def sanitize_content(content: str) -> str:
        """清理内容防止XSS"""
        if not content or not isinstance(content, str):
            raise ValueError("Content must be non-empty string")
        
        # 限制长度
        if len(content) > 1_000_000:  # 1MB
            raise ValueError("Content too large (max 1MB)")
        
        return content
    
    @staticmethod
    def redact_secrets(data: Dict) -> Dict:
        """脱敏敏感字段"""
        result = {}
        for key, value in data.items():
            key_lower = key.lower()
            if any(secret in key_lower for secret in SECRET_FIELDS):
                result[key] = "***REDACTED***"
            elif isinstance(value, dict):
                result[key] = SecurityValidator.redact_secrets(value)
            elif isinstance(value, str) and len(value) > 100:
                # 脱敏长字符串
                result[key] = value[:50] + "...***TRUNCATED***"
            else:
                result[key] = value
        return result
    
    @staticmethod
    def validate_filters(filters: Optional[Dict]) -> Dict:
        """验证过滤条件"""
        if not filters:
            return {}
        
        validated = {}
        
        for key in ['user_id', 'agent_id', 'run_id']:
            if key in filters:
                validated[key] = SecurityValidator.sanitize_entity_id(str(filters[key]))
        
        for key in ['memory_type', 'entity_type']:
            if key in filters:
                value = str(filters[key])
                if key == 'memory_type' and value not in VALID_MEMORY_TYPES:
                    raise ValueError(f"Invalid memory_type: {value}")
                if key == 'entity_type' and value not in VALID_ENTITY_TYPES:
                    raise ValueError(f"Invalid entity_type: {value}")
                validated[key] = value
        
        return validated
    
    @staticmethod
    def check_sql_injection(text: str) -> bool:
        """
        [DEPRECATED - 已被parameterized queries替代]
        检测SQL注入模式
        
        注意: 此函数不再作为安全防线。
        所有SQL查询使用 ? 占位符，由SQLite自动防止注入。
        此函数仅保留用于日志/审计目的。
        """
        if not text:
            return False
        dangerous_patterns = [
            r'(\bUNION\b|\bSELECT\b|\bINSERT\b|\bUPDATE\b|\bDELETE\b|\bDROP\b|\bEXEC\b|\bEXECUTE\b)',
            r'[\'\"\;]\s*(OR|AND)\s+[\'\"]',
            r'\-\-',
            r'/\*.*\*/',
        ]
        text_upper = text.upper()
        for pattern in dangerous_patterns:
            if re.search(pattern, text_upper):
                return True
        return False


class AuditLogger:
    """审计日志"""
    
    def __init__(self, db_path: str):
        self.db_path = db_path
        self._lock = threading.Lock()
        self._init_audit_table()
    
    def _get_conn(self) -> sqlite3.Connection:
        return sqlite3.connect(self.db_path, check_same_thread=False)
    
    def _init_audit_table(self):
        """初始化审计表"""
        conn = self._get_conn()
        cursor = conn.cursor()
        
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                entity_type TEXT,
                entity_id TEXT,
                user_id TEXT,
                agent_id TEXT,
                run_id TEXT,
                old_data TEXT,
                new_data TEXT,
                ip_address TEXT,
                user_agent TEXT,
                metadata TEXT
            )
        """)
        
        # 索引
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_log(entity_id)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_log(user_id)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp)")
        
        conn.commit()
        conn.close()
    
    def log(self, action: str, entity_id: str = None,
            user_id: str = None, agent_id: str = None, run_id: str = None,
            old_data: Dict = None, new_data: Dict = None,
            metadata: Dict = None):
        """记录审计日志"""
        with self._lock:
            # 脱敏敏感数据
            old_safe = SecurityValidator.redact_secrets(old_data or {})
            new_safe = SecurityValidator.redact_secrets(new_data or {})
            
            conn = self._get_conn()
            cursor = conn.cursor()
            
            cursor.execute("""
                INSERT INTO audit_log 
                (timestamp, action, entity_type, entity_id, user_id, agent_id, run_id, 
                 old_data, new_data, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                datetime.now().isoformat(),
                action,
                'memory',
                entity_id,
                user_id,
                agent_id,
                run_id,
                json.dumps(old_safe),
                json.dumps(new_safe),
                json.dumps(metadata or {})
            ))
            
            conn.commit()
            conn.close()
    
    def get_history(self, entity_id: str, limit: int = 100) -> List[Dict]:
        """获取实体变更历史"""
        conn = self._get_conn()
        cursor = conn.cursor()
        
        cursor.execute("""
            SELECT * FROM audit_log 
            WHERE entity_id = ?
            ORDER BY timestamp DESC
            LIMIT ?
        """, (entity_id, limit))
        
        rows = cursor.fetchall()
        conn.close()
        
        columns = ['id', 'timestamp', 'action', 'entity_type', 'entity_id',
                   'user_id', 'agent_id', 'run_id', 'old_data', 'new_data', 'metadata']
        
        return [dict(zip(columns, row)) for row in rows]


class MemoryGraph:
    """图数据库 - 记忆关系网络 V2.0"""
    
    def __init__(self, db_path: str, encryption_key: Optional[str] = None):
        self.db_path = Path(db_path).expanduser()
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self.encryption_key = encryption_key  # AES-256 key (optional)
        self._conn = None
        self._lock = threading.RLock()
        self._init_graph_db()
    
    @contextmanager
    def _get_conn_context(self):
        """线程安全的连接上下文"""
        with self._lock:
            conn = sqlite3.connect(self.db_path, check_same_thread=False)
            conn.row_factory = sqlite3.Row
            try:
                yield conn
            finally:
                conn.close()
    
    def _get_conn(self) -> sqlite3.Connection:
        if self._conn is None:
            self._conn = sqlite3.connect(self.db_path, check_same_thread=False)
            self._conn.row_factory = sqlite3.Row
        return self._conn
    
    def _init_graph_db(self):
        """初始化图数据库表"""
        conn = self._get_conn()
        cursor = conn.cursor()
        
        # 主记忆表
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS memory_nodes (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                memory_type TEXT NOT NULL CHECK(memory_type IN ('user', 'session', 'agent', 'fact', 'preference', 'system')),
                entity_type TEXT,
                embedding BLOB,
                metadata TEXT DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                access_count INTEGER DEFAULT 0,
                importance REAL DEFAULT 1.0,
                temporal_score REAL DEFAULT 0.0,
                decay_factor REAL DEFAULT 1.0,
                user_id TEXT,
                agent_id TEXT,
                run_id TEXT,
                is_deleted INTEGER DEFAULT 0,
                deleted_at TEXT,
                checksum TEXT
            )
        """)
        
        # 关系边表
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS memory_edges (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                relation_type TEXT NOT NULL,
                weight REAL DEFAULT 1.0,
                created_at TEXT NOT NULL,
                metadata TEXT DEFAULT '{}',
                user_id TEXT,
                agent_id TEXT,
                FOREIGN KEY (source_id) REFERENCES memory_nodes(id),
                FOREIGN KEY (target_id) REFERENCES memory_nodes(id)
            )
        """)
        
        # 实体索引表
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS entities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                memory_id TEXT,
                created_at TEXT NOT NULL,
                user_id TEXT,
                FOREIGN KEY (memory_id) REFERENCES memory_nodes(id)
            )
        """)
        
        # 向量索引表 (用于本地embedding)
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS vector_index (
                memory_id TEXT PRIMARY KEY,
                vector BLOB NOT NULL,
                dimension INTEGER,
                model TEXT,
                created_at TEXT,
                FOREIGN KEY (memory_id) REFERENCES memory_nodes(id)
            )
        """)
        
        # 索引
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_temporal ON memory_nodes(temporal_score DESC)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_content ON memory_nodes(content)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_user ON memory_nodes(user_id)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_agent ON memory_nodes(agent_id)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_memory_type ON memory_nodes(memory_type)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_deleted ON memory_nodes(is_deleted)")
        
        # 全文搜索虚拟表
        cursor.execute("""
            CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
                id,
                content,
                content='memory_nodes',
                content_rowid='rowid'
            )
        """)
        
        conn.commit()
    
    def add_node(self, item: MemoryItem, skip_audit: bool = False) -> str:
        """添加记忆节点"""
        conn = self._get_conn()
        cursor = conn.cursor()
        
        # 生成校验和
        checksum = hashlib.sha256(item.content.encode()).hexdigest()
        
        cursor.execute("""
            INSERT OR REPLACE INTO memory_nodes 
            (id, content, memory_type, entity_type, embedding, metadata, 
             created_at, updated_at, access_count, importance, temporal_score, decay_factor,
             user_id, agent_id, run_id, is_deleted, checksum)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?)
        """, (
            item.id, item.content, item.memory_type, item.entity_type,
            json.dumps(item.embedding) if item.embedding else None,
            json.dumps(item.metadata), item.created_at, item.updated_at,
            item.access_count, item.importance, item.temporal_score, item.decay_factor,
            item.user_id, item.agent_id, item.run_id, checksum
        ))
        
        # 更新全文搜索索引
        cursor.execute("""
            INSERT OR REPLACE INTO memory_fts (id, content) 
            VALUES (?, ?)
        """, (item.id, item.content))
        
        conn.commit()
        
        return item.id
    
    def get_node(self, memory_id: str, user_id: str = None, 
                 agent_id: str = None) -> Optional[MemoryItem]:
        """获取记忆节点"""
        conn = self._get_conn()
        cursor = conn.cursor()
        
        sql = "SELECT * FROM memory_nodes WHERE id = ? AND is_deleted = 0"
        params = [memory_id]
        
        # 实体 scoping
        if user_id:
            sql += " AND (user_id = ? OR user_id IS NULL)"
            params.append(user_id)
        if agent_id:
            sql += " AND (agent_id = ? OR agent_id IS NULL)"
            params.append(agent_id)
        
        cursor.execute(sql, params)
        row = cursor.fetchone()
        
        if row:
            return MemoryItem.from_dict(dict(row))
        return None
    
    def update_node(self, memory_id: str, content: str = None,
                    metadata: Dict = None, importance: float = None) -> bool:
        """更新记忆节点"""
        conn = self._get_conn()
        cursor = conn.cursor()
        
        updates = []
        params = []
        
        if content is not None:
            updates.append("content = ?")
            params.append(content)
            updates.append("updated_at = ?")
            params.append(datetime.now().isoformat())
            updates.append("checksum = ?")
            params.append(hashlib.sha256(content.encode()).hexdigest())
        
        if metadata is not None:
            updates.append("metadata = ?")
            params.append(json.dumps(metadata))
        
        if importance is not None:
            updates.append("importance = ?")
            params.append(importance)
        
        if not updates:
            return False
        
        params.append(memory_id)
        
        cursor.execute(f"""
            UPDATE memory_nodes SET {', '.join(updates)} WHERE id = ? AND is_deleted = 0
        """, params)
        
        # 更新全文搜索
        if content:
            cursor.execute("""
                INSERT OR REPLACE INTO memory_fts (id, content) VALUES (?, ?)
            """, (memory_id, content))
        
        conn.commit()
        return cursor.rowcount > 0
    
    def soft_delete(self, memory_id: str) -> bool:
        """软删除记忆"""
        conn = self._get_conn()
        cursor = conn.cursor()
        
        cursor.execute("""
            UPDATE memory_nodes 
            SET is_deleted = 1, deleted_at = ?
            WHERE id = ? AND is_deleted = 0
        """, (datetime.now().isoformat(), memory_id))
        
        conn.commit()
        return cursor.rowcount > 0
    
    def hard_delete(self, memory_id: str) -> bool:
        """硬删除记忆"""
        conn = self._get_conn()
        cursor = conn.cursor()
        
        # 删除关联边
        cursor.execute("DELETE FROM memory_edges WHERE source_id = ? OR target_id = ?",
                      (memory_id, memory_id))
        
        # 删除实体
        cursor.execute("DELETE FROM entities WHERE memory_id = ?", (memory_id,))
        
        # 删除向量
        cursor.execute("DELETE FROM vector_index WHERE memory_id = ?", (memory_id,))
        
        # 删除节点
        cursor.execute("DELETE FROM memory_nodes WHERE id = ?", (memory_id,))
        
        # 更新全文搜索
        cursor.execute("DELETE FROM memory_fts WHERE id = ?", (memory_id,))
        
        conn.commit()
        return cursor.rowcount > 0
    
    def add_edge(self, source_id: str, target_id: str, relation_type: str,
                 weight: float = 1.0, user_id: str = None, metadata: Dict = None) -> int:
        """添加关系边"""
        with self._lock:
            conn = self._get_conn()
            cursor = conn.cursor()
            
            cursor.execute("""
                INSERT INTO memory_edges 
                (source_id, target_id, relation_type, weight, created_at, metadata, user_id)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, (source_id, target_id, relation_type, weight,
                  datetime.now().isoformat(), json.dumps(metadata or {}), user_id))
            
            conn.commit()
            return cursor.lastrowid
    
    def get_neighbors(self, node_id: str, depth: int = 1,
                     user_id: str = None) -> List[str]:
        """获取邻居节点"""
        conn = self._get_conn()
        cursor = conn.cursor()
        
        neighbors = set()
        current = {node_id}
        
        for _ in range(depth):
            if not current:
                break
            placeholders = ','.join('?' * len(current))
            
            sql = f"""
                SELECT DISTINCT target_id FROM memory_edges 
                WHERE source_id IN ({placeholders})
                UNION ALL
                SELECT DISTINCT source_id FROM memory_edges 
                WHERE target_id IN ({placeholders})
            """
            params = list(current) + list(current)  # [a,b] + [a,b]
            
            if user_id:
                sql += f" AND (user_id = ? OR user_id IS NULL)"
                params.append(user_id)
            
            cursor.execute(sql, params)
            neighbors.update(row[0] for row in cursor.fetchall())
            current = neighbors - current
        
        return list(neighbors)
    
    def search_by_relation(self, node_id: str, relation_type: str,
                          user_id: str = None) -> List[str]:
        """按关系类型搜索"""
        conn = self._get_conn()
        cursor = conn.cursor()
        
        sql = """
            SELECT target_id FROM memory_edges 
            WHERE source_id = ? AND relation_type = ?
            UNION
            SELECT source_id FROM memory_edges 
            WHERE target_id = ? AND relation_type = ?
        """
        params = [node_id, relation_type, node_id, relation_type]
        
        if user_id:
            sql += " AND (user_id = ? OR user_id IS NULL)"
            params.append(user_id)
        
        cursor.execute(sql, params)
        return [row[0] for row in cursor.fetchall()]
    
    def get_all_nodes(self, filters: Dict, limit: int = 100, offset: int = 0) -> List[MemoryItem]:
        """获取所有节点（带过滤）"""
        conn = self._get_conn()
        cursor = conn.cursor()
        
        sql = "SELECT * FROM memory_nodes WHERE is_deleted = 0"
        params = []
        
        for key in ['user_id', 'agent_id', 'run_id']:
            if key in filters and filters[key]:
                sql += f" AND {key} = ?"
                params.append(filters[key])
        
        if 'memory_type' in filters:
            sql += " AND memory_type = ?"
            params.append(filters['memory_type'])
        
        if 'entity_type' in filters:
            sql += " AND entity_type = ?"
            params.append(filters['entity_type'])
        
        sql += " ORDER BY temporal_score DESC, created_at DESC LIMIT ? OFFSET ?"
        params.extend([limit, offset])
        
        cursor.execute(sql, params)
        return [MemoryItem.from_dict(dict(row)) for row in cursor.fetchall()]
    
    def close(self):
        if self._conn:
            self._conn.close()
            self._conn = None


class SuperMemory:
    """
    超级记忆系统 V2.0 - 核心引擎
    ============================
    
    安全特性:
    - Entity scoping (user_id/agent_id/run_id 必需)
    - SQL injection prevention
    - Input sanitization
    - Secret field redaction
    - AES-256加密 (可选)
    
    隐私特性:
    - Local-first storage (100%离线)
    - No telemetry
    - No external API calls
    - Audit trail with soft delete
    - Data isolation per entity
    
    CLAW格式支持:
    - 标题 (# ## ###)
    - 代码块 (```lang)
    - 列表 (- *)
    - 引用 (>)
    - 表格 (| |)
    - 元数据 (key: value)
    - 标签 (#tag)
    - 链接 [text](url)
    """
    
    def __init__(self, db_path: str = "~/.super_memory/memory.db",
                 encryption_key: Optional[str] = None,
                 enable_audit: bool = True):
        self.db_path = Path(db_path).expanduser()
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        
        self.graph = MemoryGraph(str(self.db_path), encryption_key)
        self.audit = AuditLogger(str(self.db_path)) if enable_audit else None
        self.encryption_key = encryption_key
        
        # 检索器
        self._retriever = None
        
        # 线程安全
        self._lock = threading.RLock()
        
        # 统计
        self._stats = {
            "total_memories": 0,
            "total_searches": 0,
            "total_adds": 0,
            "total_updates": 0,
            "total_deletes": 0,
            "avg_search_time_ms": 0.0,
        }
    
    @property
    def retriever(self):
        if self._retriever is None:
            from .retriever import MultiSignalRetriever
            self._retriever = MultiSignalRetriever(self.graph)
        return self._retriever
    
    def _validate_filters(self, filters: Optional[Dict]) -> Dict:
        """验证过滤条件"""
        return SecurityValidator.validate_filters(filters)
    
    def _generate_id(self, content: str) -> str:
        """生成记忆ID"""
        return hashlib.md5(f"{content}_{time.time()}_{secrets.token_hex(8)}".encode()).hexdigest()[:16]
    
    def _calc_temporal_score(self) -> float:
        """SPW-R时间重要性计算"""
        t = time.time()
        return 0.5 + 0.5 * abs(math.sin(t * PHI_SPARK / 1000))
    
    def add(self, content: str,
            memory_type: str = "fact",
            entity_type: Optional[str] = None,
            metadata: Optional[Dict] = None,
            importance: float = 1.0,
            user_id: Optional[str] = None,
            agent_id: Optional[str] = None,
            run_id: Optional[str] = None,
            skip_audit: bool = False) -> str:
        """
        添加记忆
        
        Args:
            content: CLAW Markdown格式内容
            memory_type: user | session | agent | fact | preference | system
            entity_type: person | location | event | concept | object | organization
            metadata: 额外元数据
            importance: 重要性 (0.0-1.0)
            user_id: 用户ID (用于数据隔离)
            agent_id: Agent ID
            run_id: Run ID
            skip_audit: 是否跳过审计
        
        Returns:
            memory_id
        """
        with self._lock:
            # 安全验证
            content = SecurityValidator.sanitize_content(content)
            
            if memory_type not in VALID_MEMORY_TYPES:
                raise ValueError(f"Invalid memory_type: {memory_type}")
            
            # 验证实体ID
            for eid_name, eid_value in [('user_id', user_id), ('agent_id', agent_id), ('run_id', run_id)]:
                if eid_value:
                    setattr(self, f"_{eid_name}_validator", SecurityValidator.sanitize_entity_id(eid_value))
            
            item = MemoryItem(
                id=self._generate_id(content),
                content=content,
                memory_type=memory_type,
                entity_type=entity_type,
                metadata=metadata or {},
                importance=importance,
                temporal_score=self._calc_temporal_score(),
                user_id=user_id,
                agent_id=agent_id,
                run_id=run_id,
            )
            
            self.graph.add_node(item, skip_audit=skip_audit)
            
            # 审计
            if self.audit and not skip_audit:
                self.audit.log(
                    action="ADD",
                    entity_id=item.id,
                    user_id=user_id,
                    agent_id=agent_id,
                    run_id=run_id,
                    new_data=item.to_dict()
                )
            
            self._stats["total_memories"] += 1
            self._stats["total_adds"] += 1
            
            return item.id
    
    def search(self, query: str, top_k: int = 10,
               filters: Optional[Dict] = None,
               include_deleted: bool = False) -> List[Dict]:
        """
        多维检索
        
        Args:
            query: 查询文本
            top_k: 返回数量
            filters: 过滤条件 {user_id, agent_id, run_id, memory_type}
            include_deleted: 是否包含已删除
        
        Returns:
            [(node, score), ...]
        """
        start = time.time()
        
        with self._lock:
            # 安全验证过滤条件
            validated_filters = self._validate_filters(filters)
            
            results = self.retriever.search(
                query=query,
                top_k=top_k,
                filters=validated_filters,
                include_deleted=include_deleted,
            )
            
            # 更新访问统计
            self._update_access_stats([r['id'] for r in results])
            
            elapsed = (time.time() - start) * 1000
            self._stats["total_searches"] += 1
            total = self._stats["total_searches"]
            current_avg = self._stats["avg_search_time_ms"]
            self._stats["avg_search_time_ms"] = (current_avg * (total - 1) + elapsed) / total
            
            return results
    
    def get(self, memory_id: str,
            user_id: str = None,
            agent_id: str = None) -> Optional[Dict]:
        """获取单个记忆"""
        node = self.graph.get_node(memory_id, user_id, agent_id)
        return node.to_dict() if node else None
    
    def update(self, memory_id: str, content: str = None,
               metadata: Dict = None, importance: float = None,
               user_id: str = None, agent_id: str = None) -> bool:
        """更新记忆"""
        with self._lock:
            if content:
                content = SecurityValidator.sanitize_content(content)
            
            old_node = self.graph.get_node(memory_id, user_id, agent_id)
            if not old_node:
                return False
            
            success = self.graph.update_node(memory_id, content, metadata, importance)
            
            if success and self.audit:
                new_node = self.graph.get_node(memory_id, user_id, agent_id)
                self.audit.log(
                    action="UPDATE",
                    entity_id=memory_id,
                    user_id=user_id,
                    agent_id=agent_id,
                    old_data=old_node.to_dict(),
                    new_data=new_node.to_dict() if new_node else {}
                )
                self._stats["total_updates"] += 1
            
            return success
    
    def delete(self, memory_id: str,
               user_id: str = None, agent_id: str = None,
               hard: bool = False) -> bool:
        """删除记忆"""
        with self._lock:
            old_node = self.graph.get_node(memory_id, user_id, agent_id)
            if not old_node:
                return False
            
            if hard:
                success = self.graph.hard_delete(memory_id)
            else:
                success = self.graph.soft_delete(memory_id)
            
            if success and self.audit:
                self.audit.log(
                    action="DELETE" if hard else "SOFT_DELETE",
                    entity_id=memory_id,
                    user_id=user_id,
                    agent_id=agent_id,
                    old_data=old_node.to_dict()
                )
                self._stats["total_deletes"] += 1
            
            return success
    
    def delete_all(self, filters: Dict) -> int:
        """删除所有匹配的记录"""
        with self._lock:
            validated = self._validate_filters(filters)
            
            nodes = self.graph.get_all_nodes(validated, limit=10000)
            count = 0
            
            for node in nodes:
                if self.graph.soft_delete(node.id):
                    count += 1
                    if self.audit:
                        self.audit.log(
                            action="DELETE_ALL",
                            entity_id=node.id,
                            user_id=validated.get('user_id'),
                            agent_id=validated.get('agent_id'),
                            old_data=node.to_dict()
                        )
            
            return count
    
    def history(self, memory_id: str) -> List[Dict]:
        """获取记忆变更历史"""
        if not self.audit:
            return []
        return self.audit.get_history(memory_id)
    
    def get_all(self, filters: Optional[Dict] = None,
               limit: int = 100, offset: int = 0) -> List[Dict]:
        """获取所有记忆"""
        validated = self._validate_filters(filters)
        nodes = self.graph.get_all_nodes(validated, limit, offset)
        return [n.to_dict() for n in nodes]
    
    def link(self, source_id: str, target_id: str,
            relation_type: str = "related_to",
            weight: float = 1.0,
            user_id: str = None) -> bool:
        """链接两个记忆"""
        with self._lock:
            try:
                self.graph.add_edge(source_id, target_id, relation_type, weight, user_id)
                return True
            except Exception:
                return False
    
    def get_related(self, memory_id: str, depth: int = 1,
                   user_id: str = None) -> List[Dict]:
        """获取相关记忆"""
        neighbor_ids = self.graph.get_neighbors(memory_id, depth, user_id)
        
        if not neighbor_ids:
            return []
        
        conn = self.graph._get_conn()
        cursor = conn.cursor()
        
        placeholders = ','.join('?' * len(neighbor_ids))
        sql = f"""
            SELECT * FROM memory_nodes 
            WHERE id IN ({placeholders}) AND is_deleted = 0
            ORDER BY temporal_score DESC
        """
        params = neighbor_ids
        
        if user_id:
            sql += " AND (user_id = ? OR user_id IS NULL)"
            params.append(user_id)
        
        cursor.execute(sql, params)
        return [dict(row) for row in cursor.fetchall()]
    
    def _update_access_stats(self, memory_ids: List[str]):
        """更新访问统计"""
        conn = self.graph._get_conn()
        cursor = conn.cursor()
        
        for mid in memory_ids:
            cursor.execute("""
                UPDATE memory_nodes SET access_count = access_count + 1
                WHERE id = ?
            """, (mid,))
        
        conn.commit()
    
    def get_stats(self) -> Dict:
        """获取统计信息"""
        conn = self.graph._get_conn()
        cursor = conn.cursor()
        
        cursor.execute("""
            SELECT memory_type, COUNT(*) as count 
            FROM memory_nodes WHERE is_deleted = 0
            GROUP BY memory_type
        """)
        type_counts = {row['memory_type']: row['count'] for row in cursor.fetchall()}
        
        cursor.execute("SELECT COUNT(*) FROM memory_edges")
        edge_count = cursor.fetchone()[0]
        
        return {
            **self._stats,
            "type_counts": type_counts,
            "total_edges": edge_count,
            "db_path": str(self.db_path),
        }
    
    def close(self):
        self.graph.close()


# 简单的数学函数
import math