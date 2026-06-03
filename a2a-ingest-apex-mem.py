#!/usr/bin/env python3
"""
把 a2a-resources/cache/*/README.md 导入 APEX-MEM。
每个 repo 一条 working 维度的记忆：包含 repo 名、key concepts、URL。
working 维度：当前活跃任务——是补 Ω 短板（working=0 → working>0）最快的办法。
"""
import json
import sys
import urllib.request
from pathlib import Path

CACHE = Path("/Users/lihongxin/.openclaw/workspace/a2a-resources/cache")
# 直发 8767 后端才能传 dimension 字段；bridge 会丢 dimension
APEX_BASE = "http://127.0.0.1:8767"
USER = "xuanji-apex"

# 高价值 repo 优先级（与 APEX 当前方向对齐）
PRIORITY = {
    "mem0ai_mem0": 0.95,           # 记忆框架
    "noahshinn_reflexion": 0.92,   # 反思循环论文
    "langchain-ai_langgraph": 0.90, # 多 agent 编排
    "microsoft_autogen": 0.88,     # 多 agent
    "openai_openai-agents-python": 0.88,  # agent SDK
    "deap_deap": 0.78,             # 进化算法
    "pyg-team_pytorch_geometric": 0.78,  # 图神经网络
    "geek-ai_MAgent": 0.72,        # 多 agent
    "openai_openai-python": 0.85,  # OpenAI SDK
    "langchain-ai_langchain": 0.85, # LLM 框架
    "anthropics_anthropic-sdk-python": 0.82,
    "google-a2a_a2a": 0.80,        # A2A 协议
    "google_generative-ai-python": 0.80,
    "Significant-Gravitas_AutoGPT": 0.75,
    "QwenLM_Qwen-Agent": 0.75,
}


def extract_repo_meta(cache_dir: Path) -> dict:
    """从 cache 目录里提取 repo 元数据。只取标题+首句+关键句，避免 README 模板造成冲突。"""
    readme = cache_dir / "README.md"
    if not readme.exists():
        return {}

    text = readme.read_text(errors="replace")
    repo = cache_dir.name.replace("_", "/", 1)

    # FIX: 只提取前 60 行的标题、首句、关键句。过滤 HTML 模板 (<p align> / <img / <a href)
    lines = []
    for raw in text.splitlines()[:60]:
        line = raw.strip()
        if not line or line.startswith(("<!--", "<p", "<img", "<a ", "![", "|", "---", "<table")):
            continue
        if line.startswith("#"):
            lines.append(line)
        elif len(lines) < 8 and len(line) > 20 and not line.startswith("<"):
            lines.append(line)
        if len(lines) >= 8:
            break

    title = ""
    desc = ""
    for ln in lines:
        if ln.startswith("# ") and not title:
            title = ln[2:].strip()
        elif not desc and ln and not ln.startswith("#") and len(ln) > 20:
            desc = ln[:200]

    return {
        "repo": repo,
        "title": title[:120],
        "desc": desc[:200],
        "url": f"https://github.com/{repo}",
        "priority": PRIORITY.get(cache_dir.name, 0.55),
    }


def main():
    cache_dirs = sorted([d for d in CACHE.iterdir() if d.is_dir()])
    print(f"Found {len(cache_dirs)} cache dirs in a2a-resources")

    ingested = 0
    skipped = 0
    for cache_dir in cache_dirs:
        meta = extract_repo_meta(cache_dir)
        if not meta:
            skipped += 1
            continue

        repo = meta["repo"]
        priority = meta["priority"]
        title = meta.get("title", "")[:120]
        desc = meta.get("desc", "")[:300]

        # 只 ingest 提取的元数据，不带整 README 原文，避免 39 条 A2A 工作记忆被 LongMemEval 误判为冲突
        content = (
            f"[A2A 吸收 {cache_dir.name}] priority={priority} repo={repo}\n"
            f"title: {title}\n"
            f"desc: {desc}\n"
            f"url: {meta['url']}\n"
            f"absorbed_from: a2a-resources/cache/{cache_dir.name}\n"
            f"（working 维度记忆：用于补 Ω 短板）"
        )

        # 走 APEX-MEM 原生 /v1/memories POST 传 dimension 字段
        payload = {
            "content": content,
            "dimension": "working",          # 关键：补 Ω 短板
            "metadata": {
                "dimension": "working",
                "repo": repo,
                "priority": priority,
                "source": "a2a-resources",
                "absorbed_at": "2026-06-03",
            },
        }
        try:
            req = urllib.request.Request(
                f"{APEX_BASE}/v1/memories",
                data=json.dumps(payload).encode(),
                headers={"Content-Type": "application/json"},
                method="POST",
            )
            with urllib.request.urlopen(req, timeout=10) as resp:
                result = json.loads(resp.read())
                ingested += 1
                if ingested % 10 == 0:
                    print(f"  ingested {ingested}/{len(cache_dirs)}")
        except Exception as e:
            print(f"  ERR {repo}: {e}")
            skipped += 1

    print(f"\nDone. ingested={ingested}, skipped={skipped}")


if __name__ == "__main__":
    main()
