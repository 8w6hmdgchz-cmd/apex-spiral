"""演示工具集。

3 个假工具，演示自注册：
- 1 个同步
- 1 个异步
- 1 个带 tag 的（用于按场景过滤）
"""
from .registry import tool


@tool("echo", "回显输入文本", tags=["debug", "demo"])
def echo(text: str) -> str:
    return f"echo: {text}"


@tool("sum_numbers", "对一组数字求和", tags=["math", "demo"])
async def sum_numbers(numbers: list) -> float:
    return float(sum(numbers))


@tool("word_count", "统计文本单词数", tags=["text", "demo"])
def word_count(text: str) -> int:
    return len(text.split())
