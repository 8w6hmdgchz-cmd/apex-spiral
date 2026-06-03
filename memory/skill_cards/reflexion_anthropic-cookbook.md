# Anthropic Cookbook — 评估器-优化器循环

## 触发场景
任务结果可被客观评价、有明确改进空间时自动触发。适用于代码、报告、方案等可迭代优化的任务。

## 关键代码片段

evaluator_optimizer.ipynb
```python
# L52-70 核心循环
def loop(task, evaluator_prompt, generator_prompt):
    memory = []
    thoughts, result = generate(generator_prompt, task)
    memory.append(result)
    while True:
        evaluation, feedback = evaluate(evaluator_prompt, result, task)
        if evaluation == "PASS":           # 评估通过 → 退出循环
            return result, chain_of_thought
        # 失败 → 带上历史反馈重生成
        context = "\n".join(["Previous attempts:",
            *[f"- {m}" for m in memory], f"\nFeedback: {feedback}"])
        thoughts, result = generate(generator_prompt, task, context)
        memory.append(result)

# util.py L19-34 XML解析
def extract_xml(text, tag):
    match = re.search(f"<{tag}>(.*?)</{tag}>", text, re.DOTALL)
    return match.group(1) if match else ""
```

## 踩坑提醒
- **反思后真改的机制**：每次失败的 context 会累积历史 attempts，生成器必须读全部历史才能不重复犯错；不要只传最新一条 feedback。
- 评估器和生成器必须分开调用（两个独立 LLM 调用），不能合并成一次，否则自我评分会偏高。

## 落地到 APEX 的具体路径
- **文件**：`APEX/agents/reflexion_loop.py`
- **工作量**：约 80 行代码，直接复用 `generate + evaluate + loop` 三函数结构
- **接入点**：在 APEX 的任务规划节点后，插入 reflexion_loop 包装层，评估失败时自动回到规划节点重试（最多 N 轮）
