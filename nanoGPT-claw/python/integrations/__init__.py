# Integrations Module
"""
NanoGPT-Claw Integrations
==========================
真实集成模块
"""
from .github_integration import GitHubIntegration
from .feishu_integration import FeishuIntegration
from .auto_research import AutoResearchIntegration
from .openhands import OpenHandsIntegration
from .superpowers import SuperPowersEngine

__all__ = [
    "GitHubIntegration",
    "FeishuIntegration",
    "AutoResearchIntegration",
    "OpenHandsIntegration",
    "SuperPowersEngine"
]
