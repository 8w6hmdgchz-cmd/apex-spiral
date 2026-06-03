# Agent Core - 事件驱动的 Agent
"""
NanoGPT-Claw Agent Core
========================
事件驱动的 Agent 系统
- 消息处理管道
- Tool 调用
- LLM 推理
"""
import logging
from typing import Optional, List, Dict, Any
from dataclasses import dataclass
from datetime import datetime

from core.config import Config, get_config
from core.logging import get_logger
from core.event_bus import (
    EventBus, Event, EventType, Tool, ToolResult,
    get_event_bus
)
# 直接从子模块导入
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

# 安全导入：先禁用其他模块的导入
import integrations
# 现在直接导入我们需要的
from integrations.llm import create_llm_provider, LLMMessage, LLMResponse
from integrations.feishu_integration import FeishuIntegration, FeishuMessage, FeishuEvent


@dataclass
class Conversation:
    """会话"""
    id: str
    messages: List[LLMMessage]
    created_at: datetime
    updated_at: datetime


class Agent:
    """Agent 核心"""
    
    def __init__(self, config: Optional[Config] = None):
        self._config = config or get_config()
        self._logger = get_logger("agent")
        
        # 事件总线
        self._event_bus = get_event_bus()
        
        # LLM Provider
        self._llm_provider = create_llm_provider(self._config.llm)
        self._event_bus.register_provider(self._llm_provider)
        
        # 飞书集成
        self._feishu = FeishuIntegration(self._config.feishu)
        
        # 会话存储
        self._conversations: Dict[str, Conversation] = {}
        
        # 注册内置工具
        self._register_builtin_tools()
        
        # 注册事件处理器
        self._setup_event_handlers()
    
    def _register_builtin_tools(self):
        """注册内置工具"""
        
        class EchoTool(Tool):
            name = "echo"
            description = "回显输入的内容"
            parameters = {"message": "要回显的内容"}
            
            async def execute(self, message: str = "", **kwargs) -> ToolResult:
                return ToolResult(success=True, output=message)
        
        class TimeTool(Tool):
            name = "get_time"
            description = "获取当前时间"
            parameters = {}
            
            async def execute(self, **kwargs) -> ToolResult:
                return ToolResult(success=True, output=datetime.now().isoformat())
        
        class ListToolsTool(Tool):
            name = "list_tools"
            description = "列出所有可用工具"
            parameters = {}
            
            def __init__(self, event_bus: EventBus):
                self._event_bus = event_bus
            
            async def execute(self, **kwargs) -> ToolResult:
                tools = self._event_bus.list_tools()
                return ToolResult(success=True, output=tools)
        
        self._event_bus.register_tool(EchoTool())
        self._event_bus.register_tool(TimeTool())
        self._event_bus.register_tool(ListToolsTool(self._event_bus))
    
    def _setup_event_handlers(self):
        """设置事件处理器"""
        # 注册飞书事件处理器
        self._feishu.register_event_handler(self._handle_feishu_event)
    
    async def _handle_feishu_event(self, event: FeishuEvent):
        """处理飞书事件"""
        if event.content and event.sender_id:
            self._logger.info(f"收到飞书消息: {event.content[:50]}")
            
            # 发布消息接收事件
            await self._event_bus.publish(Event(
                event_type=EventType.MESSAGE_RECEIVED,
                source="feishu",
                data={
                    "content": event.content,
                    "sender_id": event.sender_id,
                    "message_id": event.message_id,
                }
            ))
            
            # 处理消息
            response = await self.process_message(
                user_id=event.sender_id,
                message=event.content
            )
            
            # 发送回复
            if response:
                await self._feishu.send_message(FeishuMessage(
                    content=response,
                    receive_id=event.sender_id,
                    receive_id_type="open_id"
                ))
                
                # 发布消息发送事件
                await self._event_bus.publish(Event(
                    event_type=EventType.MESSAGE_SENT,
                    source="feishu",
                    data={"content": response[:50]}
                ))
    
    async def process_message(self, user_id: str, message: str) -> str:
        """处理用户消息"""
        # 获取或创建会话
        if user_id not in self._conversations:
            self._conversations[user_id] = Conversation(
                id=user_id,
                messages=[
                    LLMMessage(
                        role="system",
                        content="你是一个有用的助手。你可以使用工具来帮助用户。"
                    )
                ],
                created_at=datetime.now(),
                updated_at=datetime.now()
            )
        
        conversation = self._conversations[user_id]
        
        # 添加用户消息
        conversation.messages.append(LLMMessage(role="user", content=message))
        conversation.updated_at = datetime.now()
        
        # 调用 LLM
        response = await self._llm_provider.chat(
            messages=conversation.messages,
            temperature=0.7,
            max_tokens=2000
        )
        
        # 添加助手回复
        conversation.messages.append(LLMMessage(role="assistant", content=response.content))
        conversation.updated_at = datetime.now()
        
        return response.content
    
    async def start(self):
        """启动 Agent"""
        self._logger.info("启动 Agent...")
        
        # 启动事件总线
        await self._event_bus.start()
        
        # 启动飞书 WebSocket
        if self._config.feishu.app_id and self._config.feishu.app_secret:
            await self._feishu.start_websocket()
        
        self._logger.info("Agent 已启动")
    
    async def stop(self):
        """停止 Agent"""
        self._logger.info("停止 Agent...")
        
        # 停止事件总线
        await self._event_bus.stop()
        
        # 关闭飞书连接
        await self._feishu.close()
        
        self._logger.info("Agent 已停止")
    
    def get_event_bus(self) -> EventBus:
        """获取事件总线"""
        return self._event_bus
    
    def get_feishu(self) -> FeishuIntegration:
        """获取飞书集成"""
        return self._feishu
    
    def get_llm_provider(self):
        """获取 LLM Provider"""
        return self._llm_provider
