#!/usr/bin/env python3
"""
APEX Medical Knowledge API
==========================
医学知识库 API 服务，集成到 APEX 系统
"""

import json
from flask import Flask, request, jsonify
from medical_search import get_medical_kb

app = Flask(__name__)
kb = get_medical_kb()

@app.route("/health")
def health():
    return jsonify({"status": "ok", "service": "apex-medical"})

@app.route("/medical/stats")
def stats():
    """获取医学知识库统计"""
    return jsonify(kb.get_stats())

@app.route("/medical/search")
def search():
    """搜索医学文献"""
    query = request.args.get("q", "")
    if not query:
        return jsonify({"error": "q parameter required"})
    
    results = kb.search_papers(query)
    return jsonify({"query": query, "results": results, "count": len(results)})

@app.route("/medical/gsea")
def gsea():
    """获取GSEA通路分析"""
    comparison = request.args.get("comparison", "HZacute_vs_HZresolved")
    pathways = kb.get_gsea_pathways(comparison)
    return jsonify({
        "comparison": comparison,
        "pathways": pathways,
        "count": len(pathways)
    })

@app.route("/medical/deseq2")
def deseq2():
    """获取DESeq2分析结果"""
    comparison = request.args.get("comparison", "HZacute_vs_HZresolved")
    results = kb.get_deseq2_results(comparison)
    return jsonify({"comparison": comparison, "results": results})

@app.route("/medical/meta")
def meta():
    """列出Meta分析项目"""
    projects = kb.list_meta_analyses()
    return jsonify({"projects": projects, "count": len(projects)})

@app.route("/medical/research/search")
def research_search():
    """搜索科研项目"""
    query = request.args.get("q", "")
    if not query:
        return jsonify({"error": "q parameter required"})
    
    results = kb.search_research_projects(query)
    return jsonify({"query": query, "results": results, "count": len(results)})

if __name__ == "__main__":
    print("🏥 APEX Medical Knowledge API starting on :18523")
    app.run(host="127.0.0.1", port=18523, debug=False)
