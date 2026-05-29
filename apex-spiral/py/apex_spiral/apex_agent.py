"""
APEX Agent - 整合 Reflexion + Memory Stream + Observation
基于论文实现完整的 AI Agent 架构
"""

from typing import Callable, Optional, List, Dict, Any
from dataclasses import dataclass

from .reflexion import ApexReflexion, ReflexionConfig
from .memory_stream import ApexMemoryStream, MemoryStreamConfig, MemoryType
from .observation import ApexObservation, ObservationConfig, Observation


@dataclass
class ApexAgentConfig:
    """APEX Agent 配置"""
    # Reflexion
    max_attempts: int = 3
    phi_initial: float = 0.15
    
    # Memory Stream
    memory_capacity: int = 1000
    reflection_threshold: int = 20
    
    # Observation
    observe_interval_seconds: int = 300


class ApexAgent:
    """
    APEX 智能体 - 完整架构
    
    整合三大核心模块：
    1. Reflexion - 自我反思
    2. MemoryStream - 时序记忆
    3. Observation - 主动感知
    """
    
    def __init__(
        self,
        llm_func: Callable[[str], str],
        embed_func: Optional[Callable[[str], List[float]]] = None,
        config: Optional[ApexAgentConfig] = None
    ):
        """
        Args:
            llm_func: LLM 调用函数
            embed_func: 向量化函数 (可选)
            config: 配置
        """
        self.llm = llm_func
        self.embed = embed_func
        self.config = config or ApexAgentConfig()
        
        # 初始化三大模块
        self.reflexion = ApexReflexion(
            llm_func=llm_func,
            config=ReflexionConfig(
                max_attempts=self.config.max_attempts,
                phi_increment_on_failure=0.05,
                phi_increment_on_success=0.01,
            )
        )
        
        self.memory = ApexMemoryStream(
            embed_func=embed_func,
            llm_func=llm_func,
            config=MemoryStreamConfig(
                max_size=self.config.memory_capacity,
                reflection_threshold=self.config.reflection_threshold
            )
        )
        
        self.observation = ApexObservation(
            llm_func=llm_func,
            memory_stream=self.memory,
            config=ObservationConfig(
                check_interval_seconds=self.config.observe_interval_seconds
            )
        )
        
        # Φ 元认知绑定到 reflexion
        self._phi = self.config.phi_initial
    
    @property
    def phi(self) -> float:
        """获取 Φ 元认知值"""
        return self.reflexion.get_phi()
    
    @phi.setter
    def phi(self, value: float) -> None:
        """设置 Φ 元认知值"""
        self.reflexion.set_phi(value)
        self._phi = value
    
    def execute(
        self,
        task: str,
        execute_func: Optional[Callable[[str, List[str]], Any]] = None
    ) -> Any:
        """
        执行任务（带 Reflexion）
        
        Args:
            task: 任务描述
            execute_func: 执行函数，如果为 None，使用默认执行
        """
        if execute_func is None:
            execute_func = self._default_execute
        
        # 使用 Reflexion 执行
        result = self.reflexion.execute_with_reflection(task, execute_func)
        
        # 添加到记忆
        self.memory.add(
            content=f"任务: {task}\n结果: {result}",
            memory_type=MemoryType.EXECUTION,
            importance=0.7
        )
        
        return result
    
    def _default_execute(self, task: str, reflections: List[str]) -> str:
        """默认执行：直接用 LLM"""
        prompt = f"任务: {task}"
        
        if reflections:
            prompt += f"\n\n参考反思:\n" + "\n".join([f"- {r}" for r in reflections])
        
        return self.llm(prompt)
    
    def observe(self) -> List[Observation]:
        """主动观察"""
        return self.observation.observe()
    
    def remember(self, content: str, importance: float = 0.5) -> str:
        """
        添加记忆
        
        Returns:
            memory_id
        """
        return self.memory.add(
            content=content,
            memory_type=MemoryType.OBSERVATION,
            importance=importance
        )
    
    def recall(self, query: str, n: int = 5) -> List[str]:
        """
        检索记忆
        
        Args:
            query: 查询
            n: 返回数量
        """
        memories = self.memory.retrieve(query, n=n)
        return [m.content for m in memories]
    
    def think(self, prompt: str) -> str:
        """
        思考（带记忆上下文）
        """
        # 检索相关记忆
        relevant = self.recall(prompt, n=3)
        
        # 构建思考 prompt
        context = ""
        if relevant:
            context = "\n\n相关记忆:\n" + "\n".join([f"- {r}" for r in relevant])
        
        full_prompt = f"{prompt}{context}"
        
        return self.llm(full_prompt)
    
    def reflect(self) -> str:
        """
        手动触发高层反思
        """
        insight = self.memory.synthesize_insight()
        if insight:
            return insight.content
        return "反思：经验不足，无法生成高层见解"
    
    def summary(self) -> Dict[str, Any]:
        """返回系统摘要"""
        return {
            "phi": self.phi,
            "reflexion": self.reflexion.summary(),
            "memory": self.memory.summary(),
            "observation": self.observation.summary()
        }
    
    def status(self) -> str:
        """返回状态报告"""
        s = self.summary()
        
        lines = [
            "=" * 40,
            "APEX Agent 状态",
            "=" * 40,
            f"Φ 元认知: {s['phi']:.2f}",
            f"反思次数: {s['reflexion']['total_reflections']}",
            f"记忆数量: {s['memory']['total_memories']}",
            f"见解数量: {s['memory']['insights']}",
            f"观察次数: {s['observation']['total_observations']}",
            "=" * 40
        ]
        
        return "\n".join(lines)
