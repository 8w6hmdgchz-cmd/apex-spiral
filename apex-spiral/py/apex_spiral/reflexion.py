"""
APEX Reflexion Module - 基于论文 arXiv:2303.11366
实现不更新权重的强化学习反思机制
"""

import time
from typing import Optional, Callable, Any, List
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime


class FeedbackType(Enum):
    """反馈信号类型"""
    SUCCESS = "success"
    FAILURE = "failure"
    PARTIAL = "partial"
    UNKNOWN = "unknown"


@dataclass
class Reflection:
    """反思记录"""
    timestamp: str
    task: str
    result: str
    feedback_type: FeedbackType
    reflection_text: str  # 自然语言反思
    lessons: List[str] = field(default_factory=list)
    next_action: str = ""


@dataclass
class ReflexionConfig:
    """Reflexion 配置"""
    max_attempts: int = 3
    phi_increment_on_failure: float = 0.05
    phi_increment_on_success: float = 0.01
    phi_decrement_on_repeated_failure: float = -0.02
    memory_capacity: int = 100


class ApexReflexion:
    """
    APEX 元认知核心：Reflexion Loop
    
    基于论文《Reflexion: Language Agents with Verbal Reinforcement Learning》
    
    核心机制：
    1. 执行任务
    2. 获取反馈
    3. 生成语言反思
    4. 存入记忆
    5. 用反思指导下一轮
    """
    
    def __init__(
        self,
        llm_func: Callable[[str], str],
        memory_func: Optional[Callable[[str, str], None]] = None,
        retrieve_func: Optional[Callable[[str, int], List[str]]] = None,
        config: Optional[ReflexionConfig] = None
    ):
        """
        Args:
            llm_func: LLM 调用函数，输入 prompt，返回 response
            memory_func: 记忆存储函数 (key, value) -> None
            retrieve_func: 记忆检索函数 (query, n) -> List[str]
            config: 配置
        """
        self.llm = llm_func
        self.save_memory = memory_func
        self.retrieve_memory = retrieve_func
        self.config = config or ReflexionConfig()
        
        # Φ 元认知初始值 (很低，需要通过反思提升)
        self.phi = 0.15
        
        # 反思历史
        self.reflections: List[Reflection] = []
        
        # 当前任务上下文
        self.current_task: str = ""
        self.attempt_count: int = 0
    
    def execute_with_reflection(
        self,
        task: str,
        execute_func: Callable[[str, List[str]], Any]
    ) -> Any:
        """
        带 Reflexion 的任务执行
        
        Args:
            task: 任务描述
            execute_func: 执行函数 (task, reflections) -> result
            
        Returns:
            result: 执行结果
        """
        self.current_task = task
        self.attempt_count = 0
        
        while self.attempt_count < self.config.max_attempts:
            # 1. 获取相关反思
            relevant_reflections = self._get_relevant_reflections(task)
            
            # 2. 执行
            result = execute_func(task, relevant_reflections)
            
            # 3. 评估反馈
            feedback = self._evaluate(result)
            
            # 4. 如果成功，结束
            if feedback.type == FeedbackType.SUCCESS:
                # 小幅提升 Φ
                self.phi = min(1.0, self.phi + self.config.phi_increment_on_success)
                return result
            
            # 5. 如果失败，反思
            if feedback.type == FeedbackType.FAILURE:
                reflection = self._generate_reflection(task, result, feedback)
                self._store_reflection(reflection)
                
                # 大幅提升 Φ (失败是学习机会)
                self.phi = min(1.0, self.phi + self.config.phi_increment_on_failure)
                
                # 用反思改进任务描述
                task = self._apply_reflection(task, reflection)
            
            self.attempt_count += 1
        
        # 达到最大尝试次数
        return result
    
    def _get_relevant_reflections(self, query: str) -> List[str]:
        """检索相关反思"""
        if not self.retrieve_memory:
            # 如果没有检索函数，使用本地记忆
            return [r.reflection_text for r in self.reflections[-5:]]
        
        # 使用外部检索 (Mem0)
        return self.retrieve_memory(query, n=5)
    
    def _evaluate(self, result: Any) -> FeedbackType:
        """评估结果，生成反馈"""
        prompt = f"""评估以下执行结果，返回类型：
- success: 任务成功完成
- failure: 任务明确失败
- partial: 部分成功
- unknown: 无法判断

结果: {result}

只返回一个词: success/failure/partial/unknown"""
        
        response = self.llm(prompt).strip().lower()
        
        if 'success' in response:
            return FeedbackType.SUCCESS
        elif 'failure' in response:
            return FeedbackType.FAILURE
        elif 'partial' in response:
            return FeedbackType.PARTIAL
        return FeedbackType.UNKNOWN
    
    def _generate_reflection(
        self,
        task: str,
        result: Any,
        feedback: FeedbackType
    ) -> Reflection:
        """生成语言反思"""
        prompt = f"""任务: {task}
结果: {result}

用自然语言详细反思：
1. 什么出错了？
2. 为什么会出错？
3. 下次应该怎么做？

格式：
[问题] ...
[原因] ...
[改进] ..."""
        
        reflection_text = self.llm(prompt)
        
        # 提取教训
        lessons = self._extract_lessons(reflection_text)
        
        return Reflection(
            timestamp=datetime.now().isoformat(),
            task=task,
            result=str(result),
            feedback_type=feedback,
            reflection_text=reflection_text,
            lessons=lessons
        )
    
    def _extract_lessons(self, reflection_text: str) -> List[str]:
        """从反思中提取教训"""
        prompt = f"""从以下反思中提取教训，格式为一行一个：
{reflection_text}

只输出教训列表，每行一个，不要其他内容。"""
        
        response = self.llm(prompt)
        return [line.strip() for line in response.split('\n') if line.strip()]
    
    def _store_reflection(self, reflection: Reflection) -> None:
        """存储反思到记忆"""
        self.reflections.append(reflection)
        
        # 如果有外部记忆函数，调用它
        if self.save_memory:
            key = f"reflexion_{reflection.timestamp}"
            self.save_memory(key, reflection.reflection_text)
        
        # 限制容量
        if len(self.reflections) > self.config.memory_capacity:
            self.reflections = self.reflections[-self.config.memory_capacity:]
    
    def _apply_reflection(self, task: str, reflection: Reflection) -> str:
        """将反思应用到任务描述"""
        prompt = f"""原始任务: {task}
反思: {reflection.reflection_text}

根据反思，改进任务描述，使其能够避免之前的错误。
只返回改进后的任务描述，不要其他内容。"""
        
        return self.llm(prompt)
    
    def get_phi(self) -> float:
        """获取当前 Φ 值"""
        return self.phi
    
    def set_phi(self, phi: float) -> None:
        """设置 Φ 值"""
        self.phi = max(0.0, min(1.0, phi))
    
    def summary(self) -> dict:
        """返回反思摘要"""
        return {
            "phi": self.phi,
            "total_reflections": len(self.reflections),
            "recent_reflections": [
                {
                    "timestamp": r.timestamp,
                    "feedback": r.feedback_type.value,
                    "preview": r.reflection_text[:100]
                }
                for r in self.reflections[-3:]
            ]
        }
