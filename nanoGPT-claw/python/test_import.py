#!/usr/bin/env python3
"""
简单的导入测试脚本，验证模块结构是否正确
"""
import sys
from pathlib import Path

# 添加当前目录到路径
sys.path.insert(0, str(Path(__file__).parent))

print("=" * 60)
print("测试 NanoGPT-Claw Python 模块导入")
print("=" * 60)

try:
    print("\n[1/5] 导入 core.config...")
    from core.config import Config, get_config
    print("   ✓ 成功")
    
    print("\n[2/5] 导入 core.event_bus...")
    from core.event_bus import EventBus, Event, EventType, get_event_bus
    print("   ✓ 成功")
    
    print("\n[3/5] 导入 core.agent...")
    from core.agent import Agent
    print("   ✓ 成功")
    
    print("\n[4/5] 导入 integrations.llm...")
    from integrations.llm import create_llm_provider, LLMMessage
    print("   ✓ 成功")
    
    print("\n[5/5] 导入 integrations.feishu_integration...")
    from integrations.feishu_integration import FeishuIntegration, FeishuMessage
    print("   ✓ 成功")
    
    print("\n" + "=" * 60)
    print("所有模块导入测试通过!")
    print("=" * 60)
    
    # 测试创建基本对象
    print("\n\n测试创建基本对象...")
    
    print("\n[1/3] 创建 EventBus...")
    bus = get_event_bus()
    print(f"   ✓ 成功, 工具列表: {[t['name'] for t in bus.list_tools()]}")
    
    print("\n[2/3] 创建 Config...")
    config = get_config()
    print(f"   ✓ 成功, LLM 提供商: {config.llm.provider}")
    
    print("\n[3/3] 测试事件创建...")
    event = Event(
        event_type=EventType.MESSAGE_RECEIVED,
        source="test",
        data={"message": "Hello World"}
    )
    print(f"   ✓ 成功, 事件类型: {event.event_type}")
    
    print("\n" + "=" * 60)
    print("对象创建测试通过!")
    print("=" * 60)
    
    print("""\n系统结构总结:
- core/config.py: 配置管理
- core/event_bus.py: 事件总线、工具/Provider 管理
- core/agent.py: Agent 核心逻辑
- integrations/llm.py: LLM 集成 (OpenAI/Anthropic/Ollama)
- integrations/feishu_integration.py: 飞书集成 (使用官方 SDK)

使用方法:
1. 复制 .env.example 为 .env 并填入配置
2. 运行: python main.py --help
3. 交互式聊天: python main.py repl
4. 启动飞书 Agent: python main.py start
""")
    
except Exception as e:
    print(f"\n✗ 错误: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)
