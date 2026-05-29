"""
ApexSpiral - 璇玑帝国 APEX 终极闭环进化公式 Python 实现

基于论文:
- Reflexion: arXiv:2303.11366
- Generative Agents: arXiv:2304.03442
- Voyager: arXiv:2305.16291
"""

__version__ = "0.2.0"
__author__ = "璇玑帝国"

# 核心计算
from apex_spiral.core import ApexCalculator, ApexParams

# 新增核心机制
from apex_spiral.reflexion import ApexReflexion, ReflexionConfig, Reflection, FeedbackType
from apex_spiral.memory_stream import ApexMemoryStream, MemoryStreamConfig, Memory, MemoryType
from apex_spiral.observation import ApexObservation, ObservationConfig, Observation, ObservationType
from apex_spiral.apex_agent import ApexAgent, ApexAgentConfig

__all__ = [
    # 核心计算
    "ApexCalculator",
    "ApexParams",
    
    # Reflexion
    "ApexReflexion",
    "ReflexionConfig",
    "Reflection",
    "FeedbackType",
    
    # Memory Stream
    "ApexMemoryStream",
    "MemoryStreamConfig",
    "Memory",
    "MemoryType",
    
    # Observation
    "ApexObservation",
    "ObservationConfig",
    "Observation",
    "ObservationType",
    
    # Agent
    "ApexAgent",
    "ApexAgentConfig",
    
    # 版本
    "__version__"
]
