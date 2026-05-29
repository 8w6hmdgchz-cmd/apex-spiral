"""
APEX Observation Module - 主动感知模块
实现不依赖用户消息的主动环境感知
"""

import time
from typing import Callable, List, Optional, Dict, Any
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum


class ObservationType(Enum):
    """观察类型"""
    TIME = "time"                    # 时间感知
    PENDING = "pending"              # 待处理任务
    ANOMALY = "anomaly"              # 异常检测
    PATTERN = "pattern"              # 模式识别
    REMINDER = "reminder"            # 提醒
    CONTEXT = "context"              # 上下文变化


@dataclass
class Observation:
    """观察记录"""
    type: ObservationType
    content: str
    timestamp: str
    priority: float = 0.5  # 0-1, 优先级
    metadata: Dict[str, Any] = field(default_factory=dict)
    acknowledged: bool = False


@dataclass
class ObservationConfig:
    """Observation 配置"""
    check_interval_seconds: int = 300  # 5分钟检查一次
    max_pending_age_hours: float = 24.0  # 超过24小时算过期
    anomaly_threshold: float = 0.7  # 异常阈值


class ApexObservation:
    """
    APEX 主动感知模块
    
    核心能力：
    1. 主动检查环境状态
    2. 识别待处理任务
    3. 检测异常
    4. 模式识别
    """
    
    def __init__(
        self,
        llm_func: Optional[Callable[[str], str]] = None,
        memory_stream=None,  # ApexMemoryStream
        config: Optional[ObservationConfig] = None
    ):
        """
        Args:
            llm_func: LLM 函数，用于复杂分析
            memory_stream: 记忆流引用
            config: 配置
        """
        self.llm = llm_func
        self.memory_stream = memory_stream
        self.config = config or ObservationConfig()
        
        self.observations: List[Observation] = []
        self.last_check: Optional[datetime] = None
        
        # 检查函数列表
        self.check_functions: List[Callable[[], List[Observation]]] = []
        
        # 注册默认检查
        self._register_default_checks()
    
    def _register_default_checks(self) -> None:
        """注册默认检查函数"""
        # 时间感知
        self.register_check(self._check_time)
    
    def register_check(
        self,
        check_func: Callable[[], List[Observation]],
        name: str = ""
    ) -> None:
        """注册检查函数"""
        self.check_functions.append(check_func)
    
    def observe(self) -> List[Observation]:
        """
        执行主动观察
        
        Returns:
            新观察列表
        """
        new_observations = []
        
        for check_func in self.check_functions:
            try:
                obs = check_func()
                new_observations.extend(obs)
            except Exception as e:
                print(f"观察检查失败: {e}")
        
        # 存储观察
        for obs in new_observations:
            self.observations.append(obs)
            
            # 如果有记忆流，同步存储
            if self.memory_stream:
                from .memory_stream import MemoryType
                self.memory_stream.add(
                    content=f"[{obs.type.value}] {obs.content}",
                    memory_type=MemoryType.OBSERVATION,
                    importance=obs.priority,
                    metadata={"observation_type": obs.type.value}
                )
        
        self.last_check = datetime.now()
        
        # 清理旧观察
        self._prune_old()
        
        return new_observations
    
    def _check_time(self) -> List[Observation]:
        """检查时间"""
        now = datetime.now()
        observations = []
        
        # 早安检查 (8AM - 10AM)
        if 8 <= now.hour <= 10:
            observations.append(Observation(
                type=ObservationType.CONTEXT,
                content=f"早上好！当前时间 {now.strftime('%H:%M')}",
                timestamp=now.isoformat(),
                priority=0.3
            ))
        
        # 晚安检查 (22PM - 23PM)
        if 22 <= now.hour <= 23:
            observations.append(Observation(
                type=ObservationType.CONTEXT,
                content=f"晚上了！当前时间 {now.strftime('%H:%M')}",
                timestamp=now.isoformat(),
                priority=0.3
            ))
        
        # 整点检查
        if now.minute == 0:
            observations.append(Observation(
                type=ObservationType.TIME,
                content=f"整点时间: {now.strftime('%H:00')}",
                timestamp=now.isoformat(),
                priority=0.2
            ))
        
        return observations
    
    def check_pending_tasks(
        self,
        get_pending_func: Callable[[], List[Dict[str, Any]]]
    ) -> List[Observation]:
        """
        检查待处理任务
        
        Args:
            get_pending_func: 获取待处理任务的函数
        """
        observations = []
        
        try:
            pending = get_pending_func()
            
            for task in pending:
                age_hours = (datetime.now() - datetime.fromisoformat(
                    task.get("created_at", datetime.now().isoformat())
                )).total_seconds() / 3600
                
                priority = 0.5
                if age_hours > self.config.max_pending_age_hours:
                    priority = 0.9  # 高优先级
                    content = f"⏰ 超时任务: {task.get('name', '未命名')} (超过{age_hours:.1f}小时)"
                else:
                    content = f"待处理: {task.get('name', '未命名')}"
                
                observations.append(Observation(
                    type=ObservationType.PENDING,
                    content=content,
                    timestamp=datetime.now().isoformat(),
                    priority=priority,
                    metadata={"task": task}
                ))
        except Exception as e:
            observations.append(Observation(
                type=ObservationType.ANOMALY,
                content=f"检查待处理任务失败: {e}",
                timestamp=datetime.now().isoformat(),
                priority=0.8
            ))
        
        return observations
    
    def detect_anomaly(
        self,
        check_func: Callable[[], Dict[str, Any]]
    ) -> Optional[Observation]:
        """
        检测异常
        
        Args:
            check_func: 返回 {"status": "ok"/"warning"/"error", "message": ""}
        """
        try:
            result = check_func()
            status = result.get("status", "ok")
            
            if status in ("warning", "error"):
                message = result.get("message", "未知问题")
                
                priority = 0.9 if status == "error" else 0.6
                
                return Observation(
                    type=ObservationType.ANOMALY,
                    content=f"⚠️ 异常: {message}" if status == "warning" else f"🚨 错误: {message}",
                    timestamp=datetime.now().isoformat(),
                    priority=priority,
                    metadata={"status": status}
                )
        except Exception as e:
            return Observation(
                type=ObservationType.ANOMALY,
                content=f"异常检测失败: {e}",
                timestamp=datetime.now().isoformat(),
                priority=0.7
            )
        
        return None
    
    def recognize_pattern(
        self,
        history: List[Dict[str, Any]],
        pattern_type: str = "default"
    ) -> Optional[Observation]:
        """
        识别模式
        
        Args:
            history: 历史记录
            pattern_type: 模式类型
        """
        if not self.llm or not history:
            return None
        
        prompt = f"""分析以下历史记录，识别模式：

{history}

识别：
1. 重复出现的模式
2. 周期性规律
3. 异常趋势

如果发现重要模式，返回模式描述。
如果无明显模式，返回 "无模式"。"""
        
        try:
            result = self.llm(prompt)
            
            if "无模式" not in result:
                return Observation(
                    type=ObservationType.PATTERN,
                    content=f"🔄 识别到模式: {result}",
                    timestamp=datetime.now().isoformat(),
                    priority=0.6,
                    metadata={"pattern_type": pattern_type}
                )
        except Exception as e:
            print(f"模式识别失败: {e}")
        
        return None
    
    def get_unacknowledged(self, min_priority: float = 0.5) -> List[Observation]:
        """获取未确认的观察"""
        return [
            o for o in self.observations
            if not o.acknowledged and o.priority >= min_priority
        ]
    
    def acknowledge(self, observation: Observation) -> None:
        """确认观察"""
        observation.acknowledged = True
    
    def _prune_old(self, max_age_hours: float = 24.0) -> None:
        """删除旧观察"""
        cutoff = datetime.now().timestamp() - max_age_hours * 3600
        
        self.observations = [
            o for o in self.observations
            if datetime.fromisoformat(o.timestamp).timestamp() > cutoff
            or o.priority > 0.8  # 高优先级保留
        ]
    
    def summary(self) -> dict:
        """返回观察摘要"""
        unacknowledged = self.get_unacknowledged()
        
        return {
            "total_observations": len(self.observations),
            "unacknowledged": len(unacknowledged),
            "last_check": self.last_check.isoformat() if self.last_check else None,
            "check_functions": len(self.check_functions),
            "pending_alerts": [
                {"type": o.type.value, "content": o.content[:50], "priority": o.priority}
                for o in unacknowledged[:5]
            ]
        }
