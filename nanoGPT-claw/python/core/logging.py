# Logging Module
"""
NanoGPT-Claw Logging
====================
高质量日志系统
"""
import logging
import sys
from typing import Optional
from rich.console import Console
from rich.logging import RichHandler


def setup_logging(level: str = "INFO") -> logging.Logger:
    """设置高质量日志系统"""
    # 创建 logger
    logger = logging.getLogger("nanogpt-claw")
    logger.setLevel(getattr(logging, level.upper(), logging.INFO))

    # Rich 格式化的处理器
    rich_handler = RichHandler(
        console=Console(),
        rich_tracebacks=True,
        tracebacks_show_locals=False,
        markup=True,
    )
    rich_handler.setFormatter(
        logging.Formatter(
            "%(asctime)s | %(name)s | %(levelname)s | %(message)s",
            datefmt="[%Y-%m-%d %H:%M:%S]"
        )
    )

    # 添加处理器
    if not logger.handlers:
        logger.addHandler(rich_handler)
    logger.propagate = False

    return logger


def get_logger(name: str) -> logging.Logger:
    """获取子模块 logger"""
    return logging.getLogger(f"nanogpt-claw.{name}")
