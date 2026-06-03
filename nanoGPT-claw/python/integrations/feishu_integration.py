# Feishu Integration (官方飞书包！)
"""
NanoGPT-Claw Feishu Integration
=================================
真实飞书集成
- 消息发送
- 事件处理
- 文件上传
"""
import logging
from typing import Optional, List, Dict, Any
from dataclasses import dataclass
from datetime import datetime
import httpx

from core.config import FeishuConfig
from core.logging import get_logger


@dataclass
class FeishuMessage:
    """飞书消息"""
    content: str
    message_type: str = "text"
    receive_id_type: str = "open_id"
    receive_id: Optional[str] = None


@dataclass
class FeishuResult:
    """飞书操作结果"""
    success: bool
    output: str
    error: Optional[str]
    duration_ms: int


class FeishuIntegration:
    """Feishu 真实集成"""

    def __init__(self, config: Optional[FeishuConfig] = None):
        """初始化"""
        self._config = config or FeishuConfig()
        self._logger = get_logger("feishu")
        self._http_client = httpx.AsyncClient(timeout=30.0)

        self._tenant_access_token: Optional[str] = None
        self._token_expires_at: float = 0.0

    async def close(self) -> None:
        """关闭 HTTP 客户端"""
        await self._http_client.aclose()

    async def get_tenant_access_token(self) -> Optional[str]:
        """获取租户访问令牌"""
        if self._tenant_access_token and datetime.now().timestamp() < self._token_expires_at:
            return self._tenant_access_token

        if not self._config.app_id or not self._config.app_secret:
            raise ValueError("飞书 App ID 和 App Secret 未配置")

        try:
            url = "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal"
            response = await self._http_client.post(
                url,
                json={
                    "app_id": self._config.app_id,
                    "app_secret": self._config.app_secret
                }
            )
            response.raise_for_status()
            data = response.json()

            if data.get("code") == 0:
                self._tenant_access_token = data.get("tenant_access_token")
                self._token_expires_at = (
                    datetime.now().timestamp() + data.get("expire", 7200) - 300
                )
                return self._tenant_access_token

        except Exception as e:
            self._logger.error(f"获取访问令牌失败: {e}")

        return None

    async def send_message(self, message: FeishuMessage) -> FeishuResult:
        """发送飞书消息（真实）"""
        start = datetime.now()
        self._logger.info(f"发送飞书消息: {message.content[:50]}...")

        try:
            token = await self.get_tenant_access_token()
            if not token:
                return FeishuResult(
                    success=False,
                    output="",
                    error="无法获取访问令牌",
                    duration_ms=int((datetime.now() - start).total_seconds() * 1000),
                )

            url = "https://open.feishu.cn/open-apis/im/v1/messages"
            headers = {
                "Authorization": f"Bearer {token}",
                "Content-Type": "application/json",
            }

            body = {
                "receive_id": message.receive_id or "",
                "msg_type": message.message_type,
                "content": '{"text":"' + message.content.replace('"', '\\"') + '"}',
            }

            response = await self._http_client.post(url, headers=headers, json=body)
            result_data = response.json()

            success = result_data.get("code") == 0
            output = str(result_data)
            error = result_data.get("msg") if not success else None

            self._logger.info(f"消息发送成功: {success}")

            return FeishuResult(
                success=success,
                output=output,
                error=error,
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
            )

        except Exception as e:
            self._logger.error(f"发送飞书消息失败: {e}")
            return FeishuResult(
                success=False,
                output="",
                error=str(e),
                duration_ms=int((datetime.now() - start).total_seconds() * 1000),
            )

    async def get_stats(self) -> Dict[str, Any]:
        """获取统计"""
        return {
            "is_available": bool(self._config.app_id and self._config.app_secret),
        }
