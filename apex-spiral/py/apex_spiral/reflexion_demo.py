#!/usr/bin/env python3
"""
Reflexion Loop 演示
运行一个真实任务来演示评估器-优化器循环

任务：让生成器写一个 Python 函数，评估器检查函数是否正确实现
"""

import sys
import os

# 添加路径
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "py"))

from reflexion_loop import ReflexionLoop, mimo_llm


def demo_task():
    """演示任务：生成一个正确的 Python 函数"""
    
    task = """请用 Python 写一个函数，计算斐波那契数列第 n 项（使用循环，不是递归）。
要求：
1. 函数名为 fibonacci
2. 参数 n 为非负整数
3. 返回第 n 项的值
4. 必须有清晰的注释
5. 要处理 n=0 和 n=1 的边界情况
"""
    
    evaluator_prompt = """任务描述：
{task}

生成结果：
{result}

请严格评估上述 Python 代码是否满足所有要求：
1. 函数名为 fibonacci
2. 参数 n 为非负整数
3. 返回第 n 项的值
4. 有清晰的注释
5. 正确处理 n=0 和 n=1 的边界情况

如果完全满足所有要求：
<evaluation>PASS</evaluation>
<feedback>无</feedback>

如果有任何不满足：
<evaluation>FAIL</evaluation>
<feedback>具体说明哪个要求没有满足，以及如何修改</feedback>
"""
    
    generator_prompt = """任务：{task}

{context}

请生成满足所有要求的 Python 代码。
"""
    
    print("=" * 60)
    print("Reflexion Loop 演示")
    print("=" * 60)
    print(f"\n任务：{task[:100]}...\n")
    
    # 创建 ReflexionLoop 实例
    loop = ReflexionLoop(llm_func=mimo_llm)
    
    # 运行循环
    print("开始运行评估器-优化器循环...\n")
    print("-" * 40)
    
    result, history = loop.loop(
        task=task,
        evaluator_prompt=evaluator_prompt,
        generator_prompt=generator_prompt,
        max_attempts=3
    )
    
    # 输出结果
    print("-" * 40)
    print("\n循环结束！")
    print("=" * 60)
    
    summary = loop.summary()
    print(f"\n📊 总尝试次数: {summary['total_attempts']}")
    print(f"✅ 通过次数: {summary['passes']}")
    print(f"❌ 失败次数: {summary['fails']}")
    print(f"\n🎯 最终评估: {summary['final_evaluation']}")
    
    print("\n" + "=" * 60)
    print("最终生成结果:")
    print("=" * 60)
    print(result)
    
    print("\n" + "=" * 60)
    print("历史记录:")
    print("=" * 60)
    for record in history:
        print(f"\n--- 尝试 #{record.attempt_number} ---")
        print(f"评估: {record.evaluation}")
        if record.feedback:
            print(f"反馈: {record.feedback}")
    
    print("\n" + "=" * 60)
    print("完成")
    print("=" * 60)
    
    return result, summary


if __name__ == "__main__":
    # 检查 API key
    if not os.environ.get("MIMO_API_KEY"):
        print("⚠️  警告：MIMO_API_KEY 环境变量未设置")
        print("请先设置：export MIMO_API_KEY='your-api-key'")
        print("\n将使用模拟模式演示...")
        
        # 模拟模式
        def mock_llm(prompt: str) -> str:
            print(f"[模拟 LLM 调用]\nPrompt 长度: {len(prompt)} 字符")
            if "evaluation" in prompt.lower():
                return "<evaluation>PASS</evaluation>\n<feedback>无</feedback>"
            return "# Fibonacci function\ndef fibonacci(n):\n    if n <= 0:\n        return 0\n    if n == 1:\n        return 1\n    a, b = 0, 1\n    for _ in range(2, n + 1):\n        a, b = b, a + b\n    return b"
        
        loop = ReflexionLoop(llm_func=mock_llm)
        
        task = "请用 Python 写一个函数，计算斐波那契数列第 n 项"
        result, history = loop.loop(task, max_attempts=2)
        
        print("\n模拟模式结果:")
        print(result)
    else:
        demo_task()
