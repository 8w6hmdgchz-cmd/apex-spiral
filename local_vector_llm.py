#!/usr/bin/env python3
"""APEX本地向量服务 - 使用本机Ollama nomic-embed-text"""
import requests
import json
from flask import Flask, request, jsonify
from threading import Lock
import time

app = Flask(__name__)
OLLAMA = "http://localhost:11434"
MODEL = "nomic-embed-text"

# 向量内存
vector_store = {}
store_lock = Lock()

def cosine_sim(a, b):
    return sum(x * y for x, y in zip(a, b)) / (
        sum(x * x for x in a) ** 0.5 * 
        sum(x * x for x in b) ** 0.5
    )

@app.route("/api/v1/vector/encode", methods=["POST"])
def encode():
    """编码文本为向量"""
    data = request.get_json()
    text = data.get("text", "")
    
    r = requests.post(
        f"{OLLAMA}/api/embeddings",
        json={"model": MODEL, "prompt": text}
    )
    emb = r.json()["embedding"]
    
    with store_lock:
        vid = len(vector_store)
        vector_store[vid] = {"text": text, "embedding": emb[:768]}
    
    return jsonify({"id": vid, "dimension": 768, "status": "ok"})

@app.route("/api/v1/vector/search", methods=["POST"])
def search():
    """向量相似度搜索"""
    data = request.get_json()
    query = data.get("query", "")
    k = data.get("k", 5)
    
    query_emb = requests.post(
        f"{OLLAMA}/api/embeddings",
        json={"model": MODEL, "prompt": query}
    ).json()["embedding"]
    
    with store_lock:
        results = [
            (vid, cosine_sim(query_emb, v["embedding"]), v["text"])
            for vid, v in vector_store.items()
        ]
        results.sort(key=lambda x: -x[1])
    
    return jsonify({
        "query": query,
        "results": [
            {"rank": i+1, "text": t, "score": round(s, 4)}
            for i, (_, s, t) in enumerate(results[:k])
        ]
    })

@app.route("/api/v1/vector/batch", methods=["POST"])
def batch_encode():
    """批量编码"""
    data = request.get_json()
    texts = data.get("texts", [])
    
    results = []
    for text in texts:
        r = requests.post(
            f"{OLLAMA}/api/embeddings",
            json={"model": MODEL, "prompt": text}
        )
        emb = r.json()["embedding"]
        with store_lock:
            vid = len(vector_store)
            vector_store[vid] = {"text": text, "embedding": emb[:768]}
        results.append({"id": vid, "text": text[:50]})
    
    return jsonify({"count": len(results), "status": "ok"})

@app.route("/apex/formula", methods=["POST"])
def apex_formula():
    """APEX公式: EV = BV + sum(Gene_i x PHII_i)"""
    data = request.get_json()
    bv = data.get("bv", 0)
    genes = data.get("genes", [])
    
    total_dg = sum(g.get("delta_g", 0) for g in genes)
    ev = bv + total_dg
    
    return jsonify({
        "bv": bv,
        "gene_count": len(genes),
        "total_dg": total_dg,
        "ev": ev,
        "formula": "EV = BV + sum(Gene_i x Phi_i)"
    })

@app.route("/health", methods=["GET"])
def health():
    return jsonify({
        "status": "running",
        "model": MODEL,
        "size_mb": 274,
        "dimensions": 768,
        "ollama": "http://localhost:11434",
        "store_size": len(vector_store),
        "service": "APEX Vector LLM"
    })

@app.route("/", methods=["GET"])
def root():
    return jsonify({
        "service": "APEX Vector LLM",
        "model": MODEL,
        "ollama": "http://localhost:11434",
        "endpoints": [
            "POST /api/v1/vector/encode",
            "POST /api/v1/vector/search",
            "POST /api/v1/vector/batch",
            "POST /apex/formula",
            "GET /health"
        ]
    })

if __name__ == "__main__":
    print("=== APEX本地向量LLM启动 ===")
    print(f"模型: nomic-embed-text (274MB, 768维)")
    print(f"服务地址: http://0.0.0.0:11000")
    print(f"Ollama: http://localhost:11434")
    print("向量服务已就绪!")
    app.run(host="0.0.0.0", port=11000, debug=False)
