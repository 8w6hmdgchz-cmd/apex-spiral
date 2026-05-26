#!/usr/bin/env python3
"""本地向量LLM服务 - 使用Ollama nomic-embed-text"""
import requests
from flask import Flask, request, jsonify

app = Flask(__name__)

OLLAMA_URL = "http://localhost:11434"
MODEL = "nomic-embed-text"

@app.route("/api/v1/embed", methods=["POST"])
def embed():
    data = request.get_json()
    prompt = data.get("prompt", "")
    if not prompt:
        return jsonify({"error": "No prompt"}), 400
    
    res = requests.post(f"{OLLAMA_URL}/api/embeddings",
            json={"model": MODEL, "prompt": prompt},
            timeout=30)
    result = res.json()
    return jsonify({
        "model": MODEL,
        "embedding": result.get("embedding", []),
        "status": "ok"
    })

@app.route("/api/v1/vector/similar", methods=["POST"])
def similar():
    data = request.get_json()
    target_prompt = data.get("target", "")
    corpus = data.get("corpus", [])
    top_k = data.get("top_k", 5)
    
    # Embed target
    target_res = requests.post(f"{OLLAMA_URL}/api/embeddings",
            json={"model": MODEL, "prompt": target_prompt},
            timeout=30)
    target_emb = target_res.json()["embedding"]
    
    # Embed corpus
    corpus_embs = []
    for text in corpus:
        r = requests.post(f"{OLLAMA_URL}/api/embeddings",
                json={"model": MODEL, "prompt": text}, timeout=30)
        corpus_embs.append(r.json()["embedding"])
    
    # Cosine similarity
    def dot(a, b):
        return sum(x*y for x, y in zip(a, b))
    def norm(a):
        return sum(x*x for x in a) ** 0.5
    
    sims = []
    for i, emb in enumerate(corpus_embs):
        sim = dot(target_emb, emb) / (norm(target_emb) * norm(emb))
        sims.append((sim, corpus[i], i))
    
    sims.sort(key=lambda x: -x[0])
    return jsonify({
        "target": target_prompt,
        "results": [{"rank": j+1, "text": t, "score": round(s, 4)} 
                     for s, t, j in sims[:top_k]]
    })

@app.route("/health", methods=["GET"])
def health():
    return jsonify({"status": "ok", "model": MODEL, 
                      "size": "274MB", "dimensions": 768})

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=9100, debug=False)
    print("本地向量服务: http://localhost:9100")
    print("API: POST /api/v1/embed")
    print("Similarity: POST /api/v1/vector/similar")
