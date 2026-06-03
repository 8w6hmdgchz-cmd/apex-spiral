#!/usr/bin/env python3
"""
SuperMemory - 超级记忆系统 V2.0
===============================
安全 · 隐私 · 本地化 · 全平台兼容

超越mem0的特性:
1. 本地SQLite + 图数据库混合存储
2. CLAW原生Markdown格式支持
3. 多维检索: 语义 + BM25 + 图关系 + 时间
4. SPW-R神经振荡增强选择 (Φ=3.38)
5. ApexSpiral自我指涉进化
6. 安全: Entity scoping, SQL注入防护, XSS防护
7. 隐私: 本地优先, 无遥测, 审计日志

对比mem0:
┌─────────────────┬────────────────┬────────────────┐
│ 特性            │ mem0           │ SuperMemory    │
├─────────────────┼────────────────┼────────────────┤
│ 存储            │ 云服务/API     │ 本地SQLite ✓   │
│ 格式            │ 纯文本         │ CLAW Markdown ✓│
│ 检索            │ 语义+BM25      │ 四维融合 ✓     │
│ 进化            │ 无             │ ApexSpiral ✓   │
│ 安全            │ API Key        │ 零依赖 ✓       │
│ 部署            │ 需要网络       │ 100%离线 ✓     │
│ 审计            │ 有限           │ 完整日志 ✓     │
│ 关系图          │ 无             │ 图数据库 ✓     │
└─────────────────┴────────────────┴────────────────┘

Author: 璇玑 Xuanji-58
License: MIT
"""

from .core import (
    SuperMemory, MemoryItem, MemoryGraph, AuditLogger,
    SecurityValidator, VALID_MEMORY_TYPES, VALID_ENTITY_TYPES, PHI_SPARK
)
from .memory import MemoryManager, create_memory
from .retriever import MultiSignalRetriever, BM25, EmbeddingEncoder
from .indexer import CLAWIndexer, CLAWBuilder, CLAWParser, CLAWValidator
from .longmemeval import (
    MemoryEvaluator,        # MRR/NDCG/Recall@k评估
    MemoryConsolidator,     # 记忆整合算法
    ConflictResolver,       # 冲突检测与解决
    TemporalUpdater,        # 时间更新处理
)
from .providers import (
    EmbeddingProvider,      # Embedding Provider抽象
    LocalTFIDFProvider,     # 本地TF-IDF (默认)
    OpenAIProvider,         # OpenAI Embedding
    OllamaProvider,         # Ollama本地LLM
    HuggingFaceProvider,    # HuggingFace
    VectorStoreProvider,    # 向量存储抽象
    LocalSQLiteVectorStore, # 本地SQLite向量 (默认)
    QdrantProvider,         # Qdrant向量存储
    ChromaProvider,         # Chroma向量存储
    ProviderFactory,        # Provider工厂
)

__all__ = [
    # Core
    "SuperMemory",
    "MemoryItem",
    "MemoryGraph",
    "AuditLogger",
    "SecurityValidator",
    "VALID_MEMORY_TYPES",
    "VALID_ENTITY_TYPES",
    "PHI_SPARK",
    # Memory Manager
    "MemoryManager",
    "create_memory",
    # Retriever
    "MultiSignalRetriever",
    "BM25",
    "EmbeddingEncoder",
    # Indexer
    "CLAWIndexer",
    "CLAWBuilder",
    "CLAWParser",
    "CLAWValidator",
    # LongMemEval对齐 (V3.0新增)
    "MemoryEvaluator",       # MRR, NDCG, Recall@k
    "MemoryConsolidator",   # 重要性衰减整合
    "ConflictResolver",     # 事实冲突解决
    "TemporalUpdater",      # 时间更新处理
    # Provider抽象 (V3.0新增)
    "EmbeddingProvider",     # Embedding抽象
    "LocalTFIDFProvider",   # 本地TF-IDF
    "OpenAIProvider",       # OpenAI
    "OllamaProvider",       # Ollama
    "HuggingFaceProvider",  # HuggingFace
    "VectorStoreProvider",   # 向量存储抽象
    "LocalSQLiteVectorStore", # 本地SQLite向量
    "QdrantProvider",       # Qdrant
    "ChromaProvider",       # Chroma
    "ProviderFactory",      # Provider工厂
]

# 版本信息
VERSION = "3.0.0"
FEATURES = [
    "local_storage",
    "claw_markdown",
    "multi_signal_retrieval",
    "graph_database",
    "spw_r_enhancement",
    "apex_spiral_evolution",
    "entity_scoping",
    "sql_injection_prevention",
    "xss_prevention",
    "audit_logging",
    "soft_delete",
    "encryption_optional",
    # LongMemEval对齐 (V3.0新增)
    "mrr_ndcg_recall",     # 检索评估指标
    "memory_consolidation", # 记忆整合
    "conflict_resolution",  # 冲突解决
    "temporal_update",     # 时间更新
    # Provider抽象 (V3.0新增)
    "pluggable_embedding",  # 可插拔Embedding
    "pluggable_vectorstore", # 可插拔向量存储
]

def get_version():
    return VERSION

def get_features():
    return FEATURES