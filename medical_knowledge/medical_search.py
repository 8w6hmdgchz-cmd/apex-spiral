#!/usr/bin/env python3
"""
APEX 医学知识库集成
====================
集成医学文献、GSEA分析、Meta分析到APEX系统

功能：
- 医学文献检索
- GSEA通路分析查询
- Meta分析结果
- 统计计算
"""

import os
import json
from pathlib import Path
from typing import Dict, List, Optional
import subprocess

# 知识库路径
BASE_PATH = Path.home() / ".openclaw" / "workspace" / "medical_knowledge"
MEDICAL_KB = BASE_PATH / "medical_knowledge"
GSE25252 = BASE_PATH / "GSE25252"
RESEARCH = BASE_PATH / "research_projects"

class MedicalKnowledgeBase:
    """医学知识库"""
    
    def __init__(self):
        self.base_path = BASE_PATH
        self.available = all([
            MEDICAL_KB.exists(),
            GSE25252.exists(),
            RESEARCH.exists()
        ])
    
    def search_papers(self, query: str, max_results: int = 10) -> List[Dict]:
        """搜索医学文献"""
        results = []
        
        # 搜索 medical-knowledge 目录
        if MEDICAL_KB.exists():
            for md_file in MEDICAL_KB.rglob("*.md"):
                try:
                    content = md_file.read_text()
                    if query.lower() in content.lower():
                        results.append({
                            "file": str(md_file.relative_to(HOME)),
                            "type": "knowledge",
                            "path": str(md_file)
                        })
                except:
                    pass
        
        return results[:max_results]
    
    def get_gsea_pathways(self, comparison: str = "HZacute_vs_HZresolved") -> List[Dict]:
        """获取GSEA通路分析结果"""
        pathways = []
        hallmark_dir = GSE25252 / "GSEA" / comparison / "Hallmark" / "prerank"
        kegg_dir = GSE25252 / "GSEA" / comparison / "KEGG" / "prerank"
        
        for pdf_file in hallmark_dir.glob("*.pdf"):
            pathways.append({
                "name": pdf_file.stem,
                "type": "Hallmark",
                "file": str(pdf_file)
            })
        
        for pdf_file in kegg_dir.glob("*.pdf"):
            pathways.append({
                "name": pdf_file.stem,
                "type": "KEGG", 
                "file": str(pdf_file)
            })
        
        return pathways
    
    def get_deseq2_results(self, comparison: str = "HZacute_vs_HZresolved") -> Dict:
        """获取DESeq2差异分析结果"""
        sig_file = GSE25252 / f"{comparison}_sig.csv"
        all_file = GSE25252 / f"{comparison}_all.csv"
        
        results = {}
        if sig_file.exists():
            results["significant"] = str(sig_file)
        if all_file.exists():
            results["all_genes"] = str(all_file)
        
        return results
    
    def list_meta_analyses(self) -> List[Dict]:
        """列出Meta分析项目"""
        meta_projects = []
        
        # 查找Meta分析目录
        for meta_dir in (Path.home() / "Desktop").glob("*Meta*"):
            if meta_dir.is_dir():
                meta_projects.append({
                    "name": meta_dir.name,
                    "path": str(meta_dir)
                })
        
        return meta_projects
    
    def search_research_projects(self, query: str) -> List[Dict]:
        """搜索科研项目"""
        results = []
        
        if RESEARCH.exists():
            for doc in RESEARCH.glob("*.docx"):
                if query.lower() in doc.name.lower():
                    results.append({
                        "name": doc.name,
                        "type": "application",
                        "path": str(doc)
                    })
        
        return results
    
    def get_stats(self) -> Dict:
        """获取知识库统计"""
        gsea_count = 0
        for pdf in GSE25252.rglob("*.pdf"):
            gsea_count += 1
        
        md_count = len(list(MEDICAL_KB.rglob("*.md"))) if MEDICAL_KB.exists() else 0
        
        return {
            "available": self.available,
            "gsea_pathways": gsea_count,
            "medical_kb_docs": md_count,
            "meta_projects": len(self.list_meta_analyses()),
            "paths": {
                "medical_knowledge": str(MEDICAL_KB),
                "GSE25252": str(GSE25252),
                "research": str(RESEARCH)
            }
        }


# 全局实例
_kb = None

def get_medical_kb() -> MedicalKnowledgeBase:
    global _kb
    if _kb is None:
        _kb = MedicalKnowledgeBase()
    return _kb


if __name__ == "__main__":
    kb = get_medical_kb()
    
    print("=== 医学知识库状态 ===")
    stats = kb.get_stats()
    print(f"可用: {stats['available']}")
    print(f"GSEA通路: {stats['gsea_pathways']}")
    print(f"医学知识: {stats['medical_kb_docs']}")
    print(f"Meta分析: {stats['meta_projects']}")
    
    print("\n=== GSEA 通路示例 ===")
    pathways = kb.get_gsea_pathways()
    print(f"找到 {len(pathways)} 个通路")
    for p in pathways[:5]:
        print(f"  [{p['type']}] {p['name']}")
