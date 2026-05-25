#!/usr/bin/env python3
"""
APEX Token Optimizer - 主流程集成
整合: 三层记忆 + trace_id + 错误模型 + 降级策略 + Rust API调用
"""
import sys
import time
import logging
from typing import Dict, Any, Optional

sys.path.insert(0, '/Users/lihongxin/.openclaw/workspace/skills/apex-skill-selector/scripts')
sys.path.insert(0, '/Users/lihongxin/.openclaw/workspace/skills/apex-token-optimizer')

from swr_memory import SWRMemoryManager
from emv_client import EMVClient, build_select_request
from token_trace import (
    new_trace_id, TokenOptimizerWithTrace, TokenOptimizerConfig,
    MockTokenAdapter, ErrorType, APEXError, CircuitBreaker
)

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class APEXWorkflow:
    """
    APEX 主工作流
    整合所有组件: 记忆 → trace → 优化 → Rust API → 响应
    """
    
    def __init__(self):
        # 三层记忆管理器
        self.memory = SWRMemoryManager()
        
        # Rust EMV API 客户端
        self.emv_client = EMVClient()
        
        # Token 优化器 (带 trace_id + 错误处理)
        token_adapter = MockTokenAdapter()
        self.token_optimizer = TokenOptimizerWithTrace(token_adapter)
        
        # 指标
        self.metrics = {
            "total_requests": 0,
            "success": 0,
            "fallback": 0,
            "error": 0
        }
        
        logger.info("[APEXWorkflow] 初始化完成")
    
    def execute(self, task: str, context: Dict[str, Any]) -> Dict[str, Any]:
        """
        执行主流程
        
        流程:
        1. 生成 trace_id
        2. 记忆查询
        3. Token 优化
        4. Rust API 选择
        5. 执行反馈
        """
        trace_id = new_trace_id()
        self.metrics["total_requests"] += 1
        
        logger.info(f"[{trace_id}] === 任务开始 ===")
        logger.info(f"[{trace_id}] 任务: {task}")
        
        result = {
            "trace_id": trace_id,
            "task": task,
            "success": False,
            "data": None,
            "error": None
        }
        
        try:
            # ===== 1. 记忆查询 =====
            logger.info(f"[{trace_id}] [Step 1/5] 查询记忆...")
            memory_result = self.memory.query(layers=["ring", "replay", "longterm"])
            logger.info(f"[{trace_id}] 记忆状态: ring={memory_result['ring']['stats']['len']}帧, "
                       f"replay={memory_result['replay']['stats']['len']}条")
            
            # ===== 2. Token 优化 =====
            logger.info(f"[{trace_id}] [Step 2/5] Token优化...")
            original_tokens = context.get("tokens", [1, 2, 3])
            coords = context.get("coords", [])
            
            optimized_tokens, opt_trace_id = self.token_optimizer.optimize(
                tokens=original_tokens,
                coords=coords,
                original_tokens=original_tokens
            )
            
            if opt_trace_id != trace_id:
                logger.warning(f"[{trace_id}] trace_id不匹配: {opt_trace_id}")
            
            logger.info(f"[{trace_id}] Token优化: {len(original_tokens)} → {len(optimized_tokens)}")
            
            # ===== 3. Rust API 选择 =====
            logger.info(f"[{trace_id}] [Step 3/5] Rust API选择...")
            
            ring_stats = memory_result['ring']['stats']
            gini = ring_stats.get("mean", 0.5) * 0.62
            
            api_request = build_select_request(
                request_id=trace_id,
                task_type="apex_workflow",
                goal=task,
                features=context.get("features", [0.5] * 9),
                memory_summary={
                    "recent_success_rate": ring_stats.get("mean", 0.5),
                    "recent_failure_rate": 1 - ring_stats.get("mean", 0.5)
                },
                candidates=context.get("candidates", [
                    {"skill_id": "gini-select", "prior": 0.4},
                    {"skill_id": "swrs-replay", "prior": 0.35},
                    {"skill_id": "apex-repair", "prior": 0.25}
                ]),
                gini=gini,
                delta_g=0.08,
                swr_trigger=True
            )
            
            api_response = self.emv_client.select_skill(api_request)
            selected_skill = api_response.get("selected_skill", "unknown")
            logger.info(f"[{trace_id}] 选中技能: {selected_skill}")
            
            # ===== 4. 添加记忆 =====
            logger.info(f"[{trace_id}] [Step 4/5] 记录记忆...")
            
            fitness = context.get("fitness", 0.8)
            add_result = self.memory.add(
                skill=selected_skill,
                fitness=fitness,
                data={"task": task, "trace_id": trace_id},
                trigger="task_end"
            )
            
            if add_result['ring_result']['archived']:
                logger.info(f"[{trace_id}] 记忆已固化")
            
            # ===== 5. 返回结果 =====
            logger.info(f"[{trace_id}] [Step 5/5] 返回结果...")
            
            result["success"] = True
            result["data"] = {
                "selected_skill": selected_skill,
                "tokens": {
                    "original": len(original_tokens),
                    "optimized": len(optimized_tokens)
                },
                "api_response": api_response,
                "memory_stats": self.memory.full_stats()
            }
            
            self.metrics["success"] += 1
            logger.info(f"[{trace_id}] === 任务成功 ===")
            
        except APEXError as e:
            logger.error(f"[{trace_id}] APEXError: {e.to_dict()}")
            result["error"] = e.to_dict()
            self.metrics["error"] += 1
            
        except Exception as e:
            logger.error(f"[{trace_id}] Unexpected error: {e}")
            result["error"] = {"type": "unexpected", "message": str(e)}
            self.metrics["error"] += 1
        
        # 打印指标
        if self.metrics["total_requests"] % 10 == 0:
            logger.info(f"[{trace_id}] 累计指标: {self.metrics}")
        
        return result
    
    def get_metrics(self) -> Dict[str, Any]:
        """获取指标"""
        opt_metrics = self.token_optimizer.get_metrics_summary()
        return {
            "workflow": self.metrics,
            "optimizer": opt_metrics,
            "circuit_state": self.token_optimizer.get_circuit_state()
        }

# ============ 测试 ============

if __name__ == "__main__":
    print("=" * 60)
    print("APEX Token Optimizer - 主流程集成测试")
    print("=" * 60)
    
    workflow = APEXWorkflow()
    
    # 测试用例
    test_cases = [
        {
            "name": "标准任务",
            "task": "分类任务",
            "context": {
                "tokens": list(range(100)),
                "coords": [[960, 540]],
                "features": [0.5, 0.7, 0.6, 0.8, 0.4, 0.9, 0.3, 0.75, 0.65],
                "fitness": 0.85
            }
        },
        {
            "name": "高Token任务",
            "task": "复杂推理",
            "context": {
                "tokens": list(range(500)),
                "coords": [[1920, 1080]],
                "features": [0.6, 0.8, 0.5, 0.9, 0.3, 0.7, 0.4, 0.85, 0.55],
                "fitness": 0.92
            }
        },
        {
            "name": "低置信度任务",
            "task": "快速分类",
            "context": {
                "tokens": [1, 2, 3],
                "coords": [],
                "features": [0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.4, 0.5],
                "fitness": 0.55
            }
        }
    ]
    
    success_count = 0
    error_count = 0
    
    for i, test in enumerate(test_cases, 1):
        print(f"\n[Test {i}] {test['name']}")
        print(f"  任务: {test['task']}")
        
        result = workflow.execute(test['task'], test['context'])
        
        if result['success']:
            print(f"  ✅ 成功")
            print(f"     选中技能: {result['data']['selected_skill']}")
            print(f"     Token: {result['data']['tokens']['original']} → {result['data']['tokens']['optimized']}")
            success_count += 1
        else:
            print(f"  ❌ 失败: {result['error']}")
            error_count += 1
    
    print("\n" + "=" * 60)
    print("测试结果汇总")
    print("=" * 60)
    print(f"  总测试: {len(test_cases)}")
    print(f"  成功: {success_count}")
    print(f"  失败: {error_count}")
    print(f"  成功率: {success_count/len(test_cases)*100:.1f}%")
    
    print("\n[全局指标]")
    print(f"  {workflow.get_metrics()}")
    
    print("\n" + "=" * 60)
    print("主流程集成测试完成")
    print("=" * 60)
