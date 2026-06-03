# Configuration Module
"""
NanoGPT-Claw Configuration
==========================
高质量配置系统
"""
import os
from dataclasses import dataclass, field
from typing import Optional, List, Dict, Any
from pathlib import Path
from dotenv import load_dotenv

# Load environment variables from .env file
load_dotenv()


@dataclass
class LLMConfig:
    """LLM 配置"""
    provider: str = "openai"  # openai, anthropic, ollama
    model: str = "gpt-4o"
    api_key: Optional[str] = None
    base_url: Optional[str] = None
    temperature: float = 0.7
    max_tokens: int = 2000
    timeout: int = 120


@dataclass
class FeishuConfig:
    """飞书配置（用官方包）"""
    app_id: Optional[str] = None
    app_secret: Optional[str] = None
    verification_token: Optional[str] = None
    encrypt_key: Optional[str] = None
    webhook_url: Optional[str] = None


@dataclass
class GitHubConfig:
    """GitHub配置（用官方包）"""
    token: Optional[str] = None
    webhook_secret: Optional[str] = None
    app_id: Optional[str] = None
    private_key: Optional[str] = None


@dataclass
class AutoResearchConfig:
    """AutoResearch配置"""
    arxiv_enabled: bool = True
    semantic_scholar_enabled: bool = True
    serp_api_key: Optional[str] = None
    max_results: int = 10


@dataclass
class OpenHandsConfig:
    """OpenHands配置"""
    workspace_dir: Path = Path("./workspace")
    sandbox_enabled: bool = True
    max_command_duration: int = 300
    allow_destructive: bool = False


@dataclass
class Config:
    """主配置"""
    llm: LLMConfig = field(default_factory=LLMConfig)
    feishu: FeishuConfig = field(default_factory=FeishuConfig)
    github: GitHubConfig = field(default_factory=GitHubConfig)
    autoresearch: AutoResearchConfig = field(default_factory=AutoResearchConfig)
    openhands: OpenHandsConfig = field(default_factory=OpenHandsConfig)
    debug: bool = False
    log_level: str = "INFO"


def get_config() -> Config:
    """从环境变量加载配置"""
    config = Config()

    # LLM
    config.llm.provider = os.getenv("LLM_PROVIDER", "openai")
    config.llm.model = os.getenv("LLM_MODEL", "gpt-4o")
    config.llm.api_key = os.getenv("OPENAI_API_KEY") or os.getenv("ANTHROPIC_API_KEY")
    config.llm.base_url = os.getenv("LLM_BASE_URL")

    # Feishu
    config.feishu.app_id = os.getenv("FEISHU_APP_ID")
    config.feishu.app_secret = os.getenv("FEISHU_APP_SECRET")
    config.feishu.verification_token = os.getenv("FEISHU_VERIFICATION_TOKEN")
    config.feishu.encrypt_key = os.getenv("FEISHU_ENCRYPT_KEY")
    config.feishu.webhook_url = os.getenv("FEISHU_WEBHOOK_URL")

    # GitHub
    config.github.token = os.getenv("GITHUB_TOKEN")
    config.github.webhook_secret = os.getenv("GITHUB_WEBHOOK_SECRET")
    config.github.app_id = os.getenv("GITHUB_APP_ID")
    config.github.private_key = os.getenv("GITHUB_PRIVATE_KEY")

    # AutoResearch
    config.autoresearch.serp_api_key = os.getenv("SERP_API_KEY")
    autoresearch_max_results = os.getenv("AUTORESEARCH_MAX_RESULTS", "10")
    if autoresearch_max_results.isdigit():
        config.autoresearch.max_results = int(autoresearch_max_results)

    # OpenHands
    workspace_dir = os.getenv("OPENHANDS_WORKSPACE_DIR")
    if workspace_dir:
        config.openhands.workspace_dir = Path(workspace_dir)
    sandbox_enabled = os.getenv("OPENHANDS_SANDBOX_ENABLED", "true")
    config.openhands.sandbox_enabled = sandbox_enabled.lower() in ("true", "1", "yes")

    # Debug
    config.debug = os.getenv("DEBUG", "false").lower() in ("true", "1", "yes")
    config.log_level = os.getenv("LOG_LEVEL", "INFO")

    return config
