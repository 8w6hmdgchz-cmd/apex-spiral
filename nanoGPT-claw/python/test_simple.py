#!/usr/bin/env python3
"""
简单的核心模块测试脚本
"""
import sys
from pathlib import Path

# 添加当前目录到路径
sys.path.insert(0, str(Path(__file__).parent))

print("=" * 60)
print("测试 NanoGPT-Claw Python 核心模块")
print("=" * 60)

try:
    print("\n[1/4] 导入 core.config...")
    from core.config import Config, get_config
    print("   ✓ 成功")
    
    print("\n[2/4] 导入 core.event_bus...")
    from core.event_bus import EventBus, Event, EventType, get_event_bus
    print("   ✓ 成功")
    
    print("\n[3/4] 导入 integrations.llm (直接)...")
    import importlib.util
    llm_path = Path(__file__).parent / "integrations" / "llm.py"
    spec = importlib.util.spec_from_file_location("llm", llm_path)
    llm_module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(llm_module)
    print("   ✓ 成功")
    
    print("\n[4/4] 导入 integrations.feishu_integration (直接)...")
    feishu_path = Path(__file__).parent / "integrations" / "feishu_integration.py"
    spec = importlib.util.spec_from_file_location("feishu", feishu_path)
    feishu_module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(feishu_module)
    print("   ✓ 成功")
    
    print("\n" + "=" * 60)
    print("核心模块导入测试通过!")
    print("=" * 60)
    
    # 测试创建基本对象
    print("\n\n测试创建基本对象...")
    
    print("\n[1/2] 创建 EventBus...")
    bus = get_event_bus()
    
    # 测试工具注册系统
    from core.event_bus import Tool, ToolResult
    
    class TestTool(Tool):
        name = "test_tool"
        description = "测试工具"
        parameters = {"input": "输入值"}
        
        async def execute(self, input: str = "", **kwargs) -> ToolResult:
            return ToolResult(success=True, output=f"处理: {input}")
    
    bus.register_tool(TestTool())
    tools = bus.list_tools()
    print(f"   ✓ 成功, 工具列表: {[t['name'] for t in tools]}")
    
    print("\n[2/2] 创建 Config...")
    config = get_config()
    print(f"   ✓ 成功, LLM 提供商: {config.llm.provider}")
    
    print("\n" + "=" * 60)
    print("系统结构完整!")
    print("=" * 60)
    
    print("""\n项目结构:
/workspace/nanoGPT-claw/python/
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

使用方法:
1. 复制 .env.example 为 .env 并填入配置
2. 运行: python main.py --help
3. 交互式聊天: python main.py repl
4. 启动飞书 Agent: python main.py start

核心特性:
✓ 事件驱动架构 (EventBus)
✓ 可扩展的工具系统
✓ LLM Provider 抽象层
✓ 飞书官方 SDK 集成
✓ 支持 OpenAI/Anthropic/Ollama
""")
    
except Exception as e:
    print(f"\n✗ 错误: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)
