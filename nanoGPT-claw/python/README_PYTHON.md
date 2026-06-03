# NanoGPT-Claw Python 集成层

基于事件驱动架构的 Agent 系统，使用官方 SDK 集成飞书和 LLM。

## 架构特点

- **事件驱动**：基于 EventBus 的松耦合架构
- **工具系统**：可扩展的 Tool 注册机制
- **Provider 抽象**：统一的 LLM Provider 接口
- **官方 SDK**：使用 lark-oapi-sdk 等官方库
- **分层设计**：Rust 核心 + Python 集成层（快速开发）

## 快速开始

### 1. 安装依赖

```bash
cd python
pip install -r requirements.txt
```

### 2. 配置环境

复制示例配置并填入真实值：

```bash
cp .env.example .env
# 编辑 .env 文件
```

必须配置的内容：
- `LLM_PROVIDER` 和对应的 API Key（OpenAI/Anthropic/Ollama）
- （可选）飞书 App ID 和 Secret

### 3. 使用方式

#### 交互式聊天

```bash
python main.py repl
```

#### 单轮对话

```bash
python main.py chat "你好，介绍一下你自己"
```

#### 启动飞书机器人

```bash
python main.py start
```

## 项目结构

```
python/
├── main.py                      # 主入口
├── requirements.txt             # Python 依赖
├── .env.example                # 配置示例
├── core/
│   ├── __init__.py
│   ├── config.py               # 配置管理
│   ├── logging.py              # 日志系统
│   ├── exceptions.py           # 异常定义
│   ├── event_bus.py            # 事件总线 + 工具/Provider 系统
│   └── agent.py                # Agent 核心
└── integrations/
    ├── __init__.py
    ├── llm.py                  # LLM 集成 (OpenAI/Anthropic/Ollama)
    └── feishu_integration.py   # 飞书集成 (官方 SDK)
```

## 核心模块

### EventBus（事件总线）

事件发布订阅系统，同时管理 Tools 和 Providers：

```python
from core.event_bus import get_event_bus, Event, EventType

bus = get_event_bus()

# 注册工具
class MyTool(Tool):
    name = "my_tool"
    description = "我的工具"
    parameters = {"arg": "参数说明"}
    
    async def execute(self, arg: str = "", **kwargs) -> ToolResult:
        return ToolResult(success=True, output=f"处理结果: {arg}")

bus.register_tool(MyTool())

# 调用工具
result = await bus.call_tool("my_tool", arg="hello")

# 订阅事件
async def handler(event: Event):
    print(f"收到事件: {event.event_type}")

bus.subscribe(EventType.MESSAGE_RECEIVED, handler)

# 发布事件
await bus.publish(Event(
    event_type=EventType.MESSAGE_RECEIVED,
    source="test",
    data={"message": "Hello"}
))
```

### Agent（智能体）

```python
from core.agent import Agent
from core.config import get_config

config = get_config()
agent = Agent(config)

# 启动
await agent.start()

# 处理消息
response = await agent.process_message(user_id="user1", message="你好")
```

## 扩展开发

### 添加新的 LLM Provider

继承 `Provider` 类并实现 `complete` 方法：

```python
from core.event_bus import Provider

class MyProvider(Provider):
    name = "my_provider"
    
    async def complete(self, prompt: str, **kwargs) -> Any:
        # 实现你的 LLM 调用
        pass
```

### 添加新的 Tool

继承 `Tool` 类并实现 `execute` 方法：

```python
from core.event_bus import Tool, ToolResult

class WeatherTool(Tool):
    name = "get_weather"
    description = "获取天气"
    parameters = {"city": "城市名称"}
    
    async def execute(self, city: str = "", **kwargs) -> ToolResult:
        # 调用天气 API
        weather_data = await fetch_weather(city)
        return ToolResult(success=True, output=weather_data)
```

## 与 Rust 层的关系

Python 层负责：
- 快速集成第三方服务（飞书、LLM API 等）
- 事件驱动的业务逻辑
- 可扩展的工具系统

Rust 层负责：
- 高性能核心逻辑
- 内存安全和并发处理
- 守护进程管理

## 配置参考

### LLM 配置

```env
LLM_PROVIDER=openai          # openai | anthropic | ollama
LLM_MODEL=gpt-4o
OPENAI_API_KEY=sk-xxx
```

### 飞书配置

```env
FEISHU_APP_ID=cli_xxx
FEISHU_APP_SECRET=xxx
```

## 开发说明

这是按照你的要求构建的**分层融合架构**：
- 不预先分层，使用事件驱动
- 所有能力统一注册为 Tool/Provider
- 飞书直接使用官方 SDK
- Python 层快速开发，Rust 层高性能

## 下一步

1. 填入 `.env` 配置
2. 运行 `python main.py repl` 测试交互
3. 根据需要添加新的 Tools/Providers
4. （可选）启动飞书机器人：`python main.py start`
