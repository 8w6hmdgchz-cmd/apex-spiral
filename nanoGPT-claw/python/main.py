# NanoGPT-Claw Python Integration Layer
"""
NanoGPT-Claw Python Integration Layer
======================================

使用方法:
    python main.py --help
    python main.py github-search "query"
    python main.py autoresearch "topic"
    python main.py openhands read "file.txt"
"""
import asyncio
import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from core.config import get_config
from core.logging import setup_logging
from integrations import (
    GitHubIntegration,
    AutoResearchIntegration,
    OpenHandsIntegration,
    SuperPowersEngine,
    FeishuIntegration,
)


async def main():
    parser = argparse.ArgumentParser(description="NanoGPT-Claw Python Integration Layer")
    subparsers = parser.add_subparsers(title="commands", dest="command")

    # GitHub
    github_parser = subparsers.add_parser("github-search", help="搜索 GitHub 仓库")
    github_parser.add_argument("query", type=str, help="搜索关键词")

    # AutoResearch
    ar_parser = subparsers.add_parser("autoresearch", help="学术研究搜索")
    ar_parser.add_argument("query", type=str, help="搜索关键词")

    # OpenHands
    oh_parser = subparsers.add_parser("openhands", help="执行 OpenHands 任务")
    oh_parser.add_argument("subcommand", type=str, help="子命令: read, write, exec")
    oh_parser.add_argument("arg", type=str, help="参数")
    oh_parser.add_argument("--content", type=str, help="写入内容 (仅用于 write)")

    # 超级任务
    super_parser = subparsers.add_parser("super-task", help="执行超级任务")
    super_parser.add_argument("task", type=str, help="任务描述")

    # 版本
    parser.add_argument("--version", action="version", version="0.1.0")

    args = parser.parse_args()

    # 初始化
    config = get_config()
    setup_logging(config.log_level)

    if args.command == "github-search":
        if not config.github.token:
            print("需要配置 GITHUB_TOKEN 环境变量")
            sys.exit(1)

        github = GitHubIntegration(config.github)
        github.initialize()
        print(f"搜索: {args.query}")
        repos = github.search_repos(args.query)
        print()
        for repo in repos[:20]:
            print(f"⭐ {repo.stars:6} | {repo.full_name}")
            print(f"   {repo.description}")
            print()

    elif args.command == "autoresearch":
        ar = AutoResearchIntegration(config.autoresearch)
        try:
            result = await ar.comprehensive_search(args.query)
            print()
            print("=" * 80)
            print(f"搜索结果: {result.total_results} 篇论文 (耗时 {result.search_time_ms}ms)")
            print("=" * 80)
            for i, paper in enumerate(result.papers[:10], 1):
                print()
                print(f"{i}. {paper.title}")
                if paper.published_date:
                    print(f"   日期: {paper.published_date}")
                if paper.citations > 0:
                    print(f"   引用: {paper.citations}")
        finally:
            await ar.close()

    elif args.command == "openhands":
        oh = OpenHandsIntegration(config.openhands)
        try:
            if args.subcommand == "read":
                result = await oh.read_file(args.arg)
                print()
                if result.success:
                    print(f"文件内容 ({len(result.output)} bytes):")
                    print(result.output)
                else:
                    print(f"错误: {result.error}")

            elif args.subcommand == "write":
                content = args.content or ""
                result = await oh.write_file(args.arg, content)
                print()
                if result.success:
                    print(f"成功写入: {result.output}")
                else:
                    print(f"错误: {result.error}")

            elif args.subcommand == "exec":
                result = await oh.execute_command(args.arg)
                print()
                print(f"输出: {result.output}")
                if result.error:
                    print(f"错误: {result.error}")
        finally:
            await oh.close()

    elif args.command == "super-task":
        engine = SuperPowersEngine(config)
        await engine.initialize()
        try:
            result = await engine.execute_super_task(args.task)
            print()
            print("=" * 80)
            print(result.final_output)
            print("=" * 80)
        finally:
            await engine.close()

    else:
        parser.print_help()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("退出")
