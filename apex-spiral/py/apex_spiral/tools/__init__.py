"""APEX-spiral tools package.

工具自注册：任何在模块顶层调用 `registry.register(...)` 的工具
会被自动发现，无需在外部手动维护工具列表。

来源：hermes-agent `tools/registry.py`（2026-06-02 学习吸收）
"""
from .registry import ToolRegistry, Tool, get_registry, tool

__all__ = ["ToolRegistry", "Tool", "get_registry", "tool"]
