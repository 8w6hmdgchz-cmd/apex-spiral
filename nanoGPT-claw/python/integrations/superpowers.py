# SuperPowers Integration (全能整合！)
"""
NanoGPT-Claw SuperPowers Engine
================================
整合所有能力的超级引擎
- GitHub
- AutoResearch
- OpenHands
- 以及更多...
"""
import logging
from typing import Optional, List, Dict, Any, Set
from dataclasses import dataclass
from datetime import datetime

from core.config import Config
from core.logging import get_logger
from core.exceptions import NanoGPTClawError
from .github_integration import GitHubIntegration, RepoInfo
from .auto_research import AutoResearchIntegration, SearchResult, PaperInfo
from .openhands import OpenHandsIntegration, OperationResult


@dataclass
class PowerConfig:
    """能力配置"""
    name: str
    enabled: bool
    priority: int


@dataclass
class PowerResult:
    """单个能力执行结果"""
    power_name: str
    success: bool
    output: str
    metrics: Dict[str, Any]
    duration_ms: int


@dataclass
class SuperResult:
    """超级引擎综合结果"""
    success: bool
    results: List[PowerResult]
    final_output: str
    total_duration_ms: int
    iterations: int


class SuperPowersEngine:
    """SuperPowers 超级整合引擎"""

    # 可用的能力
    AVAILABLE_POWERS = {
        "github": GitHubIntegration,
        "autoresearch": AutoResearchIntegration,
        "openhands": OpenHandsIntegration,
    }

    def __init__(self, config: Optional[Config] = None):
        """初始化"""
        self._config = config or Config()
        self._logger = get_logger("superpowers")
        self._execution_history: List[PowerResult] = []
        self._shared_memory: Dict[str, Any] = {}

        # 初始化各个能力模块
        self._github: Optional[GitHubIntegration] = None
        self._autoresearch: Optional[AutoResearchIntegration] = None
        self._openhands: Optional[OpenHandsIntegration] = None

    async def initialize(self) -> None:
        """初始化各个模块"""
        self._logger.info("初始化 SuperPowers 引擎...")

        # GitHub
        if self._config.github.token:
            self._github = GitHubIntegration(self._config.github)
            try:
                self._github.initialize()
                self._logger.info("  GitHub 已初始化")
            except Exception as e:
                self._logger.error(f"GitHub 初始化失败: {e}")

        # AutoResearch
        self._autoresearch = AutoResearchIntegration(self._config.autoresearch)
        self._logger.info("  AutoResearch 已初始化")

        # OpenHands
        self._openhands = OpenHandsIntegration(self._config.openhands)
        self._logger.info("  OpenHands 已初始化")

    async def close(self) -> None:
        """关闭所有模块"""
        self._logger.info("关闭 SuperPowers 引擎...")
        if self._autoresearch:
            await self._autoresearch.close()
        if self._openhands:
            await self._openhands.close()

    def get_enabled_powers(self) -> List[str]:
        """获取启用的能力列表"""
        powers = []
        if self._github and self._github.is_available():
            powers.append("github")
        if self._autoresearch and self._autoresearch.is_available():
            powers.append("autoresearch")
        if self._openhands and self._openhands.is_available():
            powers.append("openhands")
        return powers

    async def execute_github_task(self, task: str) -> PowerResult:
        """执行 GitHub 相关任务"""
        start = datetime.now()
        self._logger.info(f"[GitHub] 执行任务: {task}")
        output = ""
        success = True
        metrics = {}

        try:
            if not self._github or not self._github.is_available():
                raise NanoGPTClawError("GitHub 不可用")

            # 简单命令处理
            if task.startswith("search:"):
                query = task.split(":", 1)[1]
                repos = self._github.search_repos(query)
                output = f"找到 {len(repos)} 个仓库:\n"
                for repo in repos[:10]:
                    output += f"- {repo.full_name}: {repo.description}\n"
                metrics = {"repos_found": len(repos)}

            elif task.startswith("issues:"):
                parts = task.split("/")
                if len(parts) >= 2:
                    owner = parts[-2]
                    repo = parts[-1]
                    issues = self._github.get_issues(owner, repo)
                    output = f"找到 {len(issues)} 个 issues"
                    metrics = {"issues_found": len(issues)}

            else:
                output = f"不支持的 GitHub 任务: {task}"
                success = False

        except Exception as e:
            self._logger.error(f"GitHub 任务失败: {e}")
            success = False
            output = str(e)

        return PowerResult(
            power_name="github",
            success=success,
            output=output,
            metrics=metrics,
            duration_ms=int((datetime.now() - start).total_seconds() * 1000)
        )

    async def execute_autoresearch_task(self, task: str) -> PowerResult:
        """执行学术研究任务"""
        start = datetime.now()
        self._logger.info(f"[AutoResearch] 执行任务: {task}")
        output = ""
        success = True
        metrics = {}

        try:
            if not self._autoresearch:
                raise NanoGPTClawError("AutoResearch 不可用")

            # 搜索论文
            result = await self._autoresearch.comprehensive_search(task)
            output = f"搜索结果: 找到 {result.total_results} 篇论文 (耗时 {result.search_time_ms}ms)\n"
            for i, paper in enumerate(result.papers[:5], 1):
                output += f"{i}. {paper.title}\n"
            metrics = {
                "papers_found": result.total_results,
                "search_time_ms": result.search_time_ms,
            }

        except Exception as e:
            self._logger.error(f"AutoResearch 任务失败: {e}")
            success = False
            output = str(e)

        return PowerResult(
            power_name="autoresearch",
            success=success,
            output=output,
            metrics=metrics,
            duration_ms=int((datetime.now() - start).total_seconds() * 1000)
        )

    async def execute_openhands_task(self, task: str) -> PowerResult:
        """执行自动化操作任务"""
        start = datetime.now()
        self._logger.info(f"[OpenHands] 执行任务: {task}")
        output = ""
        success = True
        metrics = {}

        try:
            if not self._openhands:
                raise NanoGPTClawError("OpenHands 不可用")

            if task.startswith("read:"):
                file_path = task.split(":", 1)[1]
                result = await self._openhands.read_file(file_path)
                success = result.success
                output = result.output
                if result.error:
                    output += f"\n错误: {result.error}"
                metrics = {"file_size": len(result.output)}

            elif task.startswith("write:"):
                parts = task.split(":", 2)
                file_path = parts[1]
                content = parts[2] if len(parts) > 2 else ""
                result = await self._openhands.write_file(file_path, content)
                success = result.success
                output = result.output
                if result.error:
                    output += f"\n错误: {result.error}"

            elif task.startswith("exec:"):
                command = task.split(":", 1)[1]
                result = await self._openhands.execute_command(command)
                success = result.success
                output = result.output
                if result.error:
                    output += f"\n错误: {result.error}"

            else:
                output = f"不支持的 OpenHands 任务: {task}"
                success = False

        except Exception as e:
            self._logger.error(f"OpenHands 任务失败: {e}")
            success = False
            output = str(e)

        return PowerResult(
            power_name="openhands",
            success=success,
            output=output,
            metrics=metrics,
            duration_ms=int((datetime.now() - start).total_seconds() * 1000)
        )

    async def execute_super_task(self, task: str, enabled_powers: Optional[List[str]] = None) -> SuperResult:
        """执行超级任务（整合多个能力）"""
        start = datetime.now()
        self._logger.info("=" * 80)
        self._logger.info(f"开始执行超级任务: {task}")
        self._logger.info("=" * 80)

        if enabled_powers is None:
            enabled_powers = self.get_enabled_powers()

        results: List[PowerResult] = []
        iterations = 0

        # 简单任务分发
        if "github" in enabled_powers and (task.startswith("github:") or "github" in task):
            if task.startswith("github:"):
                subtask = task.split(":", 1)[1]
            else:
                subtask = task
            result = await self.execute_github_task(subtask)
            results.append(result)
            iterations += 1

        if "autoresearch" in enabled_powers:
            result = await self.execute_autoresearch_task(task)
            results.append(result)
            iterations += 1

        if "openhands" in enabled_powers and (task.startswith("read:") or task.startswith("write:") or task.startswith("exec:")):
            result = await self.execute_openhands_task(task)
            results.append(result)
            iterations += 1

        # 综合结果
        success_count = sum(1 for r in results if r.success)
        overall_success = success_count > 0

        final_output = f"超级任务完成！\n\n"
        for r in results:
            final_output += f"[{r.power_name}]  {'✅成功' if r.success else '❌失败'} ({r.duration_ms}ms):\n"
            final_output += f"{r.output}\n\n"

        total_duration = int((datetime.now() - start).total_seconds() * 1000)
        self._logger.info(f"超级任务完成，耗时 {total_duration}ms")

        return SuperResult(
            success=overall_success,
            results=results,
            final_output=final_output,
            total_duration_ms=total_duration,
            iterations=iterations
        )

    def get_stats(self) -> Dict[str, Any]:
        """获取综合统计"""
        return {
            "enabled_powers": self.get_enabled_powers(),
            "total_executions": len(self._execution_history),
            "shared_memory": dict(self._shared_memory),
        }
