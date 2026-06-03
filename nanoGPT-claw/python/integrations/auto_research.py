# AutoResearch Integration (真实 API！)
"""
NanoGPT-Claw AutoResearch Integration
======================================
真实学术研究集成
- arXiv API
- Semantic Scholar API
"""
import logging
import httpx
from typing import Optional, List, Dict, Any
from dataclasses import dataclass
from datetime import datetime
from core.config import AutoResearchConfig
from core.exceptions import AutoResearchError
from core.logging import get_logger


@dataclass
class PaperInfo:
    """论文信息"""
    id: str
    title: str
    authors: List[str]
    abstract: Optional[str]
    categories: List[str]
    published_date: Optional[str]
    updated_date: Optional[str]
    pdf_url: Optional[str]
    doi: Optional[str]
    citations: int = 0
    influence_score: Optional[float] = None


@dataclass
class SearchResult:
    """搜索结果"""
    query: str
    papers: List[PaperInfo]
    total_results: int
    search_time_ms: int


class AutoResearchIntegration:
    """AutoResearch 真实集成"""

    def __init__(self, config: Optional[AutoResearchConfig] = None):
        """初始化"""
        self._config = config or AutoResearchConfig()
        self._logger = get_logger("autoresearch")
        self._http_client = httpx.AsyncClient(timeout=30.0)

    async def close(self) -> None:
        """关闭 HTTP 客户端"""
        await self._http_client.aclose()

    def is_available(self) -> bool:
        """检查是否可用"""
        return self._config.arxiv_enabled or self._config.semantic_scholar_enabled

    async def search_arxiv(self, query: str, max_results: Optional[int] = None) -> List[PaperInfo]:
        """搜索 arXiv（真实 API）"""
        max_results = max_results or self._config.max_results
        self._logger.info(f"arXiv 搜索: {query} (max={max_results})")

        papers: List[PaperInfo] = []
        try:
            url = "https://export.arxiv.org/api/query"
            params = {
                "search_query": f"all:{query}",
                "start": 0,
                "max_results": max_results,
                "sortBy": "relevance",
                "sortOrder": "descending",
            }

            response = await self._http_client.get(url, params=params)
            response.raise_for_status()

            # 解析 arXiv 响应 (XML)
            # 简单实现，解析成 JSON 更好的库可替换
            papers = self._parse_arxiv_response(response.text)
            self._logger.info(f"arXiv 返回 {len(papers)} 篇论文")

        except httpx.HTTPStatusError as e:
            self._logger.error(f"arXiv 搜索失败: {e}")
        except Exception as e:
            self._logger.error(f"arXiv 搜索异常: {e}")

        return papers

    async def search_semantic_scholar(self, query: str, max_results: Optional[int] = None) -> List[PaperInfo]:
        """搜索 Semantic Scholar（真实 API）"""
        max_results = max_results or self._config.max_results
        self._logger.info(f"Semantic Scholar 搜索: {query} (max={max_results})")

        papers: List[PaperInfo] = []
        try:
            url = "https://api.semanticscholar.org/graph/v1/paper/search"
            params = {
                "query": query,
                "limit": min(max_results, 100),
                "fields": "title,abstract,authors,year,citationCount,externalIds",
            }

            response = await self._http_client.get(url, params=params)
            response.raise_for_status()
            data = response.json()

            # 解析结果
            if data.get("data"):
                papers = [
                    PaperInfo(
                        id=item.get("paperId", ""),
                        title=item.get("title", ""),
                        authors=[
                            a.get("name", "")
                            for a in item.get("authors", [])
                        ],
                        abstract=item.get("abstract"),
                        categories=[],
                        published_date=str(item.get("year")) if item.get("year") else None,
                        updated_date=None,
                        pdf_url=None,
                        doi=item.get("externalIds", {}).get("DOI"),
                        citations=item.get("citationCount", 0),
                    )
                    for item in data["data"]
                ]
                self._logger.info(f"Semantic Scholar 返回 {len(papers)} 篇论文")

        except httpx.HTTPStatusError as e:
            self._logger.error(f"Semantic Scholar 搜索失败: {e}")
        except Exception as e:
            self._logger.error(f"Semantic Scholar 搜索异常: {e}")

        return papers

    async def comprehensive_search(self, query: str, max_results: Optional[int] = None) -> SearchResult:
        """综合搜索（同时查多个源）"""
        max_results = max_results or self._config.max_results
        start_time = datetime.now()
        self._logger.info(f"开始综合学术搜索: {query}")

        all_papers: Dict[str, PaperInfo] = {}

        # arXiv
        if self._config.arxiv_enabled:
            arxiv_papers = await self.search_arxiv(query, max_results)
            for paper in arxiv_papers:
                all_papers[paper.id] = paper

        # Semantic Scholar
        if self._config.semantic_scholar_enabled:
            ss_papers = await self.search_semantic_scholar(query, max_results)
            for paper in ss_papers:
                if paper.id not in all_papers:
                    all_papers[paper.id] = paper
                else:
                    # 合并数据，补充引用数等
                    existing = all_papers[paper.id]
                    if not existing.citations and paper.citations:
                        existing.citations = paper.citations
                    if not existing.doi and paper.doi:
                        existing.doi = paper.doi

        # 排序（按引用数或时间）
        sorted_papers = sorted(
            all_papers.values(),
            key=lambda p: (p.citations, p.published_date),
            reverse=True
        )
        sorted_papers = sorted_papers[:max_results]

        elapsed_ms = int((datetime.now() - start_time).total_seconds() * 1000)

        self._logger.info(f"综合搜索完成: {len(sorted_papers)} 篇 (耗时 {elapsed_ms}ms)")

        return SearchResult(
            query=query,
            papers=sorted_papers,
            total_results=len(sorted_papers),
            search_time_ms=elapsed_ms
        )

    async def get_paper_details(self, paper_id: str) -> Optional[PaperInfo]:
        """获取论文详情（Semantic Scholar）"""
        self._logger.info(f"获取论文详情: {paper_id}")
        try:
            url = f"https://api.semanticscholar.org/graph/v1/paper/{paper_id}"
            params = {
                "fields": "title,abstract,authors,year,citationCount,influentialCitationCount,externalIds,references"
            }
            response = await self._http_client.get(url, params=params)
            response.raise_for_status()
            item = response.json()

            return PaperInfo(
                id=item.get("paperId", ""),
                title=item.get("title", ""),
                authors=[a.get("name", "") for a in item.get("authors", [])],
                abstract=item.get("abstract"),
                categories=[],
                published_date=str(item.get("year")) if item.get("year") else None,
                updated_date=None,
                pdf_url=None,
                doi=item.get("externalIds", {}).get("DOI"),
                citations=item.get("citationCount", 0),
                influence_score=item.get("influentialCitationCount"),
            )
        except Exception as e:
            self._logger.error(f"获取论文详情失败: {e}")
            return None

    def _parse_arxiv_response(self, xml: str) -> List[PaperInfo]:
        """简单解析 arXiv XML 响应"""
        papers: List[PaperInfo] = []
        # 简单解析，实际应用应使用 xml.etree 或 lxml
        # 这里只是演示
        import re
        entries = re.split(r'<entry>', xml)
        for entry in entries[1:]:
            entry = entry.split('</entry>', 1)[0]
            title = re.search(r'<title>(.*?)</title>', entry, re.DOTALL)
            if not title:
                continue
            title = title.group(1).strip()
            id_match = re.search(r'<id>(.*?)</id>', entry)
            if id_match:
                paper_id = id_match.group(1).split('/')[-1]
                papers.append(
                    PaperInfo(
                        id=paper_id,
                        title=title,
                        authors=[],
                        abstract=None,
                        categories=[],
                        published_date=None,
                        updated_date=None,
                        pdf_url=None,
                        doi=None,
                    )
                )
        return papers

    def get_stats(self) -> Dict[str, Any]:
        """获取统计"""
        return {
            "is_available": self.is_available(),
            "arxiv_enabled": self._config.arxiv_enabled,
            "semantic_scholar_enabled": self._config.semantic_scholar_enabled,
        }
