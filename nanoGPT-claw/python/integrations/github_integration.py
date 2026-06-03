# GitHub Integration (用官方包！)
"""
NanoGPT-Claw GitHub Integration
===============================
真实 GitHub 集成，用官方 PyGithub 包
"""
import logging
from typing import Optional, List, Dict, Any
from dataclasses import dataclass
from datetime import datetime
from github import Github, GithubException, Auth
from github.Repository import Repository
from github.Issue import Issue
from github.PullRequest import PullRequest
from github.File import File

from core.config import GitHubConfig
from core.exceptions import GitHubError
from core.logging import get_logger


@dataclass
class IssueInfo:
    """Issue 信息"""
    id: int
    number: int
    title: str
    body: Optional[str]
    state: str
    author: str
    labels: List[str]
    url: str
    created_at: datetime
    updated_at: datetime


@dataclass
class PRInfo:
    """PR 信息"""
    id: int
    number: int
    title: str
    body: Optional[str]
    state: str
    author: str
    source_branch: str
    target_branch: str
    merged: bool
    url: str
    created_at: datetime
    updated_at: datetime


@dataclass
class RepoInfo:
    """仓库信息"""
    id: int
    name: str
    full_name: str
    description: Optional[str]
    stars: int
    forks: int
    language: Optional[str]
    topics: List[str]
    open_issues: int
    url: str
    updated_at: datetime


class GitHubIntegration:
    """GitHub 真实集成"""

    def __init__(self, config: Optional[GitHubConfig] = None):
        """初始化"""
        self._config = config or GitHubConfig()
        self._client: Optional[Github] = None
        self._logger = get_logger("github")

    def initialize(self) -> None:
        """初始化 GitHub 客户端"""
        self._logger.info("初始化 GitHub 集成...")

        if not self._config.token:
            raise GitHubError("GitHub token 未配置")

        try:
            auth = Auth.Token(self._config.token)
            self._client = Github(auth=auth)

            # 验证连接
            user = self._client.get_user()
            self._logger.info(f"  GitHub 登录: {user.login}")
            self._logger.info(f"  已初始化 GitHub 集成")

        except GithubException as e:
            raise GitHubError(f"GitHub 连接失败: {e}") from e

    def is_available(self) -> bool:
        """检查是否可用"""
        return self._client is not None

    def get_repository(self, owner: str, repo: str) -> Optional[RepoInfo]:
        """获取仓库信息（真实）"""
        self._logger.info(f"获取仓库: {owner}/{repo}")
        if not self._client:
            raise GitHubError("GitHub 未初始化")

        try:
            repo_obj = self._client.get_repo(f"{owner}/{repo}")
            return RepoInfo(
                id=repo_obj.id,
                name=repo_obj.name,
                full_name=repo_obj.full_name,
                description=repo_obj.description,
                stars=repo_obj.stargazers_count,
                forks=repo_obj.forks_count,
                language=repo_obj.language,
                topics=repo_obj.get_topics(),
                open_issues=repo_obj.open_issues_count,
                url=repo_obj.html_url,
                updated_at=repo_obj.updated_at
            )
        except GithubException as e:
            self._logger.error(f"获取仓库失败: {e}")
            return None

    def list_user_repos(self, username: Optional[str] = None) -> List[RepoInfo]:
        """列出用户仓库（真实）"""
        self._logger.info(f"列出用户仓库: {username or '当前用户'}")
        if not self._client:
            raise GitHubError("GitHub 未初始化")

        try:
            if username:
                repos = self._client.get_user(username).get_repos()
            else:
                repos = self._client.get_user().get_repos()

            return [
                RepoInfo(
                    id=repo.id,
                    name=repo.name,
                    full_name=repo.full_name,
                    description=repo.description,
                    stars=repo.stargazers_count,
                    forks=repo.forks_count,
                    language=repo.language,
                    topics=repo.get_topics(),
                    open_issues=repo.open_issues_count,
                    url=repo.html_url,
                    updated_at=repo.updated_at
                )
                for repo in repos
            ]
        except GithubException as e:
            self._logger.error(f"列出仓库失败: {e}")
            return []

    def get_issues(self, owner: str, repo: str, state: str = "open") -> List[IssueInfo]:
        """获取仓库 Issues（真实）"""
        self._logger.info(f"获取仓库 Issues: {owner}/{repo}, state={state}")
        if not self._client:
            raise GitHubError("GitHub 未初始化")

        try:
            repo_obj = self._client.get_repo(f"{owner}/{repo}")
            issues = repo_obj.get_issues(state=state)
            return [
                IssueInfo(
                    id=issue.id,
                    number=issue.number,
                    title=issue.title,
                    body=issue.body,
                    state=issue.state,
                    author=issue.user.login,
                    labels=[label.name for label in issue.labels],
                    url=issue.html_url,
                    created_at=issue.created_at,
                    updated_at=issue.updated_at
                )
                for issue in issues
            ]
        except GithubException as e:
            self._logger.error(f"获取 Issues 失败: {e}")
            return []

    def get_pull_requests(self, owner: str, repo: str, state: str = "open") -> List[PRInfo]:
        """获取仓库 PR（真实）"""
        self._logger.info(f"获取仓库 PR: {owner}/{repo}, state={state}")
        if not self._client:
            raise GitHubError("GitHub 未初始化")

        try:
            repo_obj = self._client.get_repo(f"{owner}/{repo}")
            prs = repo_obj.get_pulls(state=state)
            return [
                PRInfo(
                    id=pr.id,
                    number=pr.number,
                    title=pr.title,
                    body=pr.body,
                    state=pr.state,
                    author=pr.user.login,
                    source_branch=pr.head.ref,
                    target_branch=pr.base.ref,
                    merged=pr.merged,
                    url=pr.html_url,
                    created_at=pr.created_at,
                    updated_at=pr.updated_at
                )
                for pr in prs
            ]
        except GithubException as e:
            self._logger.error(f"获取 PR 失败: {e}")
            return []

    def create_issue(self, owner: str, repo: str, title: str, body: Optional[str] = None) -> Optional[IssueInfo]:
        """创建 Issue（真实）"""
        self._logger.info(f"创建 Issue: {owner}/{repo}: {title}")
        if not self._client:
            raise GitHubError("GitHub 未初始化")

        try:
            repo_obj = self._client.get_repo(f"{owner}/{repo}")
            issue = repo_obj.create_issue(title=title, body=body)
            return IssueInfo(
                id=issue.id,
                number=issue.number,
                title=issue.title,
                body=issue.body,
                state=issue.state,
                author=issue.user.login,
                labels=[label.name for label in issue.labels],
                url=issue.html_url,
                created_at=issue.created_at,
                updated_at=issue.updated_at
            )
        except GithubException as e:
            self._logger.error(f"创建 Issue 失败: {e}")
            return None

    def search_repos(self, query: str, sort: str = "stars", order: str = "desc", per_page: int = 30) -> List[RepoInfo]:
        """搜索仓库（真实）"""
        self._logger.info(f"搜索仓库: {query}")
        if not self._client:
            raise GitHubError("GitHub 未初始化")

        try:
            repos = self._client.search_repositories(query, sort=sort, order=order)
            results = []
            for repo in repos[:per_page]:
                results.append(
                    RepoInfo(
                        id=repo.id,
                        name=repo.name,
                        full_name=repo.full_name,
                        description=repo.description,
                        stars=repo.stargazers_count,
                        forks=repo.forks_count,
                        language=repo.language,
                        topics=repo.get_topics(),
                        open_issues=repo.open_issues_count,
                        url=repo.html_url,
                        updated_at=repo.updated_at
                    )
                )
            return results
        except GithubException as e:
            self._logger.error(f"搜索仓库失败: {e}")
            return []

    def get_stats(self) -> Dict[str, Any]:
        """获取统计信息"""
        stats = {
            "is_available": self.is_available(),
        }
        if self._client:
            try:
                user = self._client.get_user()
                stats["user"] = user.login
                stats["public_repos"] = user.public_repos
            except Exception as e:
                self._logger.warning(f"获取用户统计失败: {e}")
        return stats
