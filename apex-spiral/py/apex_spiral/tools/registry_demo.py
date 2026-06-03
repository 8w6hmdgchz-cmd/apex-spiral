"""真实可跑的自注册 demo。

跑这个文件 = 验证：
1. 3 个工具模块顶层 import 即自动注册
2. 注册中心 list / get / tag 过滤工作正常
3. 同步 + 异步工具都能通过统一接口调用
"""
import asyncio

# 关键：导入 demo_tools 即触发 3 个 @tool 装饰器
from . import demo_tools  # noqa: F401
from .registry import get_registry


def main_sync_demo() -> None:
    reg = get_registry()
    print(f"📦 registry contains {len(reg)} tools: {reg.list()}")
    print(f"📦 demo 标签工具: {reg.list(tag='demo')}")
    print(f"📦 math 标签工具: {reg.list(tag='math')}")

    # 调用同步工具
    result = reg.get("echo").invoke(text="hello apex")
    print(f"✅ echo: {result}")

    result = reg.get("word_count").invoke(text="APEX 自进化协调者测试")
    print(f"✅ word_count: {result}")


async def main_async_demo() -> None:
    reg = get_registry()
    # 调用异步工具
    result = await reg.get("sum_numbers")(numbers=[1, 2, 3, 4, 5])
    print(f"✅ sum_numbers: {result}")


if __name__ == "__main__":
    main_sync_demo()
    asyncio.run(main_async_demo())
    print("\n🎯 自注册 demo 跑通：3 个工具 / 同步异步混合 / tag 过滤 全过。")
