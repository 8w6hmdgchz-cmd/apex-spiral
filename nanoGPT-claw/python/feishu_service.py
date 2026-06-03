"""
NanoGPT-Claw - 飞书服务（分层架构）
========================================

真实飞书服务 - 使用官方 lark-oapi-sdk
这是 Python 层的完整飞书服务，Rust 层可以通过命令调用此服务

Usage:
    python feishu_service.py start          # 启动飞书 WebSocket 服务
    python feishu_service.py send <msg>     # 发送飞书消息
    python feishu_service.py status         # 查看状态
"""
import asyncio
import sys
import os
import json
import argparse
from typing import Dict, Any, Optional, List
from datetime import datetime

# 添加项目路径
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from integrations.feishu_integration import (
    FeishuIntegration,
    FeishuMessage,
    FeishuResult,
    FeishuEvent
)
from core.config import FeishuConfig
from core.logging import get_logger

logger = get_logger("feishu_service")


class FeishuService:
    """完整的飞书服务（Python 层）"""

    def __init__(self):
        """初始化"""
        self.config = FeishuConfig()
        self.integration: Optional[FeishuIntegration] = None
        self._running = False
        self._setup_from_env()

    def _setup_from_env(self):
        """从环境变量设置配置"""
        if os.getenv("FEISHU_APP_ID"):
            self.config.app_id = os.getenv("FEISHU_APP_ID")
        if os.getenv("FEISHU_APP_SECRET"):
            self.config.app_secret = os.getenv("FEISHU_APP_SECRET")
        if os.getenv("FEISHU_RECEIVE_ID"):
            self.config.receive_id = os.getenv("FEISHU_RECEIVE_ID")
        if os.getenv("FEISHU_RECEIVE_ID_TYPE"):
            self.config.receive_id_type = os.getenv("FEISHU_RECEIVE_ID_TYPE")

    async def initialize(self) -> bool:
        """初始化飞书服务"""
        logger.info("正在初始化飞书服务...")

        try:
            self.integration = FeishuIntegration(self.config)

            # 注册默认事件处理器
            self.integration.register_event_handler(self._handle_incoming_message)

            stats = await self.integration.get_stats()
            logger.info(f"飞书服务状态: {stats}")

            if not stats["is_available"]:
                logger.warning(
                    "⚠️  飞书 App ID 或 App Secret 未配置，请设置环境变量：\n"
                    "  - FEISHU_APP_ID\n"
                    "  - FEISHU_APP_SECRET\n"
                    "  - FEISHU_RECEIVE_ID\n"
                    "  - FEISHU_RECEIVE_ID_TYPE (可选: open_id, chat_id, user_id, union_id, email)"
                )

            return True

        except Exception as e:
            logger.error(f"初始化失败: {e}")
            return False

    async def _handle_incoming_message(self, event: FeishuEvent):
        """处理收到的飞书消息"""
        logger.info(f"📨 收到消息: {event}")

        if event.content and event.sender_id:
            # 这里可以添加处理逻辑
            # 比如调用 Rust 层的消息处理
            logger.info(f"  发送者: {event.sender_id}")
            logger.info(f"  内容: {event.content}")

    async def send_text_message(self, text: str, receive_id: Optional[str] = None) -> Dict[str, Any]:
        """发送文本消息"""
        if not self.integration:
            return {
                "success": False,
                "error": "服务未初始化",
                "timestamp": datetime.now().isoformat()
            }

        target_id = receive_id or self.config.receive_id
        if not target_id:
            return {
                "success": False,
                "error": "未指定接收者，请设置 FEISHU_RECEIVE_ID 或传入 receive_id",
                "timestamp": datetime.now().isoformat()
            }

        message = FeishuMessage(
            content=text,
            message_type="text",
            receive_id_type=self.config.receive_id_type,
            receive_id=target_id
        )

        result = await self.integration.send_message(message)

        return {
            "success": result.success,
            "output": result.output,
            "error": result.error,
            "duration_ms": result.duration_ms,
            "timestamp": datetime.now().isoformat()
        }

    async def start_websocket(self):
        """启动 WebSocket 服务"""
        if not self.integration:
            logger.error("服务未初始化")
            return

        self._running = True
        logger.info("🚀 启动飞书 WebSocket 服务...")

        success = await self.integration.start_websocket()
        if success:
            logger.info("✅ WebSocket 服务已启动，按 Ctrl+C 停止...")
            try:
                while self._running:
                    await asyncio.sleep(1)
            except KeyboardInterrupt:
                logger.info("收到停止信号...")
            finally:
                await self.integration.close()
                self._running = False
        else:
            logger.error("❌ WebSocket 启动失败")

    async def get_status(self) -> Dict[str, Any]:
        """获取服务状态"""
        if not self.integration:
            return {
                "running": False,
                "available": False,
                "timestamp": datetime.now().isoformat()
            }

        stats = await self.integration.get_stats()
        return {
            "running": self._running,
            "available": stats.get("is_available", False),
            "sdk_available": stats.get("sdk_available", False),
            "client_initialized": stats.get("client_initialized", False),
            "websocket_connected": stats.get("websocket_connected", False),
            "config": {
                "app_id": "***" if self.config.app_id else None,
                "receive_id_type": self.config.receive_id_type,
            },
            "timestamp": datetime.now().isoformat()
        }


async def main():
    """主入口"""
    parser = argparse.ArgumentParser(
        description="NanoGPT-Claw 飞书服务（Python 层，官方 SDK）"
    )
    subparsers = parser.add_subparsers(dest="command", help="命令")

    # Start command
    start_parser = subparsers.add_parser("start", help="启动飞书 WebSocket 服务")

    # Send command
    send_parser = subparsers.add_parser("send", help="发送飞书消息")
    send_parser.add_argument("message", help="消息内容")
    send_parser.add_argument("--receive-id", help="接收者 ID（可选）")

    # Status command
    status_parser = subparsers.add_parser("status", help="查看服务状态")

    args = parser.parse_args()

    # 初始化服务
    service = FeishuService()
    initialized = await service.initialize()

    if not initialized and args.command != "status":
        print("❌ 服务初始化失败")
        return 1

    if args.command == "start":
        await service.start_websocket()

    elif args.command == "send":
        result = await service.send_text_message(args.message, args.receive_id)
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 0 if result["success"] else 1

    elif args.command == "status":
        status = await service.get_status()
        print(json.dumps(status, ensure_ascii=False, indent=2))

    else:
        parser.print_help()

    return 0


if __name__ == "__main__":
    exit_code = asyncio.run(main())
    sys.exit(exit_code)
