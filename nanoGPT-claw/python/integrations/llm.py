# LLM Integration - 使用官方 SDK
"""
NanoGPT-Claw LLM Integration
=============================
LLM API 调用集成层
- OpenAI
- Anthropic
- Ollama
"""
import logging
from typing import Optional, List, Dict, Any
from dataclasses import dataclass
from datetime import datetime

from core.config import LLMConfig
from core.logging import get_logger
from core.event_bus import Provider, Event, EventType, get_event_bus


@dataclass
class LLMMessage:
    """LLM 消息"""
    role: str  # "system", "user", "assistant"
    content: str


@dataclass
class LLMResponse:
    """LLM 响应"""
    content: str
    model: str
    usage: Optional[Dict[str, int]] = None
    duration_ms: float = 0.0


class OpenAIProvider(Provider):
    """OpenAI Provider"""
    
    name = "openai"
    
    def __init__(self, config: LLMConfig):
        self._config = config
        self._logger = get_logger("openai")
        self._client = None
        
        try:
            from openai import AsyncOpenAI
            self._client = AsyncOpenAI(
                api_key=config.api_key,
                base_url=config.base_url
            )
            self._logger.info("OpenAI 客户端初始化成功")
        except ImportError:
            self._logger.error("需要安装 openai: pip install openai")
        except Exception as e:
            self._logger.error(f"OpenAI 客户端初始化失败: {e}")
    
    async def complete(self, prompt: str, **kwargs) -> LLMResponse:
        """完成请求"""
        return await self.chat([LLMMessage(role="user", content=prompt)], **kwargs)
    
    async def chat(
        self,
        messages: List[LLMMessage],
        temperature: Optional[float] = None,
        max_tokens: Optional[int] = None,
        **kwargs
    ) -> LLMResponse:
        """聊天请求"""
        if not self._client:
            return LLMResponse(
                content="错误: OpenAI 客户端未初始化",
                model=self._config.model,
                duration_ms=0
            )
        
        start = datetime.now()
        
        # 发布请求事件
        event_bus = get_event_bus()
        await event_bus.publish(Event(
            event_type=EventType.LLM_REQUEST,
            source="openai",
            data={"prompt": messages[-1].content if messages else ""}
        ))
        
        try:
            response = await self._client.chat.completions.create(
                model=kwargs.get("model", self._config.model),
                messages=[
                    {"role": msg.role, "content": msg.content}
                    for msg in messages
                ],
                temperature=temperature if temperature is not None else self._config.temperature,
                max_tokens=max_tokens if max_tokens is not None else self._config.max_tokens,
            )
            
            result = LLMResponse(
                content=response.choices[0].message.content,
                model=response.model,
                usage={
                    "prompt_tokens": response.usage.prompt_tokens,
                    "completion_tokens": response.usage.completion_tokens,
                    "total_tokens": response.usage.total_tokens,
                } if response.usage else None,
                duration_ms=(datetime.now() - start).total_seconds() * 1000
            )
            
            # 发布响应事件
            await event_bus.publish(Event(
                event_type=EventType.LLM_RESPONSE,
                source="openai",
                data={"content": result.content}
            ))
            
            return result
            
        except Exception as e:
            self._logger.error(f"OpenAI 请求失败: {e}")
            return LLMResponse(
                content=f"错误: {str(e)}",
                model=self._config.model,
                duration_ms=(datetime.now() - start).total_seconds() * 1000
            )


class AnthropicProvider(Provider):
    """Anthropic Provider"""
    
    name = "anthropic"
    
    def __init__(self, config: LLMConfig):
        self._config = config
        self._logger = get_logger("anthropic")
        self._client = None
        
        try:
            from anthropic import AsyncAnthropic
            self._client = AsyncAnthropic(
                api_key=config.api_key,
                base_url=config.base_url
            )
            self._logger.info("Anthropic 客户端初始化成功")
        except ImportError:
            self._logger.error("需要安装 anthropic: pip install anthropic")
        except Exception as e:
            self._logger.error(f"Anthropic 客户端初始化失败: {e}")
    
    async def complete(self, prompt: str, **kwargs) -> LLMResponse:
        """完成请求"""
        if not self._client:
            return LLMResponse(
                content="错误: Anthropic 客户端未初始化",
                model=self._config.model,
                duration_ms=0
            )
        
        start = datetime.now()
        
        # 发布请求事件
        event_bus = get_event_bus()
        await event_bus.publish(Event(
            event_type=EventType.LLM_REQUEST,
            source="anthropic",
            data={"prompt": prompt}
        ))
        
        try:
            response = await self._client.messages.create(
                model=kwargs.get("model", self._config.model),
                max_tokens=kwargs.get("max_tokens", self._config.max_tokens),
                temperature=kwargs.get("temperature", self._config.temperature),
                messages=[{"role": "user", "content": prompt}]
            )
            
            result = LLMResponse(
                content=response.content[0].text,
                model=response.model,
                usage={
                    "input_tokens": response.usage.input_tokens,
                    "output_tokens": response.usage.output_tokens,
                } if response.usage else None,
                duration_ms=(datetime.now() - start).total_seconds() * 1000
            )
            
            # 发布响应事件
            await event_bus.publish(Event(
                event_type=EventType.LLM_RESPONSE,
                source="anthropic",
                data={"content": result.content}
            ))
            
            return result
            
        except Exception as e:
            self._logger.error(f"Anthropic 请求失败: {e}")
            return LLMResponse(
                content=f"错误: {str(e)}",
                model=self._config.model,
                duration_ms=(datetime.now() - start).total_seconds() * 1000
            )


class OllamaProvider(Provider):
    """Ollama Provider"""
    
    name = "ollama"
    
    def __init__(self, config: LLMConfig):
        self._config = config
        self._logger = get_logger("ollama")
        self._base_url = config.base_url or "http://localhost:11434"
    
    async def complete(self, prompt: str, **kwargs) -> LLMResponse:
        """完成请求"""
        import httpx
        
        start = datetime.now()
        
        # 发布请求事件
        event_bus = get_event_bus()
        await event_bus.publish(Event(
            event_type=EventType.LLM_REQUEST,
            source="ollama",
            data={"prompt": prompt}
        ))
        
        try:
            async with httpx.AsyncClient(timeout=120.0) as client:
                response = await client.post(
                    f"{self._base_url}/api/generate",
                    json={
                        "model": kwargs.get("model", self._config.model),
                        "prompt": prompt,
                        "stream": False,
                        "options": {
                            "temperature": kwargs.get("temperature", self._config.temperature),
                            "num_predict": kwargs.get("max_tokens", self._config.max_tokens),
                        }
                    }
                )
                response.raise_for_status()
                data = response.json()
            
            result = LLMResponse(
                content=data.get("response", ""),
                model=data.get("model", self._config.model),
                duration_ms=(datetime.now() - start).total_seconds() * 1000
            )
            
            # 发布响应事件
            await event_bus.publish(Event(
                event_type=EventType.LLM_RESPONSE,
                source="ollama",
                data={"content": result.content}
            ))
            
            return result
            
        except Exception as e:
            self._logger.error(f"Ollama 请求失败: {e}")
            return LLMResponse(
                content=f"错误: {str(e)}",
                model=self._config.model,
                duration_ms=(datetime.now() - start).total_seconds() * 1000
            )


def create_llm_provider(config: LLMConfig) -> Provider:
    """创建 LLM Provider"""
    if config.provider == "openai":
        return OpenAIProvider(config)
    elif config.provider == "anthropic":
        return AnthropicProvider(config)
    elif config.provider == "ollama":
        return OllamaProvider(config)
    else:
        raise ValueError(f"未知的 LLM provider: {config.provider}")
