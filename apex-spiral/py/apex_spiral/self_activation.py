"""
APEX Self-Activation Driver — 状态驱动的自主决策引擎

核心原则：不是定时触发，而是根据系统状态自动决定下一步动作。
当基线数据达到阈值时，触发对应行动。

阈值规则（来自用户 2026-06-02 指令）：
- 通过率 < 0.5     → trigger_reflexion（触发反思循环重试）
- 反思成功率 < 0.3  → write_mechanism_card（写机制卡片）
- 失败案例 > 5条   → rewrite_prompt（自动改提示词）
- 元认知分数下降   → pause + alert（暂停并报警）
"""

import json
import os
from pathlib import Path
from dataclasses import dataclass, asdict, field
from datetime import datetime
from typing import Optional, List, Dict, Any
from enum import Enum


# ============================================================================
# 数据结构
# ============================================================================

class ActionType(Enum):
    """可触发的动作类型"""
    TRIGGER_REFLEXION = "trigger_reflexion"
    WRITE_MECHANISM_CARD = "write_mechanism_card"
    REWRITE_PROMPT = "rewrite_prompt"
    PAUSE_AND_ALERT = "pause_and_alert"
    OBSERVE = "observe"  # 一切正常，继续观察


@dataclass
class Action:
    """一次决策动作"""
    action_type: ActionType
    reason: str
    timestamp: str = field(default_factory=lambda: datetime.now().isoformat())
    metrics_snapshot: Dict[str, Any] = field(default_factory=dict)
    priority: int = 0  # 0=低, 1=中, 2=高

    def to_dict(self) -> dict:
        return {
            "action_type": self.action_type.value,
            "reason": self.reason,
            "timestamp": self.timestamp,
            "metrics_snapshot": self.metrics_snapshot,
            "priority": self.priority
        }


@dataclass
class BaselineMetrics:
    """基线评测指标"""
    pass_rate: float           # 通过率
    total_tasks: int           # 总任务数
    passed_tasks: int          # 通过任务数
    failed_tasks: int          # 失败任务数
    reflexion_success_rate: float  # 反思成功率
    total_reflexions: int      # 总反思次数
    successful_reflexions: int  # 成功反思次数
    unclosed_failures: int     # 未闭环失败数
    metacognition_score: float # 元认知分数 (Φ)
    metacognition_history: List[float] = field(default_factory=list)  # Φ 历史
    delta_g: float = 0.0       # 进化增益
    phi: float = 0.0           # 当前 Φ 值

    @classmethod
    def from_dict(cls, data: dict) -> "BaselineMetrics":
        return cls(
            pass_rate=data.get("pass_rate", 0.0),
            total_tasks=data.get("total_tasks", 0),
            passed_tasks=data.get("passed_tasks", 0),
            failed_tasks=data.get("failed_tasks", 0),
            reflexion_success_rate=data.get("reflexion_success_rate", 0.0),
            total_reflexions=data.get("total_reflexions", 0),
            successful_reflexions=data.get("successful_reflexions", 0),
            unclosed_failures=data.get("unclosed_failures", 0),
            metacognition_score=data.get("metacognition_score", 0.0),
            metacognition_history=data.get("metacognition_history", []),
            delta_g=data.get("delta_g", 0.0),
            phi=data.get("phi", 0.0)
        )

    def to_dict(self) -> dict:
        return asdict(self)


# ============================================================================
# 异常类
# ============================================================================

class SelfActivationError(Exception):
    """自激活引擎异常"""
    pass


class BaselineLoadError(SelfActivationError):
    """基线数据加载失败"""
    pass


class FailureLogLoadError(SelfActivationError):
    """失败日志加载失败"""
    pass


# ============================================================================
# 核心引擎
# ============================================================================

class SelfActivator:
    """
    APEX 自激活驱动引擎

    根据基线数据自动决定下一步动作，不依赖定时任务。

    决策阈值（可配置）：
    - pass_rate_threshold: 通过率阈值（默认 0.5）
    - reflexion_success_threshold: 反思成功率阈值（默认 0.3）
    - unclosed_failure_threshold: 未闭环失败阈值（默认 5）
    - metacognition_decline_threshold: 元认知下降阈值（默认 0.05）
    """

    def __init__(
        self,
        baseline_path: str,
        failure_log_path: Optional[str] = None,
        activation_log_dir: Optional[str] = None,
        pass_rate_threshold: float = 0.5,
        reflexion_success_threshold: float = 0.3,
        unclosed_failure_threshold: int = 5,
        metacognition_decline_threshold: float = 0.05,
    ):
        """
        Args:
            baseline_path: 基线数据 JSON 文件路径
            failure_log_path: 失败日志 JSON 文件路径（可选）
            activation_log_dir: 决策日志目录（默认 baseline_path 同目录）
            pass_rate_threshold: 通过率阈值
            reflexion_success_threshold: 反思成功率阈值
            unclosed_failure_threshold: 未闭环失败阈值
            metacognition_decline_threshold: 元认知下降阈值
        """
        self.baseline_path = Path(baseline_path)
        self.failure_log_path = Path(failure_log_path) if failure_log_path else None

        # 日志目录：默认放在基线数据同目录的 memory 子目录
        if activation_log_dir:
            self.log_dir = Path(activation_log_dir)
        else:
            self.log_dir = self.baseline_path.parent / "memory"
        self.log_dir.mkdir(parents=True, exist_ok=True)

        self.activation_log_path = self.log_dir / "activation_log.jsonl"

        # 阈值配置
        self.pass_rate_threshold = pass_rate_threshold
        self.reflexion_success_threshold = reflexion_success_threshold
        self.unclosed_failure_threshold = unclosed_failure_threshold
        self.metacognition_decline_threshold = metacognition_decline_threshold

        # 运行时状态
        self.baseline: Optional[BaselineMetrics] = None
        self.failures: List[Dict[str, Any]] = []
        self._last_metacognition_score: Optional[float] = None

    # ------------------------------------------------------------------------
    # 数据加载
    # ------------------------------------------------------------------------

    def load_baseline(self) -> BaselineMetrics:
        """加载基线数据"""
        if not self.baseline_path.exists():
            raise BaselineLoadError(f"基线文件不存在: {self.baseline_path}")

        try:
            with open(self.baseline_path, "r", encoding="utf-8") as f:
                data = json.load(f)
        except json.JSONDecodeError as e:
            raise BaselineLoadError(f"基线文件 JSON 解析失败: {e}")

        self.baseline = BaselineMetrics.from_dict(data)
        return self.baseline

    def load_failure_log(self) -> List[Dict[str, Any]]:
        """加载失败日志"""
        if not self.failure_log_path:
            self.failures = []
            return self.failures

        if not self.failure_log_path.exists():
            self.failures = []
            return self.failures

        try:
            with open(self.failure_log_path, "r", encoding="utf-8") as f:
                self.failures = json.load(f)
        except json.JSONDecodeError:
            self.failures = []

        return self.failures

    def refresh(self) -> BaselineMetrics:
        """重新加载所有数据"""
        baseline = self.load_baseline()
        self.load_failure_log()
        return baseline

    # ------------------------------------------------------------------------
    # 核心决策逻辑
    # ------------------------------------------------------------------------

    def decide(self) -> Action:
        """
        核心决策函数：根据当前状态决定下一步动作。

        优先级（从高到低）：
        1. 元认知分数下降 → pause + alert（最高优先级）
        2. 通过率 < 阈值 → trigger_reflexion
        3. 反思成功率 < 阈值 → write_mechanism_card
        4. 未闭环失败 > 阈值 → rewrite_prompt
        5. 其他一切正常 → observe

        Returns:
            Action: 决定执行的动作
        """
        if self.baseline is None:
            self.refresh()

        b = self.baseline
        metrics = b.to_dict()

        # === 最高优先级：元认知分数下降 ===
        if self._is_metacognition_declining():
            return Action(
                action_type=ActionType.PAUSE_AND_ALERT,
                reason=f"元认知分数下降: {_fmt(b.metacognition_score)} (上次: {_fmt(self._last_metacognition_score)})",
                metrics_snapshot=metrics,
                priority=2
            )

        # === 优先级 2：通过率过低 ===
        if b.pass_rate < self.pass_rate_threshold:
            return Action(
                action_type=ActionType.TRIGGER_REFLEXION,
                reason=f"通过率 {b.pass_rate:.2%} < {self.pass_rate_threshold:.0%}",
                metrics_snapshot=metrics,
                priority=1
            )

        # === 优先级 3：反思成功率过低 ===
        if b.reflexion_success_rate < self.reflexion_success_threshold:
            return Action(
                action_type=ActionType.WRITE_MECHANISM_CARD,
                reason=f"反思成功率 {b.reflexion_success_rate:.2%} < {self.reflexion_success_threshold:.0%}",
                metrics_snapshot=metrics,
                priority=1
            )

        # === 优先级 4：未闭环失败过多 ===
        if b.unclosed_failures > self.unclosed_failure_threshold:
            return Action(
                action_type=ActionType.REWRITE_PROMPT,
                reason=f"未闭环失败 {b.unclosed_failures} 条 > {self.unclosed_failure_threshold} 条",
                metrics_snapshot=metrics,
                priority=1
            )

        # === 默认：一切正常，继续观察 ===
        return Action(
            action_type=ActionType.OBSERVE,
            reason="各项指标正常，继续观察",
            metrics_snapshot=metrics,
            priority=0
        )

    def _is_metacognition_declining(self) -> bool:
        """检查元认知分数是否下降"""
        current = self.baseline.metacognition_score

        if not self.baseline.metacognition_history:
            # 没有历史记录，记录当前值
            self._last_metacognition_score = current
            return False

        # 取最近一次历史值比较
        last = self.baseline.metacognition_history[-1]
        decline = last - current

        if decline > self.metacognition_decline_threshold:
            self._last_metacognition_score = last
            return True

        self._last_metacognition_score = last
        return False

    # ------------------------------------------------------------------------
    # 动作执行
    # ------------------------------------------------------------------------

    def act(self, action: Action) -> Dict[str, Any]:
        """
        执行动作（记录到日志，实际触发逻辑由外部处理）

        Args:
            action: 要执行的动作

        Returns:
            执行结果字典
        """
        # 1. 记录到日志
        self._log_action(action)

        # 2. 打印决策结果
        self._print_action(action)

        # 3. 返回执行摘要（实际触发由调用方处理）
        return {
            "status": "logged",
            "action": action.to_dict(),
            "log_file": str(self.activation_log_path)
        }

    def decide_and_act(self) -> Dict[str, Any]:
        """一次性完成决策和执行"""
        action = self.decide()
        return self.act(action)

    # ------------------------------------------------------------------------
    # 日志管理
    # ------------------------------------------------------------------------

    def _log_action(self, action: Action) -> None:
        """将动作写入 JSONL 日志"""
        with open(self.activation_log_path, "a", encoding="utf-8") as f:
            f.write(json.dumps(action.to_dict(), ensure_ascii=False) + "\n")

    def _print_action(self, action: Action) -> None:
        """打印动作信息"""
        priority_label = {0: "🔵", 1: "🟡", 2: "🔴"}[action.priority]
        type_label = {
            ActionType.TRIGGER_REFLEXION: "🔄 触发反思",
            ActionType.WRITE_MECHANISM_CARD: "📝 写机制卡片",
            ActionType.REWRITE_PROMPT: "✏️ 改写提示词",
            ActionType.PAUSE_AND_ALERT: "⏸️ 暂停报警",
            ActionType.OBSERVE: "👀 继续观察",
        }[action.action_type]

        print(f"\n{'='*60}")
        print(f"{priority_label} [自激活决策] {type_label}")
        print(f"   原因: {action.reason}")
        print(f"   时间: {action.timestamp}")
        if action.metrics_snapshot:
            ms = action.metrics_snapshot
            print(f"   快照: 通过率={_fmt(ms.get('pass_rate', 0))} "
                  f"| Φ={_fmt(ms.get('phi', 0))} "
                  f"| ΔG={_fmt(ms.get('delta_g', 0))}")
        print(f"{'='*60}\n")

    def get_recent_actions(self, n: int = 10) -> List[Action]:
        """读取最近的 n 条决策日志"""
        if not self.activation_log_path.exists():
            return []

        actions = []
        with open(self.activation_log_path, "r", encoding="utf-8") as f:
            lines = f.readlines()

        for line in lines[-n:]:
            try:
                data = json.loads(line.strip())
                actions.append(Action(
                    action_type=ActionType(data["action_type"]),
                    reason=data["reason"],
                    timestamp=data["timestamp"],
                    metrics_snapshot=data.get("metrics_snapshot", {}),
                    priority=data.get("priority", 0)
                ))
            except (json.JSONDecodeError, KeyError, ValueError):
                continue

        return actions

    # ------------------------------------------------------------------------
    # 工具方法
    # ------------------------------------------------------------------------

    def unclosed_failures_count(self) -> int:
        """返回未闭环失败数（兼容方法）"""
        return self.baseline.unclosed_failures if self.baseline else 0

    def summary(self) -> Dict[str, Any]:
        """返回当前状态摘要"""
        if self.baseline is None:
            self.refresh()

        b = self.baseline
        return {
            "baseline_path": str(self.baseline_path),
            "log_path": str(self.activation_log_path),
            "thresholds": {
                "pass_rate": self.pass_rate_threshold,
                "reflexion_success": self.reflexion_success_threshold,
                "unclosed_failure": self.unclosed_failure_threshold,
                "metacognition_decline": self.metacognition_decline_threshold,
            },
            "current_metrics": b.to_dict() if b else {},
            "recent_actions_count": len(self.get_recent_actions()),
        }


# ============================================================================
# 辅助函数
# ============================================================================

def load_baseline(baseline_path: str) -> BaselineMetrics:
    """便捷函数：加载基线数据"""
    activator = SelfActivator(baseline_path=baseline_path)
    return activator.load_baseline()


def _fmt(v: Any) -> str:
    """格式化数值显示"""
    if isinstance(v, float):
        return f"{v:.4f}"
    return str(v)


# ============================================================================
# 入口（直接运行）
# ============================================================================

if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print("用法: python self_activation.py <baseline_path> [failure_log_path]")
        sys.exit(1)

    baseline_path = sys.argv[1]
    failure_log_path = sys.argv[2] if len(sys.argv) > 2 else None

    activator = SelfActivator(baseline_path=baseline_path, failure_log_path=failure_log_path)

    try:
        activator.refresh()
        result = activator.decide_and_act()
        print(f"决策完成，已记录到: {result['log_file']}")
    except SelfActivationError as e:
        print(f"错误: {e}")
        sys.exit(1)
