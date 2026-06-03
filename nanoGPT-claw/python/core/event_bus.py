# Event Bus - 事件驱动核心
"""
NanoGPT-Claw Event Bus
=======================
事件驱动的核心系统
- 事件发布/订阅
- Tool 注册与调用
- Provider 管理
"""
import asyncio
import logging
from typing import Dict, List, Any, Callable, Coroutine, Optional
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime
import uuid
import inspect

from core.logging import get_logger


class EventType(Enum):
    """事件类型"""
    MESSAGE_RECEIVED = "message_received"
    MESSAGE_SENT = "message_sent"
    TOOL_CALLED = "tool_called"
    LLM_REQUEST = "llm_request"
    LLM_RESPONSE = "llm_response"
    ERROR = "error"
    SYSTEM = "system"
    CUSTOM = "custom"


@dataclass
class Event:
    """事件"""
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    event_type: EventType = EventType.CUSTOM
    timestamp: datetime = field(default_factory=datetime.now)
    source: str = "unknown"
    data: Dict[str, Any] = field(default_factory=dict)
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class ToolResult:
    """工具执行结果"""
    success: bool
    output: Any
    error: Optional[str] = None
    duration_ms: float = 0.0


class Tool:
    """工具基类"""
    
    name: str = ""
    description: str = ""
    parameters: Dict[str, Any] = field(default_factory=dict)
    
    async def execute(self, **kwargs) -> ToolResult:
        """执行工具"""
        raise NotImplementedError()


class Provider:
    """Provider 基类（如 LLM Provider）"""
    
    name: str = ""
    
    async def complete(self, prompt: str, **kwargs) -> Any:
        """完成请求"""
        raise NotImplementedError()


class EventBus:
    """事件总线"""
    
    def __init__(self):
        self._logger = get_logger("event_bus")
        self._subscribers: Dict[EventType, List[Callable[[Event], Coroutine]]] = {}
        self._tools: Dict[str, Tool] = {}
        self._providers: Dict[str, Provider] = {}
        self._event_queue: asyncio.Queue = asyncio.Queue()
        self._running = False
        self._worker_task: Optional[asyncio.Task] = None
    
    def subscribe(self, event_type: EventType, handler: Callable[[Event], Coroutine]):
        """订阅事件"""
        if event_type not in self._subscribers:
            self._subscribers[event_type] = []
        self._subscribers[event_type].append(handler)
        self._logger.debug(f"订阅事件: {event_type}, 当前订阅者: {len(self._subscribers[event_type])}")
    
    def unsubscribe(self, event_type: EventType, handler: Callable[[Event], Coroutine]):
        """取消订阅"""
        if event_type in self._subscribers:
            if handler in self._subscribers[event_type]:
                self._subscribers[event_type].remove(handler)
                self._logger.debug(f"取消订阅事件: {event_type}")
    
    async def publish(self, event: Event):
        """发布事件"""
        await self._event_queue.put(event)
        self._logger.debug(f"事件已排队: {event.event_type}")
    
    async def _process_events(self):
        """处理事件队列"""
        while self._running:
            try:
                event = await asyncio.wait_for(self._event_queue.get(), timeout=0.1)
                await self._dispatch_event(event)
                self._event_queue.task_done()
            except asyncio.TimeoutError:
                continue
            except Exception as e:
                self._logger.error(f"处理事件失败: {e}")
    
    async def _dispatch_event(self, event: Event):
        """分发事件到订阅者"""
        handlers = []
        
        # 获取该事件类型的订阅者
        if event.event_type in self._subscribers:
            handlers.extend(self._subscribers[event.event_type])
        
        # 获取所有事件的订阅者
        if EventType.CUSTOM in self._subscribers:
            handlers.extend(self._subscribers[EventType.CUSTOM])
        
        # 并行调用所有处理器
        if handlers:
            tasks = [handler(event) for handler in handlers]
            await asyncio.gather(*tasks, return_exceptions=True)
    
    def register_tool(self, tool: Tool):
        """注册工具"""
        if tool.name in self._tools:
            self._logger.warning(f"工具已存在，将被覆盖: {tool.name}")
        self._tools[tool.name] = tool
        self._logger.info(f"工具已注册: {tool.name}")
    
    def get_tool(self, name: str) -> Optional[Tool]:
        """获取工具"""
        return self._tools.get(name)
    
    def list_tools(self) -> List[Dict[str, Any]]:
        """列出所有工具"""
        return [
            {
                "name": tool.name,
                "description": tool.description,
                "parameters": tool.parameters,
            }
            for tool in self._tools.values()
        ]
    
    async def call_tool(self, name: str, **kwargs) -> ToolResult:
        """调用工具"""
        tool = self.get_tool(name)
        if not tool:
            return ToolResult(
                success=False,
                output=None,
                error=f"工具不存在: {name}"
            )
        
        start = datetime.now()
        
        # 发布工具调用事件
        await self.publish(Event(
            event_type=EventType.TOOL_CALLED,
            source="event_bus",
            data={"tool_name": name, "parameters": kwargs}
        ))
        
        try:
            result = await tool.execute(**kwargs)
            result.duration_ms = (datetime.now() - start).total_seconds() * 1000
            return result
        except Exception as e:
            self._logger.error(f"工具执行失败: {name}, 错误: {e}")
            return ToolResult(
                success=False,
                output=None,
                error=str(e),
                duration_ms=(datetime.now() - start).total_seconds() * 1000
            )
    
    def register_provider(self, provider: Provider):
        """注册 Provider"""
        if provider.name in self._providers:
            self._logger.warning(f"Provider 已存在，将被覆盖: {provider.name}")
        self._providers[provider.name] = provider
        self._logger.info(f"Provider 已注册: {provider.name}")
    
    def get_provider(self, name: str) -> Optional[Provider]:
        """获取 Provider"""
        return self._providers.get(name)
    
    def list_providers(self) -> List[str]:
        """列出所有 Provider"""
        return list(self._providers.keys())
    
    async def start(self):
        """启动事件总线"""
        if self._running:
            self._logger.warning("事件总线已在运行")
            return
        
        self._running = True
        self._worker_task = asyncio.create_task(self._process_events())
        self._logger.info("事件总线已启动")
    
    async def stop(self):
        """停止事件总线"""
        self._running = False
        
        if self._worker_task:
            self._worker_task.cancel()
            try:
                await self._worker_task
            except asyncio.CancelledError:
                pass
        
        self._logger.info("事件总线已停止")
    
    async def wait_empty(self):
        """等待事件队列清空"""
        await self._event_queue.join()


# 全局事件总线实例
_event_bus: Optional[EventBus] = None


def get_event_bus() -> EventBus:
    """获取全局事件总线"""
    global _event_bus
    if _event_bus is None:
        _event_bus = EventBus()
    return _event_bus
