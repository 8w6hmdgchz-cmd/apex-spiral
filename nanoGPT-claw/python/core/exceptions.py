# Exceptions Module
"""
NanoGPT-Claw Exceptions
========================
自定义异常
"""


class NanoGPTClawError(Exception):
    """基类异常"""
    pass


class ConfigError(NanoGPTClawError):
    """配置错误"""
    pass


class IntegrationError(NanoGPTClawError):
    """集成错误"""
    pass


class LLMError(NanoGPTClawError):
    """LLM 调用错误"""
    pass


class FeishuError(IntegrationError):
    """飞书集成错误"""
    pass


class GitHubError(IntegrationError):
    """GitHub集成错误"""
    pass


class AutoResearchError(IntegrationError):
    """AutoResearch 错误"""
    pass


class OpenHandsError(IntegrationError):
    """OpenHands 错误"""
    pass
