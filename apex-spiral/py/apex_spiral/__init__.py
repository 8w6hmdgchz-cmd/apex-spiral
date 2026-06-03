"""
ApexSpiral - 璇玑帝国 APEX 框架 (V11-only)

V10 公式已删除（2026-06-02），V11 公式来自 hernandez42/apex-codex
V11 公式：ΔG = (C·Λ·Ω·τ)/(H·t) × Φ_SPARK × Φ_AUTONOMOUS
V11 强度是 V10 的 26 倍（×Φ_SPARK×Φ_AUTONOMOUS = 10.14 增强）

基于论文:
- Reflexion: arXiv:2303.11366
- Generative Agents: arXiv:2304.03442
- Voyager: arXiv:2305.16291
"""

__version__ = "0.3.0"
__author__ = "璇玑帝国"
__formula_version__ = "V11"

# V11 公式 - 移植自 hernandez42/apex-codex
from apex_spiral.v11_formula import (
    XuanjiFormula,
    FormulaParams,
    DeltaGResult,
    DeltaGCalculator,
    FormulaConstants,
    params_from_apex_mem_state,
    compare_v10_v11,
)

# 框架模块（保留 V10 之外的通用组件）
from apex_spiral.reflexion import ApexReflexion, ReflexionConfig, Reflection, FeedbackType
from apex_spiral.memory_stream import ApexMemoryStream, MemoryStreamConfig, Memory, MemoryType
from apex_spiral.observation import ApexObservation, ObservationConfig, Observation, ObservationType
from apex_spiral.apex_agent import ApexAgent, ApexAgentConfig
from apex_spiral.apex_memory_bridge import (
    ApexMemoryBridge,
    MemoryEntry as SigmaMemoryEntry,
    MemoryType as SigmaMemoryType,
    SuperMemoryParams,
    add_memory_entry,
    calculate_sigma_memory,
)
# LongMemEval 评估器 - 移植自 hernandez42/xuanji
from apex_spiral.evaluator import (
    MemoryEvaluator,
    MemoryConsolidator,
    ConflictResolver,
    evaluate_apex_mem,
)

__all__ = [
    # V11 公式
    "XuanjiFormula",
    "FormulaParams",
    "DeltaGResult",
    "DeltaGCalculator",
    "FormulaConstants",
    "params_from_apex_mem_state",
    "compare_v10_v11",

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
    "ApexMemoryBridge",
    "SigmaMemoryEntry",
    "SigmaMemoryType",
    "SuperMemoryParams",
    "add_memory_entry",
    "calculate_sigma_memory",

    # LongMemEval 评估器
    "MemoryEvaluator",
    "MemoryConsolidator",
    "ConflictResolver",
    "evaluate_apex_mem",

    # 版本
    "__version__",
    "__formula_version__",
]
