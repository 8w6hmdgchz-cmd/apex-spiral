"""
Reflexion Loop - 评估器-优化器循环
基于 Anthropic Cookbook: evaluator_optimizer.ipynb

核心机制：
1. 生成器(GENERATOR)：根据任务和上下文生成结果
2. 评估器(EVALUATOR)：独立评估结果，返回 PASS/FAIL + 具体反馈
3. 循环：失败时累积历史 attempts + feedback 重生成

关键要求：
- 评估器与生成器必须分离（两个独立 LLM 调用）
- 历史 attempts 必须全量传递（不只是最新一条 feedback）
"""

import re
import os
from typing import Callable, Optional, List, Dict, Any, Tuple
from dataclasses import dataclass, field
from datetime import datetime


# =============================================================================
# 小米 MIMO LLM 调用
# =============================================================================

MIMO_API_URL = "https://api.mimo.ai/v1/chat/completions"
MIMO_MODEL = "mimo-v2.5-pro"
MIMO_API_KEY = os.environ.get("MIMO_API_KEY", "")


def mimo_llm(prompt: str, system: str = "你是一个有用的AI助手。") -> str:
    """
    调用小米 MIMO API 的便捷函数
    
    Args:
        prompt: 用户 prompt
        system: 系统提示（可选）
    
    Returns:
        LLM 响应文本
    """
    import json
    import urllib.request
    
    if not MIMO_API_KEY:
        raise ValueError("MIMO_API_KEY environment variable not set")
    
    payload = {
        "model": MIMO_MODEL,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.7,
        "max_tokens": 2048,
    }
    
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        MIMO_API_URL,
        data=data,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {MIMO_API_KEY}",
            "api-key": MIMO_API_KEY,
        },
        method="POST"
    )
    
    with urllib.request.urlopen(req, timeout=60) as resp:
        result = json.loads(resp.read().decode("utf-8"))
        return result["choices"][0]["message"]["content"]


# =============================================================================
# XML 解析工具（来自 Anthropic Cookbook util.py）
# =============================================================================

def extract_xml(text: str, tag: str) -> str:
    """从文本中提取指定 XML 标签内容"""
    match = re.search(f"<{tag}>(.*?)</{tag}>", text, re.DOTALL)
    return match.group(1).strip() if match else ""


# =============================================================================
# 数据结构
# =============================================================================

@dataclass
class AttemptRecord:
    """尝试记录"""
    attempt_number: int
    result: str
    evaluation: str  # "PASS" or "FAIL"
    feedback: str
    timestamp: str = field(default_factory=lambda: datetime.now().isoformat())


@dataclass
class ReflexionLoopConfig:
    """Reflexion Loop 配置"""
    max_attempts: int = 3
    temperature: float = 0.7
    max_tokens: int = 2048
    
    # 评估器 prompt 模板
    evaluator_system: str = (
        "你是一个严谨的评审员。你的职责是客观评估任务结果的完成质量。"
        "返回格式：<evaluation>PASS</evaluation>（或 FAIL）"
        "如果任务完全符合要求返回 PASS，否则返回 FAIL 并在 <feedback> 中说明具体问题。"
    )
    
    # 生成器 prompt 模板
    generator_system: str = (
        "你是一个高效的助手。根据任务描述和历史尝试记录，生成最优质的答案。"
        "如果提供了历史尝试和反馈，必须认真分析并避免重复同样的错误。"
    )


# =============================================================================
# 核心循环
# =============================================================================

class ReflexionLoop:
    """
    评估器-优化器循环（Reflexion Loop）
    
    流程：
    1. 生成器根据任务生成结果
    2. 评估器独立评估结果
    3. 如果 PASS → 返回结果
    4. 如果 FAIL → 累积历史 attempts + feedback，重生成
    5. 最多重试 max_attempts 次
    """
    
    def __init__(
        self,
        llm_func: Optional[Callable[[str], str]] = None,
        config: Optional[ReflexionLoopConfig] = None
    ):
        """
        Args:
            llm_func: LLM 调用函数，输入 prompt，返回 response
                     如果为 None，使用默认的 mimo_llm
            config: 配置
        """
        self.llm = llm_func or mimo_llm
        self.config = config or ReflexionLoopConfig()
        
        # 历史记录
        self.history: List[AttemptRecord] = []
    
    def generate(
        self,
        generator_prompt: str,
        task: str,
        context: str = ""
    ) -> str:
        """
        调用生成器 LLM
        
        Args:
            generator_prompt: 生成器专用 prompt 模板
            task: 任务描述
            context: 上下文（历史 attempts + feedback）
        
        Returns:
            生成结果
        """
        full_prompt = generator_prompt.format(task=task, context=context)
        return self.llm(full_prompt)
    
    def evaluate(
        self,
        evaluator_prompt: str,
        result: str,
        task: str
    ) -> Tuple[str, str]:
        """
        调用评估器 LLM（独立调用）
        
        Args:
            evaluator_prompt: 评估器专用 prompt 模板
            result: 生成结果
            task: 任务描述
        
        Returns:
            (evaluation, feedback)
            - evaluation: "PASS" or "FAIL"
            - feedback: 具体反馈信息
        """
        full_prompt = evaluator_prompt.format(task=task, result=result)
        response = self.llm(full_prompt)
        
        # 解析 XML
        evaluation = extract_xml(response, "evaluation").upper()
        feedback = extract_xml(response, "feedback")
        
        # 兜底解析
        if evaluation not in ("PASS", "FAIL"):
            if "pass" in response.lower():
                evaluation = "PASS"
            else:
                evaluation = "FAIL"
                if not feedback:
                    feedback = response[:200]
        
        return evaluation, feedback
    
    def build_context(self) -> str:
        """
        构建历史上下文（全量传递，不是只传最新一条）
        
        Returns:
            格式化的历史尝试记录
        """
        if not self.history:
            return ""
        
        lines = ["[历史尝试记录]"]
        for record in self.history:
            lines.append(f"\n--- 尝试 #{record.attempt_number} ---")
            lines.append(f"结果: {record.result[:500]}...")
            lines.append(f"评估: {record.evaluation}")
            if record.feedback:
                lines.append(f"反馈: {record.feedback}")
        
        return "\n".join(lines)
    
    def loop(
        self,
        task: str,
        evaluator_prompt: Optional[str] = None,
        generator_prompt: Optional[str] = None,
        max_attempts: Optional[int] = None
    ) -> Tuple[str, List[AttemptRecord]]:
        """
        核心循环：评估器-优化器
        
        Args:
            task: 任务描述
            evaluator_prompt: 评估器 prompt 模板，包含 {task} 和 {result} 占位符
            generator_prompt: 生成器 prompt 模板，包含 {task} 和 {context} 占位符
            max_attempts: 最大尝试次数（覆盖配置）
        
        Returns:
            (final_result, history)
        """
        # 使用默认 prompt 或自定义
        eval_prompt = evaluator_prompt or self._default_evaluator_prompt()
        gen_prompt = generator_prompt or self._default_generator_prompt()
        max_att = max_attempts or self.config.max_attempts
        
        # 清空历史
        self.history = []
        
        # 第一轮：直接生成
        attempt_num = 1
        context = ""
        
        result = self.generate(gen_prompt, task, context)
        
        # 评估循环
        while True:
            evaluation, feedback = self.evaluate(eval_prompt, result, task)
            
            # 记录历史
            record = AttemptRecord(
                attempt_number=attempt_num,
                result=result,
                evaluation=evaluation,
                feedback=feedback
            )
            self.history.append(record)
            
            # 评估通过
            if evaluation == "PASS":
                return result, self.history
            
            # 达到最大尝试次数
            if attempt_num >= max_att:
                return result, self.history
            
            # 构建全量上下文用于重生成
            context = self.build_context()
            
            # 重生成
            attempt_num += 1
            result = self.generate(gen_prompt, task, context)
    
    def _default_evaluator_prompt(self) -> str:
        """默认评估器 prompt"""
        return (
            "任务: {task}\n\n"
            "生成结果: {result}\n\n"
            "请评估上述结果是否完全满足了任务要求。\n"
            "完全满足返回：\n"
            "<evaluation>PASS</evaluation>\n"
            "<feedback>无</feedback>\n\n"
            "未满足返回：\n"
            "<evaluation>FAIL</evaluation>\n"
            "<feedback>具体说明哪些地方没有满足任务要求，以及如何改进</feedback>"
        )
    
    def _default_generator_prompt(self) -> str:
        """默认生成器 prompt"""
        return (
            "任务: {task}\n\n"
            "{context}\n\n"
            "请根据任务要求生成答案。"
        )
    
    def summary(self) -> Dict[str, Any]:
        """返回循环摘要"""
        return {
            "total_attempts": len(self.history),
            "final_result": self.history[-1].result if self.history else "",
            "final_evaluation": self.history[-1].evaluation if self.history else "N/A",
            "passes": sum(1 for r in self.history if r.evaluation == "PASS"),
            "fails": sum(1 for r in self.history if r.evaluation == "FAIL"),
            "records": [
                {
                    "attempt": r.attempt_number,
                    "evaluation": r.evaluation,
                    "feedback_preview": r.feedback[:100] if r.feedback else ""
                }
                for r in self.history
            ]
        }


# =============================================================================
# 便捷函数
# =============================================================================

def reflexion_loop(
    task: str,
    llm_func: Optional[Callable[[str], str]] = None,
    max_attempts: int = 3
) -> Tuple[str, List[AttemptRecord]]:
    """
    便捷函数：直接运行一个 reflexion loop
    
    Args:
        task: 任务描述
        llm_func: LLM 调用函数
        max_attempts: 最大尝试次数
    
    Returns:
        (final_result, history)
    """
    loop = ReflexionLoop(llm_func=llm_func)
    return loop.loop(task, max_attempts=max_attempts)
