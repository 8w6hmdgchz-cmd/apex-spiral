#!/usr/bin/env python3
"""
Memory Manager - 记忆节点管理 V2.0
================================
安全 · 隐私 · 完整API

对比mem0 API:
- mem0: add(), search(), update(), delete(), get(), get_all(), history(), reset()
- SuperMemory: 完整兼容 + 更多功能

Author: 璇玑 Xuanji-58
"""

import json
import sqlite3
from typing import List, Dict, Optional, Any
from datetime import datetime
from pathlib import Path
import threading

from .core import SuperMemory, MemoryItem, SecurityValidator
from .indexer import CLAWIndexer, CLAWBuilder, CLAWValidator


class MemoryManager:
    """
    记忆节点管理器 V2.0
    ===================
    
    统一管理记忆的增删改查和关系建立
    
    对比mem0 API:
    ┌────────────────┬────────────────────────┬────────────────────────┐
    │ mem0           │ SuperMemory            │ 说明                   │
    ├────────────────┼────────────────────────┼────────────────────────┤
    │ add()          │ add()                  │ CLAW格式支持           │
    │ search()       │ search()               │ 四维检索               │
    │ update()       │ update()               │ 安全验证               │
    │ delete()       │ delete()               │ 软删除+审计            │
    │ get()          │ get()                  │ Entity scoping         │
    │ get_all()      │ get_all()              │ 分页+过滤              │
    │ history()      │ history()              │ 完整审计日志           │
    │ reset()        │ reset()                │ 安全重置               │
    │ -              │ link()                 │ 关系链接               │
    │ -              │ get_related()          │ 相关记忆               │
    │ -              │ delete_all()           │ 批量删除               │
    └────────────────┴────────────────────────┴────────────────────────┘
    """
    
    # ============== mem0兼容性别名 ==============
    def add(self, content: str, **kwargs):
        """mem0兼容别名: add_memory()"""
        return self.add_memory(content=content, **kwargs)
    
    def search(self, query: str, top_k: int = 10, **kwargs):
        """mem0兼容别名: search_memories()"""
        return self.search_memories(query=query, top_k=top_k, **kwargs)
    
    def update(self, memory_id: str, **kwargs):
        """mem0兼容别名: update_memory()"""
        return self.update_memory(memory_id=memory_id, **kwargs)
    
    def delete(self, memory_id: str, **kwargs):
        """mem0兼容别名: delete_memory()"""
        return self.delete_memory(memory_id=memory_id, **kwargs)
    
    def get(self, memory_id: str, **kwargs):
        """mem0兼容别名: get_memory()"""
        return self.get_memory(memory_id=memory_id, **kwargs)
    
    def get_all(self, **kwargs):
        """mem0兼容别名: get_all_memories()"""
        return self.get_all_memories(**kwargs)
    
    def delete_all(self, **kwargs):
        """mem0兼容别名: delete_all_memories()"""
        return self.delete_all_memories(**kwargs)
    
    def __init__(self, db_path: str = "~/.super_memory/memory.db",
                 encryption_key: Optional[str] = None,
                 enable_audit: bool = True):
        self.memory = SuperMemory(
            db_path=db_path,
            encryption_key=encryption_key,
            enable_audit=enable_audit
        )
        self.indexer = CLAWIndexer()
        self._lock = threading.RLock()
    
    def add_memory(self, content: str,
                   memory_type: str = "fact",
                   entity_type: Optional[str] = None,
                   metadata: Optional[Dict] = None,
                   importance: float = 1.0,
                   user_id: Optional[str] = None,
                   agent_id: Optional[str] = None,
                   run_id: Optional[str] = None,
                   skip_index: bool = False) -> str:
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
            skip_index: 是否跳过CLAW索引
        
        Returns:
            memory_id
        """
        with self._lock:
            # 安全验证内容
            content = SecurityValidator.sanitize_content(content)
            
            memory_id = self.memory.add(
                content=content,
                memory_type=memory_type,
                entity_type=entity_type,
                metadata=metadata,
                importance=importance,
                user_id=user_id,
                agent_id=agent_id,
                run_id=run_id,
            )
            
            # CLAW索引
            if not skip_index and (content.startswith('#') or '```' in content or '#' in content):
                index = self.indexer.index(content, memory_id)
                
                # 更新metadata中的索引信息
                self.memory.update(
                    memory_id=memory_id,
                    metadata={
                        'claw_index': json.dumps(index),
                        'tags': ','.join(index.get('tags', [])),
                        'title': index.get('title', ''),
                    },
                    user_id=user_id,
                    agent_id=agent_id
                )
            
            return memory_id
    
    def search_memories(self, query: str, top_k: int = 10,
                       memory_type: Optional[str] = None,
                       user_id: Optional[str] = None,
                       agent_id: Optional[str] = None,
                       run_id: Optional[str] = None,
                       rerank: bool = False) -> List[Dict]:
        """搜索记忆"""
        with self._lock:
            filters = {}
            if memory_type:
                filters['memory_type'] = memory_type
            if user_id:
                filters['user_id'] = user_id
            if agent_id:
                filters['agent_id'] = agent_id
            if run_id:
                filters['run_id'] = run_id
            
            return self.memory.search(
                query=query,
                top_k=top_k,
                filters=filters if filters else None
            )
    
    def get_memory(self, memory_id: str,
                   user_id: str = None,
                   agent_id: str = None) -> Optional[Dict]:
        """获取单个记忆"""
        return self.memory.get(memory_id, user_id, agent_id)
    
    def update_memory(self, memory_id: str,
                     content: Optional[str] = None,
                     metadata: Optional[Dict] = None,
                     importance: float = None,
                     user_id: str = None,
                     agent_id: str = None) -> bool:
        """更新记忆"""
        with self._lock:
            if content:
                content = SecurityValidator.sanitize_content(content)
            
            return self.memory.update(
                memory_id=memory_id,
                content=content,
                metadata=metadata,
                importance=importance,
                user_id=user_id,
                agent_id=agent_id
            )
    
    def delete_memory(self, memory_id: str,
                     user_id: str = None,
                     agent_id: str = None,
                     hard: bool = False) -> bool:
        """
        删除记忆
        
        Args:
            memory_id: 记忆ID
            user_id: 用户ID
            agent_id: Agent ID
            hard: True=硬删除, False=软删除(默认)
        """
        with self._lock:
            return self.memory.delete(
                memory_id=memory_id,
                user_id=user_id,
                agent_id=agent_id,
                hard=hard
            )
    
    def delete_all_memories(self,
                           user_id: Optional[str] = None,
                           agent_id: Optional[str] = None,
                           run_id: Optional[str] = None,
                           memory_type: Optional[str] = None) -> int:
        """删除所有匹配的记录"""
        with self._lock:
            filters = {}
            if user_id:
                filters['user_id'] = user_id
            if agent_id:
                filters['agent_id'] = agent_id
            if run_id:
                filters['run_id'] = run_id
            if memory_type:
                filters['memory_type'] = memory_type
            
            return self.memory.delete_all(filters)
    
    def get_all_memories(self,
                        user_id: Optional[str] = None,
                        agent_id: Optional[str] = None,
                        run_id: Optional[str] = None,
                        memory_type: Optional[str] = None,
                        limit: int = 100,
                        offset: int = 0) -> List[Dict]:
        """获取所有记忆"""
        with self._lock:
            filters = {}
            if user_id:
                filters['user_id'] = user_id
            if agent_id:
                filters['agent_id'] = agent_id
            if run_id:
                filters['run_id'] = run_id
            if memory_type:
                filters['memory_type'] = memory_type
            
            return self.memory.get_all(
                filters=filters if filters else None,
                limit=limit,
                offset=offset
            )
    
    def get_memory_history(self, memory_id: str) -> List[Dict]:
        """获取记忆变更历史"""
        return self.memory.history(memory_id)
    
    def link_memories(self, source_id: str, target_id: str,
                     relation_type: str = "related_to",
                     weight: float = 1.0,
                     user_id: str = None) -> bool:
        """链接两个记忆

        mem0兼容别名: link()
        """
        with self._lock:
            return self.memory.link(
                source_id=source_id,
                target_id=target_id,
                relation_type=relation_type,
                weight=weight,
                user_id=user_id
            )

    # mem0兼容别名
    link = link_memories

    def get_related_memories(self, memory_id: str,
                           depth: int = 2,
                           user_id: str = None) -> List[Dict]:
        """获取相关记忆"""
        return self.memory.get_related(memory_id, depth, user_id)
    
    def get_memories_by_type(self, memory_type: str,
                            user_id: str = None) -> List[Dict]:
        """按类型获取记忆"""
        return self.get_all_memories(memory_type=memory_type, user_id=user_id)
    
    def get_memories_by_tag(self, tag: str,
                           user_id: str = None) -> List[Dict]:
        """按标签获取记忆"""
        # 搜索包含该标签的记忆
        return self.search_memories(f"#{tag}", top_k=100, user_id=user_id)
    
    def get_recent_memories(self, days: int = 7,
                           user_id: str = None) -> List[Dict]:
        """获取最近N天的记忆"""
        with self._lock:
            all_memories = self.get_all_memories(user_id=user_id, limit=1000)
            
            from datetime import datetime, timedelta
            cutoff = datetime.now() - timedelta(days=days)
            
            recent = []
            for m in all_memories:
                try:
                    created = datetime.fromisoformat(m.get('created_at', ''))
                    if created >= cutoff:
                        recent.append(m)
                except:
                    continue
            
            return sorted(recent, key=lambda x: x.get('created_at', ''), reverse=True)
    
    def get_stats(self) -> Dict:
        """获取统计信息"""
        return self.memory.get_stats()
    
    def reset(self, user_id: str = None,
              agent_id: str = None,
              run_id: str = None,
              hard: bool = False) -> Dict:
        """
        重置记忆
        
        Args:
            user_id: 用户ID (必须指定)
            agent_id: Agent ID
            run_id: Run ID
            hard: True=硬删除, False=软删除
        
        Returns:
            {deleted_count, message}
        """
        if not any([user_id, agent_id, run_id]):
            raise ValueError("At least one of user_id/agent_id/run_id must be specified")
        
        with self._lock:
            filters = {}
            if user_id:
                filters['user_id'] = user_id
            if agent_id:
                filters['agent_id'] = agent_id
            if run_id:
                filters['run_id'] = run_id
            
            if hard:
                # 硬删除：直接删除文件重新创建
                count = self.memory.delete_all(filters)
                
                # 重新初始化
                self.memory.close()
                return {
                    'deleted_count': count,
                    'message': f"Hard reset completed for {filters}"
                }
            else:
                # 软删除
                count = self.memory.delete_all(filters)
                return {
                    'deleted_count': count,
                    'message': f"Soft reset completed for {filters}"
                }
    
    def export_memories(self, user_id: str,
                       format: str = "json") -> str:
        """
        导出记忆
        
        Args:
            user_id: 用户ID
            format: json | markdown | csv
        
        Returns:
            导出数据
        """
        memories = self.get_all_memories(user_id=user_id, limit=10000)
        
        if format == "json":
            return json.dumps(memories, indent=2, ensure_ascii=False)
        
        elif format == "markdown":
            lines = [f"# Exported Memories for {user_id}\n"]
            for m in memories:
                lines.append(f"## {m.get('id', 'unknown')}")
                lines.append(f"**Type**: {m.get('memory_type', 'unknown')}")
                lines.append(f"**Created**: {m.get('created_at', 'unknown')}")
                lines.append(f"\n{m.get('content', '')}")
                lines.append("\n---\n")
            return "\n".join(lines)
        
        elif format == "csv":
            lines = ["id,content,memory_type,created_at,importance"]
            for m in memories:
                content = m.get('content', '').replace('"', '""')
                lines.append(f'"{m.get("id", "")}","{content}","{m.get("memory_type", "")}","{m.get("created_at", "")}",{m.get("importance", 1.0)}')
            return "\n".join(lines)
        
        return ""
    
    def import_memories(self, data: str,
                       format: str = "json",
                       user_id: str = None,
                       memory_type: str = "imported") -> int:
        """
        导入记忆
        
        Args:
            data: 导入数据
            format: json | markdown
            user_id: 用户ID
            memory_type: 默认记忆类型
        
        Returns:
            导入数量
        """
        count = 0
        
        if format == "json":
            try:
                memories = json.loads(data)
                if isinstance(memories, list):
                    for m in memories:
                        content = m.get('content', '')
                        if content:
                            self.add_memory(
                                content=content,
                                memory_type=m.get('memory_type', memory_type),
                                metadata=m.get('metadata'),
                                importance=m.get('importance', 1.0),
                                user_id=user_id or m.get('user_id')
                            )
                            count += 1
            except json.JSONDecodeError:
                pass
        
        elif format == "markdown":
            # 简单解析：按---分割
            sections = data.split("---")
            for section in sections:
                if section.strip():
                    self.add_memory(
                        content=section.strip(),
                        memory_type=memory_type,
                        user_id=user_id
                    )
                    count += 1
        
        return count
    
    def validate_claw(self, content: str) -> Dict:
        """
        验证CLAW格式
        
        Returns:
            {is_valid, errors, index}
        """
        is_valid, errors = CLAWValidator.validate(content)
        index = self.indexer.index(content, "temp") if content else {}
        
        return {
            'is_valid': is_valid,
            'errors': errors,
            'index': index
        }
    
    def close(self):
        """关闭连接"""
        self.memory.close()


# 便捷函数
def create_memory(db_path: str = "~/.super_memory/memory.db",
                  encryption_key: Optional[str] = None,
                  enable_audit: bool = True) -> MemoryManager:
    """创建记忆管理器"""
    return MemoryManager(
        db_path=db_path,
        encryption_key=encryption_key,
        enable_audit=enable_audit
    )


# 向后兼容别名
MemoryGraph = None  # 已在core中定义


# ============== CLI 支持 ==============
if __name__ == "__main__":
    import sys
    
    def print_help():
        print("""
SuperMemory CLI
===============
Usage: python -m super_memory.memory <command> [args]

Commands:
  add <content> [--type fact] [--user-id USER_ID]
  search <query> [--top-k 10] [--user-id USER_ID]
  list [--user-id USER_ID] [--type TYPE]
  get <memory_id>
  delete <memory_id>
  stats
  help

Examples:
  python -m super_memory.memory add "Hello world" --type fact --user-id user123
  python -m super_memory.memory search "hello" --top-k 5
  python -m super_memory.memory list --user-id user123
  python -m super_memory.memory stats
""")
    
    if len(sys.argv) < 2:
        print_help()
        sys.exit(0)
    
    cmd = sys.argv[1]
    args = sys.argv[2:]
    
    manager = create_memory()
    
    if cmd == "add":
        content = " ".join(args[:args.index('--') if '--' in args else len(args)])
        print(f"Adding: {content}")
        mid = manager.add_memory(content=content)
        print(f"Added: {mid}")
    
    elif cmd == "search":
        query = args[0] if args else ""
        top_k = 10
        if '--top-k' in args:
            top_k = int(args[args.index('--top-k') + 1])
        results = manager.search_memories(query, top_k=top_k)
        print(f"Found {len(results)} results:")
        for r in results:
            print(f"  - {r['id']}: {r['content'][:50]}... (score: {r['score']:.3f})")
    
    elif cmd == "list":
        results = manager.get_all_memories()
        print(f"Total: {len(results)} memories")
        for r in results:
            print(f"  - {r['id']}: {r['content'][:50]}...")
    
    elif cmd == "get":
        mid = args[0] if args else ""
        result = manager.get_memory(mid)
        print(json.dumps(result, indent=2))
    
    elif cmd == "delete":
        mid = args[0] if args else ""
        manager.delete_memory(mid)
        print(f"Deleted: {mid}")
    
    elif cmd == "stats":
        stats = manager.get_stats()
        print(json.dumps(stats, indent=2))
    
    elif cmd == "help":
        print_help()
    
    else:
        print(f"Unknown command: {cmd}")
        print_help()
    
    manager.close()