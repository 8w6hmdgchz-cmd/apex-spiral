"""
APEX Self-Activation Demo — 演示自激活驱动的完整决策流程

本演示：
1. 创建模拟的基线数据（代表不同系统状态）
2. 依次通过 5 个场景展示决策逻辑
3. 展示决策日志的写入和读取

场景覆盖：
- 场景1: 通过率 < 0.5 → 触发反思
- 场景2: 反思成功率 < 0.3 → 写机制卡片
- 场景3: 未闭环失败 > 5 → 改写提示词
- 场景4: 元认知分数下降 → 暂停报警
- 场景5: 一切正常 → 继续观察
"""

import json
import os
import tempfile
from pathlib import Path
from self_activation import SelfActivator, ActionType, BaselineMetrics


def create_demo_baseline(
    pass_rate: float = 0.6,
    reflexion_success_rate: float = 0.4,
    unclosed_failures: int = 3,
    metacognition_score: float = 0.65,
    metacognition_history: list = None,
    phi: float = 0.65,
    delta_g: float = 0.672,
    total_tasks: int = 10,
    passed_tasks: int = 6,
) -> dict:
    """创建模拟基线数据"""
    if metacognition_history is None:
        metacognition_history = [metacognition_score + 0.02]  # 上一次更高

    return {
        "pass_rate": pass_rate,
        "total_tasks": total_tasks,
        "passed_tasks": passed_tasks,
        "failed_tasks": total_tasks - passed_tasks,
        "reflexion_success_rate": reflexion_success_rate,
        "total_reflexions": 10,
        "successful_reflexions": int(10 * reflexion_success_rate),
        "unclosed_failures": unclosed_failures,
        "metacognition_score": metacognition_score,
        "metacognition_history": metacognition_history,
        "delta_g": delta_g,
        "phi": phi,
    }


def run_scenario(name: str, baseline_data: dict, activator: SelfActivator):
    """运行单个场景"""
    print(f"\n{'='*70}")
    print(f"📌 场景: {name}")
    print(f"{'='*70}")

    # 写入临时基线文件
    with open(activator.baseline_path, "w", encoding="utf-8") as f:
        json.dump(baseline_data, f, ensure_ascii=False, indent=2)

    # 重新加载并决策
    activator.refresh()
    action = activator.decide()

    # 执行并记录
    result = activator.act(action)
    return action


def main():
    print("""
╔══════════════════════════════════════════════════════════════════╗
║           APEX Self-Activation Demo — 自激活驱动演示             ║
║                                                                  ║
║  核心规则（非定时任务）：                                        ║
║  • 通过率 < 0.5      → trigger_reflexion（触发反思循环重试）    ║
║  • 反思成功率 < 0.3  → write_mechanism_card（写机制卡片）        ║
║  • 失败案例 > 5条    → rewrite_prompt（自动改提示词）           ║
║  • 元认知分数下降    → pause_and_alert（暂停并报警）            ║
║  • 其他              → observe（继续观察）                       ║
╚══════════════════════════════════════════════════════════════════╝
    """)

    # 使用临时目录存放演示数据
    demo_dir = Path(tempfile.mkdtemp(prefix="apex_self_activation_demo_"))
    baseline_path = demo_dir / "baseline_demo.json"
    failure_log_path = demo_dir / "failure_log.json"
    log_dir = demo_dir / "memory"

    print(f"📁 演示数据目录: {demo_dir}")

    # 初始化自激活引擎
    activator = SelfActivator(
        baseline_path=str(baseline_path),
        failure_log_path=str(failure_log_path),
        activation_log_dir=str(log_dir),
        pass_rate_threshold=0.5,
        reflexion_success_threshold=0.3,
        unclosed_failure_threshold=5,
        metacognition_decline_threshold=0.05,
    )

    # ====================================================================
    # 场景1: 通过率 < 0.5 → 触发反思
    # ====================================================================
    run_scenario(
        "通过率过低（应触发反思循环）",
        create_demo_baseline(
            pass_rate=0.3,          # < 0.5 阈值
            reflexion_success_rate=0.5,  # 正常
            unclosed_failures=2,    # 正常
            metacognition_score=0.65,
            metacognition_history=[0.67, 0.66, 0.65],  # 稳定
        ),
        activator
    )

    # ====================================================================
    # 场景2: 反思成功率 < 0.3 → 写机制卡片
    # ====================================================================
    run_scenario(
        "反思成功率过低（应写机制卡片）",
        create_demo_baseline(
            pass_rate=0.7,          # 通过率正常
            reflexion_success_rate=0.15,  # < 0.3 阈值
            unclosed_failures=2,
            metacognition_score=0.65,
            metacognition_history=[0.67, 0.66, 0.65],
        ),
        activator
    )

    # ====================================================================
    # 场景3: 未闭环失败 > 5 → 改写提示词
    # ====================================================================
    run_scenario(
        "未闭环失败过多（应改写提示词）",
        create_demo_baseline(
            pass_rate=0.6,
            reflexion_success_rate=0.5,
            unclosed_failures=8,    # > 5 阈值
            metacognition_score=0.65,
            metacognition_history=[0.67, 0.66, 0.65],
        ),
        activator
    )

    # ====================================================================
    # 场景4: 元认知分数下降 → 暂停报警（最高优先级）
    # ====================================================================
    run_scenario(
        "元认知分数下降（最高优先级 → 暂停报警）",
        create_demo_baseline(
            pass_rate=0.6,
            reflexion_success_rate=0.5,
            unclosed_failures=3,
            metacognition_score=0.58,  # 当前分数
            metacognition_history=[0.68, 0.67, 0.66, 0.65, 0.64, 0.63],  # 持续下降
        ),
        activator
    )

    # ====================================================================
    # 场景5: 一切正常 → 继续观察
    # ====================================================================
    run_scenario(
        "各项指标正常（应继续观察）",
        create_demo_baseline(
            pass_rate=0.85,         # 远超 0.5
            reflexion_success_rate=0.7,  # 远超 0.3
            unclosed_failures=1,    # 远低于 5
            metacognition_score=0.72,  # 上升中
            metacognition_history=[0.65, 0.67, 0.70, 0.72],
        ),
        activator
    )

    # ====================================================================
    # 展示决策日志
    # ====================================================================
    print(f"\n{'='*70}")
    print(f"📜 决策日志文件: {activator.activation_log_path}")
    print(f"{'='*70}")

    recent = activator.get_recent_actions(n=10)
    print(f"\n共记录 {len(recent)} 条决策:\n")

    for i, action in enumerate(recent, 1):
        type_emoji = {
            ActionType.TRIGGER_REFLEXION: "🔄",
            ActionType.WRITE_MECHANISM_CARD: "📝",
            ActionType.REWRITE_PROMPT: "✏️",
            ActionType.PAUSE_AND_ALERT: "⏸️",
            ActionType.OBSERVE: "👀",
        }[action.action_type]

        print(f"  {i}. {type_emoji} [{action.action_type.value}]")
        print(f"     原因: {action.reason}")
        print(f"     时间: {action.timestamp}")
        print()

    # ====================================================================
    # 展示状态摘要
    # ====================================================================
    print(f"\n{'='*70}")
    print(f"📊 SelfActivator 状态摘要")
    print(f"{'='*70}")
    summary = activator.summary()
    print(f"\n基线路径: {summary['baseline_path']}")
    print(f"日志路径: {summary['log_path']}")
    print(f"决策阈值:")
    for k, v in summary['thresholds'].items():
        print(f"  • {k}: {v}")
    print(f"\n最近决策数: {summary['recent_actions_count']}")

    # ====================================================================
    # 清理提示
    # ====================================================================
    print(f"""
╔══════════════════════════════════════════════════════════════════╗
║  ✅ 演示完成！                                                   ║
║                                                                  ║
║  决策日志位置:                                                   ║
║  {activator.activation_log_path}                                  ║
║                                                                  ║
║  关键特性:                                                       ║
║  • 纯状态驱动，不依赖定时任务                                    ║
║  • 每个动作都记录到 JSONL 日志                                  ║
║  • 与反思循环、评测运行器完全解耦                               ║
║  • 决策阈值可配置                                                ║
╚══════════════════════════════════════════════════════════════════╝
    """)

    return activator


if __name__ == "__main__":
    activator = main()
