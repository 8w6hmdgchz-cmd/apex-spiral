#!/usr/bin/env python3
"""
APEX Token Optimizer - Trace & Error Management
trace_id 全链路透传 + 错误模型 + 降级策略
"""
import time
import uuid
import logging
from enum import Enum
from dataclasses import dataclass, field
from typing import Optional, Callable, Any
from collections import defaultdict

logger = logging.getLogger(__name__)

# ============ trace_id 生成 ============

def new_trace_id() -> str:
    """生成唯一trace_id"""
    return f"apex-{int(time.time()*1000)}-{uuid.uuid4().hex[:8]}"

# ============ 错误分类 ============

class ErrorType(Enum):
    """错误分类：可重试/不可重试"""
    RETRYABLE = "retryable"      # 可重试：网络抖动、超时
    NON_RETRYABLE = "non_retryable"  # 不可重试：参数错误、权限问题
    FATAL = "fatal"              # 致命：Rust panic、内存溢出

class ErrorCode(Enum):
    # 可重试 (4xx/5xx)
    TIMEOUT = (4001, "timeout", ErrorType.RETRYABLE)
    NETWORK_ERROR = (4002, "network_error", ErrorType.RETRYABLE)
    SERVICE_UNAVAILABLE = (4003, "service_unavailable", ErrorType.RETRYABLE)
    RATE_LIMITED = (4004, "rate_limited", ErrorType.RETRYABLE)
    
    # 不可重试 (4xx)
    INVALID_PARAMS = (4101, "invalid_params", ErrorType.NON_RETRYABLE)
    INVALID_TOKEN = (4102, "invalid_token", ErrorType.NON_RETRYABLE)
    CONTRACT_MISMATCH = (4103, "contract_mismatch", ErrorType.NON_RETRYABLE)
    
    # 致命 (5xx)
    RUST_PANIC = (5001, "rust_panic", ErrorType.FATAL)
    OUT_OF_MEMORY = (5002, "out_of_memory", ErrorType.FATAL)
    INTERNAL_ERROR = (5003, "internal_error", ErrorType.FATAL)

@dataclass
class APEXError(Exception):
    code: int
    message: str
    error_type: ErrorType
    trace_id: str = ""
    retriable: bool = False
    
    def __post_init__(self):
        self.retriable = self.error_type == ErrorType.RETRYABLE
    
    def to_dict(self):
        return {
            "code": self.code,
            "message": self.message,
            "type": self.error_type.value,
            "trace_id": self.trace_id,
            "retriable": self.retriable
        }

# ============ Circuit Breaker (熔断器) ============

class CircuitState(Enum):
    CLOSED = "closed"      # 正常
    OPEN = "open"          # 熔断
    HALF_OPEN = "half_open"  # 半开

@dataclass
class CircuitBreakerConfig:
    failure_threshold: int = 5      # 连续失败次数阈值
    recovery_timeout: float = 30.0  # 恢复超时(秒)
    half_open_max_calls: int = 3    # 半开状态最大尝试次数

class CircuitBreaker:
    def __init__(self, name: str, cfg: CircuitBreakerConfig = None):
        self.name = name
        self.cfg = cfg or CircuitBreakerConfig()
        self.state = CircuitState.CLOSED
        self.failure_count = 0
        self.last_failure_time = 0.0
        self.half_open_calls = 0
    
    def record_success(self):
        self.failure_count = 0
        self.state = CircuitState.CLOSED
        self.half_open_calls = 0
    
    def record_failure(self):
        self.failure_count += 1
        self.last_failure_time = time.time()
        
        if self.failure_count >= self.cfg.failure_threshold:
            self.state = CircuitState.OPEN
            logger.warning(f"Circuit [{self.name}] OPEN after {self.failure_count} failures")
    
    def can_execute(self) -> bool:
        if self.state == CircuitState.CLOSED:
            return True
        
        if self.state == CircuitState.OPEN:
            # 检查恢复超时
            if time.time() - self.last_failure_time > self.cfg.recovery_timeout:
                self.state = CircuitState.HALF_OPEN
                self.half_open_calls = 0
                logger.info(f"Circuit [{self.name}] HALF_OPEN")
                return True
            return False
        
        if self.state == CircuitState.HALF_OPEN:
            return self.half_open_calls < self.cfg.half_open_max_calls
        
        return False
    
    def record_half_open_call(self):
        self.half_open_calls += 1
        if self.half_open_calls >= self.cfg.half_open_max_calls:
            self.state = CircuitState.OPEN

# ============ 指标收集器 ============

@dataclass
class MetricsCollector:
    trace_id: str
    start_time: float = field(default_factory=time.time)
    end_time: float = 0.0
    success: bool = False
    fallback: bool = False
    error_code: int = 0
    
    def record_success(self):
        self.success = True
        self.end_time = time.time()
    
    def record_failure(self, code: int):
        self.success = False
        self.error_code = code
        self.end_time = time.time()
    
    def record_fallback(self):
        self.fallback = True
        self.end_time = time.time()
    
    @property
    def latency_ms(self) -> float:
        if self.end_time == 0.0:
            return (time.time() - self.start_time) * 1000
        return (self.end_time - self.start_time) * 1000
    
    def to_dict(self):
        return {
            "trace_id": self.trace_id,
            "latency_ms": round(self.latency_ms, 2),
            "success": self.success,
            "fallback": self.fallback,
            "error_code": self.error_code
        }

class MetricsStore:
    """全局指标存储 (简化版)"""
    def __init__(self, max_size: int = 10000):
        self.max_size = max_size
        self.metrics = []
        self.counters = defaultdict(int)
    
    def add(self, m: MetricsCollector):
        self.metrics.append(m.to_dict())
        if len(self.metrics) > self.max_size:
            self.metrics.pop(0)
        
        if m.success:
            self.counters["success"] += 1
        elif m.fallback:
            self.counters["fallback"] += 1
        else:
            self.counters["failure"] += 1
    
    def summary(self) -> dict:
        total = sum(self.counters.values())
        if total == 0:
            return {"total": 0}
        
        return {
            "total": total,
            "success_rate": round(self.counters["success"] / total, 4),
            "fallback_rate": round(self.counters["fallback"] / total, 4),
            "failure_rate": round(self.counters["failure"] / total, 4),
            "recent_5": self.metrics[-5:]
        }

# 全局指标存储
_global_metrics = MetricsStore()

# ============ 降级执行器 ============

class FallbackExecutor:
    """降级策略执行器"""
    
    def __init__(self, bypass_func: Callable[[], Any]):
        self.bypass_func = bypass_func
    
    def execute(self, trace_id: str, metrics: MetricsCollector) -> Any:
        """执行降级回调"""
        logger.info(f"[{trace_id}] Executing fallback (bypass)")
        try:
            result = self.bypass_func()
            metrics.record_fallback()
            _global_metrics.add(metrics)
            return result
        except Exception as e:
            logger.error(f"[{trace_id}] Fallback failed: {e}")
            metrics.record_failure(0)
            _global_metrics.add(metrics)
            raise

# ============ 核心类 ============

@dataclass
class TokenOptimizerConfig:
    """Token优化器配置"""
    enabled: bool = True
    timeout_ms: float = 30.0
    retry_max: int = 2
    retry_delay_ms: float = 100.0
    enable_fallback: bool = True  # 失败时是否回退到原始token
    enable_circuit_breaker: bool = True
    trace_enabled: bool = True

class TokenOptimizerWithTrace:
    """
    带 trace_id + 错误处理 + 降级策略的 Token 优化器
    """
    
    def __init__(self, adapter, cfg: TokenOptimizerConfig = None):
        self.adapter = adapter
        self.cfg = cfg or TokenOptimizerConfig()
        self.circuit_breaker = CircuitBreaker(
            "token_optimizer",
            CircuitBreakerConfig(failure_threshold=5, recovery_timeout=30.0)
        )
        self.fallback_executor = FallbackExecutor(bypass_func=self._bypass_default)
    
    def _bypass_default(self):
        """默认降级：返回原始token"""
        return self.adapter.last_original_tokens
    
    def _build_error(self, code: int, message: str, trace_id: str) -> APEXError:
        """构建APEXError"""
        for ec in ErrorCode:
            if ec.value[0] == code:
                return APEXError(
                    code=code,
                    message=message,
                    error_type=ec.value[2],
                    trace_id=trace_id
                )
        return APEXError(
            code=code,
            message=message,
            error_type=ErrorType.INTERNAL_ERROR,
            trace_id=trace_id
        )
    
    def optimize(self, tokens: list, coords: list = None, 
                 original_tokens: list = None) -> tuple[list, str]:
        """
        优化token，带完整错误处理和降级
        
        Returns:
            (optimized_tokens, trace_id)
        """
        trace_id = new_trace_id() if self.cfg.trace_enabled else "no-trace"
        metrics = MetricsCollector(trace_id=trace_id)
        
        # 检查熔断器
        if self.cfg.enable_circuit_breaker and not self.circuit_breaker.can_execute():
            logger.warning(f"[{trace_id}] Circuit open, using fallback")
            result = self.fallback_executor.execute(trace_id, metrics)
            return result, trace_id
        
        # 记录原始token (用于降级)
        self.adapter.last_original_tokens = original_tokens or tokens
        
        # 执行优化
        for attempt in range(self.cfg.retry_max + 1):
            try:
                result = self.adapter.optimize(
                    trace_id=trace_id,
                    tokens=tokens,
                    coords=coords,
                    timeout_ms=self.cfg.timeout_ms
                )
                
                self.circuit_breaker.record_success()
                metrics.record_success()
                _global_metrics.add(metrics)
                
                return result, trace_id
                
            except APEXError as e:
                e.trace_id = trace_id
                
                if not e.retriable or attempt >= self.cfg.retry_max:
                    # 不可重试错误或已达最大重试次数
                    logger.error(f"[{trace_id}] Non-retryable error: {e.to_dict()}")
                    
                    if self.cfg.enable_fallback:
                        fallback_result = self.fallback_executor.execute(trace_id, metrics)
                        return fallback_result, trace_id
                    
                    metrics.record_failure(e.code)
                    _global_metrics.add(metrics)
                    raise
                
                # 可重试，延迟后重试
                logger.warning(f"[{trace_id}] Retryable error, attempt {attempt+1}/{self.cfg.retry_max}")
                time.sleep(self.cfg.retry_delay_ms / 1000.0)
                
            except Exception as e:
                logger.error(f"[{trace_id}] Unexpected error: {e}")
                
                if self.cfg.enable_fallback:
                    fallback_result = self.fallback_executor.execute(trace_id, metrics)
                    return fallback_result, trace_id
                
                metrics.record_failure(0)
                _global_metrics.add(metrics)
                raise
        
        return tokens, trace_id
    
    def get_metrics_summary(self) -> dict:
        """获取指标摘要"""
        return _global_metrics.summary()
    
    def get_circuit_state(self) -> str:
        """获取熔断状态"""
        return self.circuit_breaker.state.value

# ============ Mock Adapter (测试用) ============

class MockTokenAdapter:
    def __init__(self, fail_mode: bool = False):
        self.last_original_tokens = []
        self.fail_mode = fail_mode
    
    def optimize(self, trace_id: str, tokens: list, coords: list = None, timeout_ms: float = 30.0) -> list:
        """模拟优化：失败模式抛出异常，正常模式返回截断"""
        if self.fail_mode:
            raise APEXError(
                code=5003,
                message="internal_error",
                error_type=ErrorType.FATAL,
                trace_id=trace_id
            )
        if len(tokens) > 100:
            return tokens[:100] + [999]  # 添加一个标记token
        return tokens + [888]

# ============ 测试 ============

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    
    print("=== Token Optimizer Trace & Error Test ===\n")
    
    # 1. trace_id 生成测试
    print("[1] trace_id 生成测试")
    t1 = new_trace_id()
    t2 = new_trace_id()
    print(f"  trace_id_1: {t1}")
    print(f"  trace_id_2: {t2}")
    print(f"  唯一性: {'✅' if t1 != t2 else '❌'}")
    
    # 2. 错误分类测试
    print("\n[2] 错误分类测试")
    err = APEXError(
        code=4001,
        message="timeout",
        error_type=ErrorType.RETRYABLE,
        trace_id="test-001"
    )
    print(f"  错误: {err.to_dict()}")
    print(f"  可重试: {'✅' if err.retriable else '❌'}")
    
    # 3. 熔断器测试
    print("\n[3] 熔断器测试")
    cb = CircuitBreaker("test", CircuitBreakerConfig(failure_threshold=3, recovery_timeout=5.0))
    print(f"  初始状态: {cb.state.value}")
    
    # 模拟3次失败
    for i in range(3):
        cb.record_failure()
    print(f"  3次失败后: {cb.state.value} (预期: open)")
    
    # 4. 降级策略测试 (失败模式)
    print("\n[4] 降级策略测试 (Fatal错误→降级)")
    adapter = MockTokenAdapter(fail_mode=True)
    optimizer = TokenOptimizerWithTrace(adapter)
    original = list(range(200))
    
    result, trace_id = optimizer.optimize(tokens=[1,2,3], original_tokens=original)
    print(f"  trace_id: {trace_id}")
    print(f"  原始长度: {len(original)}, 降级后: {len(result)}")
    print(f"  使用降级: {'✅' if result == original else '❌'}")
    
    # 5. 正常优化测试
    print("\n[5] 正常优化测试")
    adapter2 = MockTokenAdapter(fail_mode=False)
    optimizer2 = TokenOptimizerWithTrace(adapter2)
    
    result2, trace_id2 = optimizer2.optimize(tokens=list(range(50)), original_tokens=list(range(50)))
    print(f"  trace_id: {trace_id2}")
    print(f"  输入: 50 tokens, 输出: {len(result2)} tokens")
    print(f"  正常优化: {'✅' if result2 != list(range(50)) else '❌'}")
    
    # 6. 指标摘要
    print("\n[6] 指标摘要")
    summary = optimizer.get_metrics_summary()
    print(f"  {summary}")
    
    print("\n=== 测试完成 ===")
