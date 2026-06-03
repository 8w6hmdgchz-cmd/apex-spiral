"""工具自注册中心。

核心思路（来自 hermes-agent `tools/registry.py`）：
- 工具文件模块顶层调用 `registry.register(...)` 完成注册
- 外部无需维护工具列表，registry 自动发现
- 支持 name / description / func 三要素
- 支持同步和异步函数

设计取舍：
- 比 hermes-agent 简化：去掉了 Toolset 分组（暂时不需要）
- 比 hermes-agent 增加：每个工具可带 `tags` 用于按场景过滤
- 比 hermes-agent 增加：单例 `get_registry()`，避免多实例混乱
"""
from __future__ import annotations

import asyncio
import inspect
import threading
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, List, Optional


@dataclass
class Tool:
    """工具描述。"""
    name: str
    description: str
    func: Callable
    tags: List[str] = field(default_factory=list)
    is_async: bool = False

    def invoke(self, *args: Any, **kwargs: Any) -> Any:
        """同步入口：异步函数走 asyncio.run；同步函数直接调。"""
        if self.is_async:
            try:
                loop = asyncio.get_event_loop()
                if loop.is_running():
                    # 已经在事件循环里，不能 asyncio.run，返回 coroutine 让上层 await
                    return self.func(*args, **kwargs)
            except RuntimeError:
                pass
            return asyncio.run(self.func(*args, **kwargs))
        return self.func(*args, **kwargs)

    async def __call__(self, *args: Any, **kwargs: Any) -> Any:
        if self.is_async:
            return await self.func(*args, **kwargs)
        # 同步函数扔到线程池里跑，避免阻塞事件循环
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, lambda: self.func(*args, **kwargs))


class ToolRegistry:
    """工具注册中心（线程安全单例模式）。"""

    def __init__(self) -> None:
        self._tools: Dict[str, Tool] = {}
        self._lock = threading.Lock()

    def register(
        self,
        name: str,
        description: str = "",
        tags: Optional[List[str]] = None,
    ) -> Callable[[Callable], Callable]:
        """装饰器用法：

        @registry.register(name="echo", description="回显输入")
        def echo(text: str) -> str:
            return text
        """
        def decorator(func: Callable) -> Callable:
            tool = Tool(
                name=name,
                description=description or (func.__doc__ or "").strip().split("\n")[0],
                func=func,
                tags=list(tags or []),
                is_async=asyncio.iscoroutinefunction(func),
            )
            with self._lock:
                if name in self._tools:
                    raise ValueError(f"tool '{name}' already registered")
                self._tools[name] = tool
            return func
        return decorator

    def get(self, name: str) -> Tool:
        with self._lock:
            if name not in self._tools:
                raise KeyError(f"tool '{name}' not found. available: {list(self._tools)}")
            return self._tools[name]

    def list(self, tag: Optional[str] = None) -> List[str]:
        with self._lock:
            if tag is None:
                return sorted(self._tools.keys())
            return sorted(n for n, t in self._tools.items() if tag in t.tags)

    def all_tools(self) -> List[Tool]:
        with self._lock:
            return list(self._tools.values())

    def __len__(self) -> int:
        return len(self._tools)

    def __repr__(self) -> str:
        return f"<ToolRegistry tools={list(self._tools)}>"


# 单例
_global_registry: Optional[ToolRegistry] = None
_global_lock = threading.Lock()


def get_registry() -> ToolRegistry:
    """全局工具注册中心（线程安全单例）。"""
    global _global_registry
    if _global_registry is None:
        with _global_lock:
            if _global_registry is None:
                _global_registry = ToolRegistry()
    return _global_registry


# 便捷装饰器：直接 @tool("name", "desc")
def tool(name: str, description: str = "", tags: Optional[List[str]] = None) -> Callable:
    """@tool("name", "desc") 装饰器糖。"""
    return get_registry().register(name=name, description=description, tags=tags)
