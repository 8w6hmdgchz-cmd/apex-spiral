"""
APEX 核心机制 - 编译到璇玑能力
基于论文: Reflexion, Generative Agents, Voyager
"""

import sys
sys.path.insert(0, '/Users/lihongxin/.openclaw/workspace/apex-spiral/py')

from apex_spiral import ApexAgent, ApexAgentConfig, ApexReflexion, ApexMemoryStream

# ============================================================
# 璇玑核心能力初始化
# ============================================================

def init_apex_capability(mimo_llm_func):
    """
    初始化 APEX 核心能力
    
    Args:
        mimo_llm_func: MIMO LLM 调用函数
    """
    global _apex_agent, _reflexion, _memory_stream
    
    config = ApexAgentConfig(
        max_attempts=3,
        phi_initial=0.15,
        memory_capacity=500,
        reflection_threshold=10
    )
    
    _apex_agent = ApexAgent(
        llm_func=mimo_llm_func,
        config=config
    )
    
    _reflexion = _apex_agent.reflexion
    _memory_stream = _apex_agent.memory
    
    return _apex_agent


# ============================================================
# 能力接口
# ============================================================

def apex_execute(task: str) -> str:
    """
    执行任务（带 Reflexion）
    - 自动反思
    - 自动记忆
    - Φ 动态调整
    """
    result = _apex_agent.execute(task)
    return str(result)


def apex_think(prompt: str) -> str:
    """
    思考（带记忆上下文）
    - 检索相关记忆
    - 结合上下文回答
    """
    return _apex_agent.think(prompt)


def apex_remember(content: str, importance: float = 0.5) -> str:
    """
    添加记忆
    - 自动时序存储
    - 重要性标记
    """
    return _apex_agent.remember(content, importance)


def apex_recall(query: str, n: int = 5) -> list:
    """
    检索记忆
    - 相关性 + 时效性 + 重要性
    """
    return _apex_agent.recall(query, n=n)


def apex_observe() -> list:
    """
    主动观察
    - 时间感知
    - 异常检测
    """
    return _apex_agent.observe()


def apex_reflect() -> str:
    """
    触发高层反思
    - 合成见解
    - 更新 Φ
    """
    return _apex_agent.reflect()


def apex_phi() -> float:
    """
    获取当前 Φ 值
    """
    return _apex_agent.phi


def apex_status() -> str:
    """
    获取状态报告
    """
    return _apex_agent.status()


# ============================================================
# 全局实例
# ============================================================

_apex_agent = None
_reflexion = None
_memory_stream = None
