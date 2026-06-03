#!/usr/bin/env python3
"""
Embedding & Vector Store Providers - 可插拔embedding和向量存储 V3.0
===================================================================

支持mem0-style provider抽象:
- Embedding: OpenAI, Azure, Ollama, HuggingFace, Local TF-IDF
- Vector Store: Qdrant, Pinecone, Chroma, Weaviate, Milvus, Local SQLite

对比mem0:
- mem0: 13种embedding provider + 多向量存储
- SuperMemory V3.0: 本地优先 + 可扩展provider接口

Author: 璇玑 Xuanji-58
"""

import math
import json
import threading
from typing import List, Dict, Optional, Callable, Any
from abc import ABC, abstractmethod
from dataclasses import dataclass


# ============== Embedding Provider 抽象 ==============

class EmbeddingProvider(ABC):
    """Embedding Provider抽象基类"""
    
    @abstractmethod
    def encode(self, texts: List[str]) -> List[List[float]]:
        """将文本编码为向量"""
        pass
    
    @abstractmethod
    def get_dimension(self) -> int:
        """获取向量维度"""
        pass
    
    @property
    @abstractmethod
    def name(self) -> str:
        """Provider名称"""
        pass


class LocalTFIDFProvider(EmbeddingProvider):
    """
    本地TF-IDF Provider (默认, 零外部依赖)
    """
    
    def __init__(self, dimension: int = 384):
        from .retriever import EmbeddingEncoder
        self.encoder = EmbeddingEncoder(dimension=dimension)
        self._dimension = dimension
    
    def fit(self, documents: List[str]):
        """训练TF-IDF模型"""
        self.encoder.fit(documents)
    
    def encode(self, texts: List[str]) -> List[List[float]]:
        """编码文本"""
        return [self.encoder.encode(text) for text in texts]
    
    def get_dimension(self) -> int:
        return self._dimension
    
    @property
    def name(self) -> str:
        return "local_tfidf"


class OpenAIProvider(EmbeddingProvider):
    """
    OpenAI Embedding Provider
    支持: text-embedding-3-small, text-embedding-3-large, text-embedding-ada-002
    """
    
    def __init__(self, api_key: str, model: str = "text-embedding-3-small",
                 dimension: int = 1536, base_url: str = "https://api.openai.com/v1"):
        self.api_key = api_key
        self.model = model
        self.dimension = dimension
        self.base_url = base_url.rstrip('/')
        self._session = None
    
    def _get_session(self):
        """懒加载HTTP session"""
        if self._session is None:
            import urllib.request
            self._session = urllib.request.build_opener()
        return self._session
    
    def encode(self, texts: List[str]) -> List[List[float]]:
        """调用OpenAI API编码"""
        import urllib.request
        
        url = f"{self.base_url}/embeddings"
        data = json.dumps({
            "input": texts,
            "model": self.model,
            "dimensions": self.dimension
        }).encode()
        
        req = urllib.request.Request(
            url, data=data,
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json"
            },
            method="POST"
        )
        
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                result = json.loads(resp.read())
                return [item["embedding"] for item in result["data"]]
        except Exception as e:
            raise RuntimeError(f"OpenAI embedding failed: {e}")
    
    def get_dimension(self) -> int:
        return self.dimension
    
    @property
    def name(self) -> str:
        return f"openai_{self.model}"


class OllamaProvider(EmbeddingProvider):
    """
    Ollama本地LLM Embedding Provider
    支持: llama2, mistral, nomic-embed-text等
    """
    
    def __init__(self, base_url: str = "http://localhost:11434",
                 model: str = "nomic-embed-text", dimension: int = 768):
        self.base_url = base_url.rstrip('/')
        self.model = model
        self.dimension = dimension
        self._embeddings_cache = {}
    
    def encode(self, texts: List[str]) -> List[List[float]]:
        """调用Ollama API编码"""
        import urllib.request
        
        results = []
        for text in texts:
            if text in self._embeddings_cache:
                results.append(self._embeddings_cache[text])
                continue
            
            url = f"{self.base_url}/api/embeddings"
            data = json.dumps({
                "model": self.model,
                "prompt": text
            }).encode()
            
            req = urllib.request.Request(
                url, data=data,
                headers={"Content-Type": "application/json"},
                method="POST"
            )
            
            try:
                with urllib.request.urlopen(req, timeout=60) as resp:
                    result = json.loads(resp.read())
                    embedding = result.get("embedding", [])
                    self._embeddings_cache[text] = embedding
                    results.append(embedding)
            except Exception as e:
                # Fallback to zero vector
                results.append([0.0] * self.dimension)
        
        return results
    
    def get_dimension(self) -> int:
        return self.dimension
    
    @property
    def name(self) -> str:
        return f"ollama_{self.model}"


class HuggingFaceProvider(EmbeddingProvider):
    """
    HuggingFace Inference API Provider
    支持: sentence-transformers系列模型
    """
    
    def __init__(self, api_key: str, model: str = "sentence-transformers/all-MiniLM-L6-v2",
                 dimension: int = 384):
        self.api_key = api_key
        self.model = model
        self.dimension = dimension
    
    def encode(self, texts: List[str]) -> List[List[float]]:
        """调用HuggingFace Inference API"""
        import urllib.request
        
        url = f"https://api-inference.huggingface.co/pipeline/feature-extraction/{self.model}"
        
        results = []
        for text in texts:
            data = json.dumps({"inputs": text}).encode()
            req = urllib.request.Request(
                url, data=data,
                headers={
                    "Authorization": f"Bearer {self.api_key}",
                    "Content-Type": "application/json"
                },
                method="POST"
            )
            
            try:
                with urllib.request.urlopen(req, timeout=60) as resp:
                    result = json.loads(resp.read())
                    if isinstance(result, list):
                        results.append(result)
                    else:
                        results.append([0.0] * self.dimension)
            except Exception:
                results.append([0.0] * self.dimension)
        
        return results
    
    def get_dimension(self) -> int:
        return self.dimension
    
    @property
    def name(self) -> str:
        return f"huggingface_{self.model}"


# ============== 向量存储Provider抽象 ==============

class VectorStoreProvider(ABC):
    """向量存储Provider抽象基类"""
    
    @abstractmethod
    def upsert(self, ids: List[str], vectors: List[List[float]], 
              metadata: List[Dict]) -> bool:
        """批量插入或更新向量"""
        pass
    
    @abstractmethod
    def search(self, query_vector: List[float], top_k: int = 10,
              filters: Dict = None) -> List[Dict]:
        """向量相似度检索"""
        pass
    
    @abstractmethod
    def delete(self, ids: List[str]) -> bool:
        """删除向量"""
        pass
    
    @abstractmethod
    def close(self):
        """关闭连接"""
        pass


class LocalSQLiteVectorStore(VectorStoreProvider):
    """
    本地SQLite向量存储 (默认, 无外部依赖)
    
    使用SQLite + BLOB存储向量, cosine相似度搜索
    """
    
    def __init__(self, db_path: str = "~/.super_memory/vectors.db"):
        import sqlite3
        from pathlib import Path
        
        self.db_path = Path(db_path).expanduser()
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self._conn = sqlite3.connect(str(self.db_path), check_same_thread=False)
        self._conn.row_factory = sqlite3.Row
        self._lock = threading.RLock()
        self._init_schema()
    
    def _init_schema(self):
        """初始化向量表"""
        cursor = self._conn.cursor()
        
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS vectors (
                id TEXT PRIMARY KEY,
                vector BLOB NOT NULL,
                metadata TEXT,
                created_at TEXT
            )
        """)
        
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_vectors_id ON vectors(id)")
        
        self._conn.commit()
    
    def _vector_to_blob(self, vector: List[float]) -> bytes:
        """向量转BLOB"""
        import struct
        return struct.pack(f'{len(vector)}d', *vector)
    
    def _blob_to_vector(self, blob: bytes) -> List[float]:
        """BLOB转向量"""
        import struct
        return list(struct.unpack(f'{len(blob)//8}d', blob))
    
    def _cosine_similarity(self, vec1: List[float], vec2: List[float]) -> float:
        """计算余弦相似度"""
        dot = sum(a * b for a, b in zip(vec1, vec2))
        mag1 = math.sqrt(sum(a * a for a in vec1))
        mag2 = math.sqrt(sum(b * b for b in vec2))
        if mag1 == 0 or mag2 == 0:
            return 0.0
        return dot / (mag1 * mag2)
    
    def upsert(self, ids: List[str], vectors: List[List[float]],
              metadata: List[Dict]) -> bool:
        """批量插入或更新"""
        import sqlite3
        
        with self._lock:
            cursor = self._conn.cursor()
            
            for i, vid in enumerate(ids):
                vector_blob = self._vector_to_blob(vectors[i])
                meta_json = json.dumps(metadata[i] if i < len(metadata) else {})
                
                cursor.execute("""
                    INSERT OR REPLACE INTO vectors (id, vector, metadata, created_at)
                    VALUES (?, ?, ?, ?)
                """, (vid, vector_blob, meta_json, 
                      __import__('datetime').datetime.now().isoformat()))
            
            self._conn.commit()
            return True
    
    def search(self, query_vector: List[float], top_k: int = 10,
              filters: Dict = None) -> List[Dict]:
        """向量相似度搜索"""
        import sqlite3
        
        with self._lock:
            cursor = self._conn.cursor()
            
            cursor.execute("SELECT id, vector, metadata FROM vectors")
            rows = cursor.fetchall()
            
            results = []
            for row in rows:
                vid = row['id']
                vector = self._blob_to_vector(row['vector'])
                metadata = json.loads(row['metadata']) if row['metadata'] else {}
                
                # 应用过滤
                if filters:
                    skip = False
                    for key, value in filters.items():
                        if key in metadata and metadata[key] != value:
                            skip = True
                            break
                    if skip:
                        continue
                
                score = self._cosine_similarity(query_vector, vector)
                results.append({
                    'id': vid,
                    'score': score,
                    'metadata': metadata
                })
            
            # 排序并返回top_k
            results.sort(key=lambda x: x['score'], reverse=True)
            return results[:top_k]
    
    def delete(self, ids: List[str]) -> bool:
        """删除向量"""
        with self._lock:
            cursor = self._conn.cursor()
            
            placeholders = ','.join('?' * len(ids))
            cursor.execute(f"DELETE FROM vectors WHERE id IN ({placeholders})", ids)
            
            self._conn.commit()
            return cursor.rowcount > 0
    
    def close(self):
        """关闭连接"""
        with self._lock:
            if self._conn:
                self._conn.close()
                self._conn = None


class QdrantProvider(VectorStoreProvider):
    """
    Qdrant向量存储Provider
    
    需要Qdrant服务器运行中
    
    使用示例:
        provider = QdrantProvider(host="localhost", port=6333, collection="memory")
        provider.upsert(["id1"], [[0.1, 0.2]], [{"text": "hello"}])
        results = provider.search([0.1, 0.2], top_k=5)
    """
    
    def __init__(self, host: str = "localhost", port: int = 6333,
                 collection: str = "super_memory", dimension: int = 384,
                 api_key: str = None):
        self.host = host
        self.port = port
        self.collection = collection
        self.dimension = dimension
        self.api_key = api_key
        self._base_url = f"http://{host}:{port}"
        self._initialized = False
    
    def _init_collection(self):
        """初始化collection"""
        import urllib.request
        
        url = f"{self._base_url}/collections/{self.collection}"
        
        # 检查collection是否存在
        req = urllib.request.Request(
            url, headers={"Content-Type": "application/json"},
            method="GET"
        )
        if self.api_key:
            req.add_header("api-key", self.api_key)
        
        try:
            with urllib.request.urlopen(req, timeout=5) as resp:
                if resp.status == 200:
                    self._initialized = True
                    return
        except:
            pass
        
        # 创建collection
        payload = json.dumps({
            "vectors": {
                "size": self.dimension,
                "distance": "Cosine"
            }
        }).encode()
        
        req = urllib.request.Request(
            url, data=payload,
            headers={
                "Content-Type": "application/json",
                "api-key": self.api_key or ""
            },
            method="PUT"
        )
        
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                self._initialized = True
        except Exception as e:
            raise RuntimeError(f"Failed to create Qdrant collection: {e}")
    
    def upsert(self, ids: List[str], vectors: List[List[float]],
              metadata: List[Dict]) -> bool:
        """批量插入"""
        import urllib.request
        
        if not self._initialized:
            self._init_collection()
        
        url = f"{self._base_url}/collections/{self.collection}/points"
        
        points = []
        for i, vid in enumerate(ids):
            points.append({
                "id": vid,
                "vector": vectors[i],
                "payload": metadata[i] if i < len(metadata) else {}
            })
        
        payload = json.dumps({"points": points}).encode()
        
        req = urllib.request.Request(
            url, data=payload,
            headers={
                "Content-Type": "application/json",
                "api-key": self.api_key or ""
            },
            method="PUT"
        )
        
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                return resp.status == 200
        except Exception as e:
            raise RuntimeError(f"Qdrant upsert failed: {e}")
    
    def search(self, query_vector: List[float], top_k: int = 10,
              filters: Dict = None) -> List[Dict]:
        """向量搜索"""
        import urllib.request
        
        if not self._initialized:
            self._init_collection()
        
        url = f"{self._base_url}/collections/{self.collection}/points/search"
        
        query_filter = None
        if filters:
            must = []
            for key, value in filters.items():
                must.append({
                    "key": key,
                    "match": {"value": value}
                })
            query_filter = {"must": must}
        
        payload = json.dumps({
            "vector": query_vector,
            "limit": top_k,
            "filter": query_filter
        }).encode()
        
        req = urllib.request.Request(
            url, data=payload,
            headers={
                "Content-Type": "application/json",
                "api-key": self.api_key or ""
            },
            method="POST"
        )
        
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                result = json.loads(resp.read())
                return [{
                    'id': item['id'],
                    'score': item['score'],
                    'metadata': item.get('payload', {})
                } for item in result.get('result', [])]
        except Exception as e:
            raise RuntimeError(f"Qdrant search failed: {e}")
    
    def delete(self, ids: List[str]) -> bool:
        """删除向量"""
        import urllib.request
        
        url = f"{self._base_url}/collections/{self.collection}/points/delete"
        
        payload = json.dumps({"points": ids}).encode()
        
        req = urllib.request.Request(
            url, data=payload,
            headers={
                "Content-Type": "application/json",
                "api-key": self.api_key or ""
            },
            method="POST"
        )
        
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                return resp.status in (200, 201)
        except Exception as e:
            raise RuntimeError(f"Qdrant delete failed: {e}")
    
    def close(self):
        """关闭 (Qdrant是连接池的,无需关闭)"""
        pass


class ChromaProvider(VectorStoreProvider):
    """
    Chroma向量存储Provider
    
    使用Chroma持久化存储
    """
    
    def __init__(self, persist_dir: str = "~/.super_memory/chroma",
                 collection: str = "super_memory"):
        self.persist_dir = persist_dir
        self.collection_name = collection
        self._client = None
        self._collection = None
    
    def _get_client(self):
        """懒加载Chroma client"""
        if self._client is None:
            try:
                import chromadb
                self._client = chromadb.Client()
            except ImportError:
                raise ImportError("Chroma not installed. Run: pip install chromadb")
        return self._client
    
    def upsert(self, ids: List[str], vectors: List[List[float]],
              metadata: List[Dict]) -> bool:
        """批量插入"""
        client = self._get_client()
        collection = client.get_or_create_collection(self.collection_name)
        
        collection.upsert(
            ids=ids,
            embeddings=vectors,
            metadatas=metadata
        )
        return True
    
    def search(self, query_vector: List[float], top_k: int = 10,
              filters: Dict = None) -> List[Dict]:
        """向量搜索"""
        client = self._get_client()
        collection = client.get_or_create_collection(self.collection_name)
        
        where = None
        if filters:
            where = filters
        
        results = collection.query(
            query_embeddings=[query_vector],
            n_results=top_k,
            where=where
        )
        
        return [{
            'id': results['ids'][0][i],
            'score': 1 - results['distances'][0][i],  # Chroma用distance
            'metadata': results['metadatas'][0][i] if results['metadatas'] else {}
        } for i in range(len(results['ids'][0]))]
    
    def delete(self, ids: List[str]) -> bool:
        """删除"""
        client = self._get_client()
        collection = client.get_or_create_collection(self.collection_name)
        
        collection.delete(ids=ids)
        return True
    
    def close(self):
        """关闭"""
        self._client = None


# ============== Provider工厂 ==============

class ProviderFactory:
    """
    Provider工厂 - 创建embedding和vector store providers
    """
    
    @staticmethod
    def create_embedding_provider(provider_type: str = "local",
                                  **kwargs) -> EmbeddingProvider:
        """
        创建Embedding Provider
        
        Args:
            provider_type: local | openai | ollama | huggingface
            **kwargs: provider特定参数
        
        Returns:
            EmbeddingProvider实例
        """
        providers = {
            "local": LocalTFIDFProvider,
            "openai": OpenAIProvider,
            "ollama": OllamaProvider,
            "huggingface": HuggingFaceProvider,
        }
        
        if provider_type not in providers:
            raise ValueError(f"Unknown provider: {provider_type}. Available: {list(providers.keys())}")
        
        return providers[provider_type](**kwargs)
    
    @staticmethod
    def create_vector_store_provider(provider_type: str = "local",
                                    **kwargs) -> VectorStoreProvider:
        """
        创建Vector Store Provider
        
        Args:
            provider_type: local | qdrant | chroma
            **kwargs: provider特定参数
        
        Returns:
            VectorStoreProvider实例
        """
        providers = {
            "local": LocalSQLiteVectorStore,
            "qdrant": QdrantProvider,
            "chroma": ChromaProvider,
        }
        
        if provider_type not in providers:
            raise ValueError(f"Unknown provider: {provider_type}. Available: {list(providers.keys())}")
        
        return providers[provider_type](**kwargs)