"""
APEX V10.1 Σ_memory Python bridge.

This module mirrors the Rust `MemoryEntry`, `MemoryType`, `SuperMemoryParams`,
and `add_memory_entry`/`calculate_sigma_memory` behavior from `apex_v10.rs` so
Python execution loops can continuously feed the Σ_memory pool without changing
Rust test code or requiring a native extension.
"""

from __future__ import annotations

import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, Iterable, List, Optional


class MemoryType(Enum):
    """Rust-compatible V10.1 memory types."""

    SEMANTIC = "Semantic"
    EPISODIC = "Episodic"
    PROCEDURAL = "Procedural"
    WORKING = "Working"


@dataclass
class MemoryEntry:
    """Python representation of Rust `MemoryEntry`."""

    id: str
    content: str
    embedding: List[float] = field(default_factory=list)
    timestamp: int = field(default_factory=lambda: int(time.time()))
    importance: float = 0.5
    memory_type: MemoryType = MemoryType.EPISODIC
    access_count: int = 0

    def __post_init__(self) -> None:
        self.importance = _clamp(self.importance, 0.0, 1.0)
        if isinstance(self.memory_type, str):
            self.memory_type = _coerce_memory_type(self.memory_type)

    def to_rust_dict(self) -> Dict[str, Any]:
        """Return a Rust-FFI-friendly payload if a native binding is added later."""
        return {
            "id": self.id,
            "content": self.content,
            "embedding": list(self.embedding),
            "timestamp": int(self.timestamp),
            "importance": float(self.importance),
            "memory_type": self.memory_type.value,
            "access_count": int(self.access_count),
        }


@dataclass
class SuperMemoryParams:
    """Python mirror of Rust `SuperMemoryParams`."""

    learn_rate: float = 0.7
    decay_factor: float = 0.95
    max_entries: int = 10000
    retention_threshold: float = 0.6
    memory_entries: List[MemoryEntry] = field(default_factory=list)


def calculate_sigma_memory(params: SuperMemoryParams) -> float:
    """Σ_memory = Learn × Search × MultiModal × Profile, matching Rust logic."""
    learn = _clamp(params.learn_rate, 0.0, 1.0)
    search = max(params.retention_threshold * params.learn_rate, 0.0) ** 0.5
    type_diversity = calculate_type_diversity(params.memory_entries)
    multimodal = max(learn * search * type_diversity, 0.1)
    profile = max(0.1, multimodal)
    decay = _clamp(params.decay_factor, 0.0, 1.0)
    return learn * search * multimodal * profile * decay


def calculate_type_diversity(entries: Iterable[MemoryEntry]) -> float:
    """Compute normalized entropy across Rust-compatible memory types."""
    entries = list(entries)
    if not entries:
        return 0.5

    total = float(len(entries))
    diversity = 0.0
    for memory_type in MemoryType:
        count = sum(1 for entry in entries if entry.memory_type == memory_type)
        if count:
            p = count / total
            diversity += -p * _log2(p)

    return _clamp(diversity / 2.0, 0.1, 1.0)


def add_memory_entry(params: SuperMemoryParams, entry: MemoryEntry) -> None:
    """Add a memory entry using the same retention behavior as Rust."""
    if params.max_entries <= 0:
        return

    if len(params.memory_entries) >= params.max_entries:
        _remove_low_importance_entries(params)

    params.memory_entries.append(entry)


def access_memory(params: SuperMemoryParams, entry_id: str) -> None:
    """Update memory access count and importance, matching Rust behavior."""
    for entry in params.memory_entries:
        if entry.id == entry_id:
            entry.access_count += 1
            entry.importance = min(entry.importance + 0.01, 1.0)
            break


def search_memory(params: SuperMemoryParams, query: str) -> List[MemoryEntry]:
    """Simple content search compatible with Rust `search_memory`."""
    needle = query.lower()
    return [entry for entry in params.memory_entries if needle in entry.content.lower()]


class ApexMemoryBridge:
    """
    Bridge that lets Python runtime code feed the V10.1 Σ_memory pool.

    The bridge intentionally degrades safely: failures are captured in
    `last_error` and never interrupt task execution.
    """

    def __init__(self, params: Optional[SuperMemoryParams] = None, enabled: bool = True):
        self.params = params or SuperMemoryParams()
        self.enabled = enabled
        self.last_error: Optional[str] = None

    def add_entry(self, entry: MemoryEntry) -> Optional[str]:
        """Add a prebuilt MemoryEntry. Returns entry id or None on failure."""
        if not self.enabled:
            return None
        try:
            add_memory_entry(self.params, entry)
            self.last_error = None
            return entry.id
        except Exception as exc:  # defensive: bridge must not break main loop
            self.last_error = f"add_entry failed: {exc}"
            return None

    def add_from_interaction(
        self,
        task: str,
        result: Any,
        *,
        importance: float = 0.7,
        memory_type: Optional[MemoryType] = None,
        embedding: Optional[List[float]] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> Optional[str]:
        """Create and add a memory entry after an important interaction."""
        metadata = metadata or {}
        content = f"任务: {task}\n结果: {result}"
        if metadata:
            content += f"\n元数据: {metadata}"

        entry = MemoryEntry(
            id=f"sigma_mem_{time.time_ns()}",
            content=content,
            embedding=embedding or [],
            timestamp=int(time.time()),
            importance=importance,
            memory_type=memory_type or self._next_balanced_type(),
            access_count=0,
        )
        return self.add_entry(entry)

    def add_interaction_bundle(
        self,
        task: str,
        result: Any,
        *,
        importance: float = 0.7,
        embedding: Optional[List[float]] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> List[str]:
        """
        Add a balanced set of entries for a completed task.

        Rust Σ_memory rewards type diversity. A task completion carries all four
        V10.1 memory facets: Working(task context), Procedural(result path),
        Episodic(interaction event), and Semantic(reusable lesson). Storing the
        facets separately prevents production traffic from collapsing into a
        single memory type and recreating the original Σ_memory bottleneck.
        """
        if not self.enabled:
            return []

        metadata = metadata or {}
        base_id = f"sigma_mem_{time.time_ns()}"
        timestamp = int(time.time())
        facets = [
            (MemoryType.WORKING, f"任务上下文: {task}", importance * 0.90),
            (MemoryType.PROCEDURAL, f"执行结果: {result}", importance),
            (MemoryType.EPISODIC, f"交互事件: 任务[{task}]完成，结果[{result}]", importance * 0.95),
            (MemoryType.SEMANTIC, f"可复用经验: 完成任务[{task}]得到结果[{result}]", importance * 0.85),
        ]

        added: List[str] = []
        for index, (memory_type, content, facet_importance) in enumerate(facets):
            if metadata:
                content += f"\n元数据: {metadata}"
            entry = MemoryEntry(
                id=f"{base_id}_{index}",
                content=content,
                embedding=embedding or [],
                timestamp=timestamp,
                importance=facet_importance,
                memory_type=memory_type,
                access_count=0,
            )
            entry_id = self.add_entry(entry)
            if entry_id:
                added.append(entry_id)
        return added

    def _next_balanced_type(self) -> MemoryType:
        counts = {
            memory_type: sum(1 for entry in self.params.memory_entries if entry.memory_type == memory_type)
            for memory_type in MemoryType
        }
        return min(counts, key=counts.get)

    def add_from_memory_stream(self, memory: Any) -> Optional[str]:
        """Convert an ApexMemoryStream Memory object into a V10.1 MemoryEntry."""
        try:
            stream_type = getattr(getattr(memory, "type", None), "value", str(getattr(memory, "type", "")))
            entry = MemoryEntry(
                id=getattr(memory, "id", f"sigma_mem_{time.time_ns()}"),
                content=getattr(memory, "content", ""),
                embedding=getattr(memory, "embedding", None) or [],
                timestamp=_timestamp_to_int(getattr(memory, "timestamp", None)),
                importance=getattr(memory, "importance", 0.5),
                memory_type=_map_stream_type(stream_type),
                access_count=0,
            )
            return self.add_entry(entry)
        except Exception as exc:
            self.last_error = f"add_from_memory_stream failed: {exc}"
            return None

    def sigma_memory(self) -> float:
        """Return current Σ_memory; errors degrade to 0.0 and are recorded."""
        try:
            return calculate_sigma_memory(self.params)
        except Exception as exc:
            self.last_error = f"sigma_memory failed: {exc}"
            return 0.0

    def summary(self) -> Dict[str, Any]:
        """Return bridge diagnostics for agent summary/status."""
        return {
            "enabled": self.enabled,
            "sigma_memory": self.sigma_memory(),
            "memory_entries": len(self.params.memory_entries),
            "type_diversity": calculate_type_diversity(self.params.memory_entries),
            "last_error": self.last_error,
        }


def _remove_low_importance_entries(params: SuperMemoryParams) -> None:
    keep_count = int(params.max_entries * 0.8)
    params.memory_entries.sort(key=lambda entry: entry.importance, reverse=True)
    params.memory_entries = params.memory_entries[:keep_count]


def _map_stream_type(stream_type: str) -> MemoryType:
    mapping = {
        "observation": MemoryType.EPISODIC,
        "reflection": MemoryType.SEMANTIC,
        "plan": MemoryType.WORKING,
        "execution": MemoryType.PROCEDURAL,
        "insight": MemoryType.SEMANTIC,
    }
    return mapping.get(str(stream_type).lower(), MemoryType.EPISODIC)


def _coerce_memory_type(value: str) -> MemoryType:
    normalized = value.lower()
    for memory_type in MemoryType:
        if normalized in {memory_type.name.lower(), memory_type.value.lower()}:
            return memory_type
    return MemoryType.EPISODIC


def _timestamp_to_int(value: Any) -> int:
    if value is None:
        return int(time.time())
    if isinstance(value, (int, float)):
        return int(value)
    try:
        from datetime import datetime

        return int(datetime.fromisoformat(str(value)).timestamp())
    except Exception:
        return int(time.time())


def _clamp(value: float, lower: float, upper: float) -> float:
    return max(lower, min(float(value), upper))


def _log2(value: float) -> float:
    import math

    return math.log2(value)
