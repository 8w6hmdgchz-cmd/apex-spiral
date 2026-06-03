# OpenHands Integration (真实操作！)
"""
NanoGPT-Claw OpenHands Integration
===================================
真实自动化操作
- 文件操作
- 命令执行
- HTTP 请求
"""
import logging
import asyncio
import os
import shlex
import subprocess
from pathlib import Path
from typing import Optional, List, Dict, Any
from dataclasses import dataclass
from datetime import datetime
import httpx

from core.config import OpenHandsConfig
from core.exceptions import OpenHandsError
from core.logging import get_logger


@dataclass
class OperationResult:
    """操作结果"""
    operation_type: str
    success: bool
    output: str
    error: Optional[str]
    duration_ms: int
    command: Optional[str]
    file_path: Optional[str]
    url: Optional[str]


class OpenHandsIntegration:
    """OpenHands 真实集成"""

    def __init__(self, config: Optional[OpenHandsConfig] = None):
        """初始化"""
        self._config = config or OpenHandsConfig()
        self._logger = get_logger("openhands")
        self._http_client = httpx.AsyncClient(timeout=300.0)

    async def close(self) -> None:
        """关闭 HTTP 客户端"""
        await self._http_client.aclose()

    def is_available(self) -> bool:
        """检查是否可用"""
        return True

    async def read_file(self, file_path: str) -> OperationResult:
        """读取文件（真实）"""
        start = datetime.now()
        self._logger.info(f"读取文件: {file_path}")
        path = Path(file_path)

        try:
            if not path.exists():
                return OperationResult(
                    operation_type="read",
                    success=False,
                    output="",
                    error=f"文件不存在: {file_path}",
                    duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                    command=None,
                    file_path=file_path,
                    url=None,
                )
            content = await asyncio.to_thread(path.read_text)
            self._logger.info(f"  成功读取 {len(content)} 字节")

            return OperationResult(
                operation_type="read",
                success=True,
                output=content,
                error=None,
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                command=None,
                file_path=file_path,
                url=None,
            )
        except Exception as e:
            self._logger.error(f"读取失败: {e}")
            return OperationResult(
                operation_type="read",
                success=False,
                output="",
                error=str(e),
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                command=None,
                file_path=file_path,
                url=None,
            )

    async def write_file(self, file_path: str, content: str) -> OperationResult:
        """写入文件（真实）"""
        start = datetime.now()
        self._logger.info(f"写入文件: {file_path} ({len(content)} 字节)")
        path = Path(file_path)

        try:
            # 检查是否允许覆盖
            if path.exists() and not self._config.allow_destructive:
                return OperationResult(
                    operation_type="write",
                    success=False,
                    output="",
                    error="文件已存在且不允许覆盖",
                    duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                    command=None,
                    file_path=file_path,
                    url=None,
                )

            if path.parent:
                await asyncio.to_thread(path.parent.mkdir, parents=True, exist_ok=True)

            await asyncio.to_thread(path.write_text, content)
            self._logger.info("  成功写入")

            return OperationResult(
                operation_type="write",
                success=True,
                output=f"成功写入 {len(content)} 字节",
                error=None,
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                command=None,
                file_path=file_path,
                url=None,
            )
        except Exception as e:
            self._logger.error(f"写入失败: {e}")
            return OperationResult(
                operation_type="write",
                success=False,
                output="",
                error=str(e),
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                command=None,
                file_path=file_path,
                url=None,
            )

    async def delete_file(self, file_path: str) -> OperationResult:
        """删除文件（真实）"""
        start = datetime.now()
        self._logger.info(f"删除文件: {file_path}")
        path = Path(file_path)

        try:
            if not path.exists():
                return OperationResult(
                    operation_type="delete",
                    success=False,
                    output="",
                    error=f"文件不存在: {file_path}",
                    duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                    command=None,
                    file_path=file_path,
                    url=None,
                )

            if not self._config.allow_destructive:
                return OperationResult(
                    operation_type="delete",
                    success=False,
                    output="",
                    error="不允许破坏性操作",
                    duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                    command=None,
                    file_path=file_path,
                    url=None,
                )

            await asyncio.to_thread(path.unlink)
            self._logger.info("  成功删除")

            return OperationResult(
                operation_type="delete",
                success=True,
                output="文件已删除",
                error=None,
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                command=None,
                file_path=file_path,
                url=None,
            )
        except Exception as e:
            self._logger.error(f"删除失败: {e}")
            return OperationResult(
                operation_type="delete",
                success=False,
                output="",
                error=str(e),
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                command=None,
                file_path=file_path,
                url=None,
            )

    async def execute_command(self, command: str, cwd: Optional[str] = None) -> OperationResult:
        """执行命令（真实）"""
        start = datetime.now()
        self._logger.info(f"执行命令: {command}")
        cwd = cwd or self._config.workspace_dir

        try:
            # 使用 subprocess 执行
            args = shlex.split(command)
            result = await asyncio.create_subprocess_exec(
                *args,
                cwd=cwd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )

            stdout, stderr = await asyncio.wait_for(
                result.communicate(),
                timeout=self._config.max_command_duration
            )

            success = result.returncode == 0
            output = stdout
            error = stderr if stderr else None

            self._logger.info(f"  命令完成 (exit code: {result.returncode})")

            return OperationResult(
                operation_type="execute",
                success=success,
                output=output,
                error=error,
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                command=command,
                file_path=None,
                url=None,
            )

        except asyncio.TimeoutError:
            self._logger.error("  命令超时")
            return OperationResult(
                operation_type="execute",
                success=False,
                output="",
                error="命令执行超时",
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                command=command,
                file_path=None,
                url=None,
            )

        except Exception as e:
            self._logger.error(f"  命令执行失败: {e}")
            return OperationResult(
                operation_type="execute",
                success=False,
                output="",
                error=str(e),
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                command=command,
                file_path=None,
                url=None,
            )

    async def http_request(self, method: str, url: str, body: Optional[str] = None) -> OperationResult:
        """执行 HTTP 请求（真实）"""
        start = datetime.now()
        self._logger.info(f"HTTP 请求: {method} {url}")

        try:
            request_kwargs: Dict[str, Any] = {}
            if method in ("POST", "PUT", "PATCH") and body:
                request_kwargs["json"] = {"data": body}

            response = await self._http_client.request(
                method.upper(),
                url,
                **request_kwargs
            )

            output = f"状态: {response.status_code}\n\n{response.text}"
            success = response.status_code < 400

            return OperationResult(
                operation_type="http",
                success=success,
                output=output,
                error=None if success else f"HTTP 错误: {response.status_code}",
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                command=None,
                file_path=None,
                url=url,
            )

        except Exception as e:
            self._logger.error(f"HTTP 请求失败: {e}")
            return OperationResult(
                operation_type="http",
                success=False,
                output="",
                error=str(e),
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                command=None,
                file_path=None,
                url=url,
            )

    def get_stats(self) -> Dict[str, Any]:
        """获取统计"""
        return {
            "is_available": self.is_available(),
            "sandbox_enabled": self._config.sandbox_enabled,
            "workspace_dir": str(self._config.workspace_dir),
        }
