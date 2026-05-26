#!/usr/bin/env python3
"""APEX向量记忆系统 - 向量存储+基因检索+决策优化"""
import requests
import json
import os
import sys
from datetime import datetime

OLLAMA_ENDPOINT = "http://localhost:11434/api/embeddings"
MODEL = "nomic-embed-text"
DIMENSION = 768


def cosine(a, b):
    n1 = sum(x * x for x in a)
    n2 = sum(x * x for x in b)
    if n1 == 0 or n2 == 0:
        return 0
    return sum(x * y for x, y in zip(a, b)) / (n1 ** 0.5 * n2 ** 0.5)


def encode(text):
    try:
        r = requests.post(OLLAMA_ENDPOINT,
                            json={"model": MODEL, "prompt": text},
                            timeout=30)
        return r.json()["embedding"][:DIMENSION]
    except Exception:
        return [0.0] * DIMENSION


class VectorMemory:
    """向量记忆系统"""

    def __init__(self, store_path="vector_memory.json"):
        self.store_path = store_path
        self.data = self._load()

    def add(self, text, meta=None):
        emb = encode(text)
        rec = {"id": len(self.data), "text": text, "embedding": emb,
               "meta": meta or {}, "time": datetime.now().isoformat()}
        self.data.append(rec)
        self._save()
        return rec

    def search(self, query, k=5):
        q_emb = encode(query)
        results = [(i, cosine(q_emb, r["embedding"]), r) for i, r in enumerate(self.data)]
        results.sort(key=lambda x: -x[1])
        return [{"rank": j+1, "score": round(s, 4), "text": r["text"]}
                for j, (i, s, r) in enumerate(results[:k])]

    def count(self):
        return len(self.data)

    def _load(self):
        if os.path.exists(self.store_path):
            try:
                with open(self.store_path, "r") as f:
                    return json.load(f)
            except Exception:
                return []
        return []

    def _save(self):
        with open(self.store_path + ".tmp", "w") as f:
            json.dump(self.data, f, ensure_ascii=False)
        os.replace(self.store_path + ".tmp", self.store_path)


class GeneRetriever:
    """向量基因检索系统"""

    def __init__(self, store_path="gene_pool.json"):
        self.store_path = store_path
        self.genes = self._load()

    def add(self, name, desc, dg=0):
        emb = encode(desc)
        g = {"name": name, "desc": desc, "emb": emb, "delta_g": dg}
        self.genes.append(g)
        self._save()
        return g

    def find_best(self, query, k=3):
        q_emb = encode(query)
        scored = [(g, cosine(q_emb, g["emb"])) for g in self.genes if "emb" in g]
        scored.sort(key=lambda x: -x[1])
        return [{"rank": j+1, "similarity": round(s, 4), "gene": g}
                for j, (g, s) in enumerate(scored[:k])]

    def count(self):
        return len(self.genes)

    def _load(self):
        if os.path.exists(self.store_path):
            try:
                with open(self.store_path, "r") as f:
                    return json.load(f)
            except Exception:
                return []
        return []

    def _save(self):
        with open(self.store_path + ".tmp", "w") as f:
            json.dump(self.genes, f, ensure_ascii=False, indent=2)
        os.replace(self.store_path + ".tmp", self.store_path)


class VectorDecision:
    """向量决策优化"""

    def __init__(self, store_path="decision.json"):
        self.store_path = store_path
        self.options = self._load()

    def add(self, name, desc):
        emb = encode(desc)
        self.options.append({"name": name, "desc": desc, "emb": emb})
        self._save()

    def optimize(self, goal, k=3):
        g_emb = encode(goal)
        scored = [(o, cosine(g_emb, o["emb"])) for o in self.options if "emb" in o]
        scored.sort(key=lambda x: -x[1])
        return [{"rank": j+1, "alignment": round(s, 4), "option": o}
                for j, (o, s) in enumerate(scored[:k])]

    def _load(self):
        if os.path.exists(self.store_path):
            try:
                with open(self.store_path, "r") as f:
                    return json.load(f)
            except Exception:
                return []
        return []

    def _save(self):
        with open(self.store_path + ".tmp", "w") as f:
            json.dump(self.options, f, ensure_ascii=False, indent=2)
        os.replace(self.store_path + ".tmp", self.store_path)


def test_memory():
    mem = VectorMemory()
    mem.add("EV = BV + sum(Gene_i * Phi_i)")
    mem.add("APEX十二因子: Agent = DeltaG * Product(Fi)")
    mem.add("吞噬Round 8: 新增36模块, DG=138.5")
    mem.add("量化回测系统: A股+期货历史数据")

    print(f"记忆库: {mem.count()}条\n")
    for r in mem.search("AI进化方案"):
        print(f"  R{r['rank']}: {r['text']} ({r['score']})")


def test_genes():
    gr = GeneRetriever()
    if len(gr.genes) == 0:
        gr.add("QuantumAgent", "量子计算智能体架构", 9.5)
        gr.add("EvolutionaryLLM", "语言模型进化引擎", 9.0)
        gr.add("SelfEvolve", "自我进化智能系统", 10.0)
        gr.add("DeepGenome", "深度学习基因序列", 8.5)

    print(f"\n基因池: {gr.count()}个\n")
    for b in gr.find_best("高性能AI"):
        print(f"  #{b['rank']}: {b['gene']['name']} ({b['similarity']})")


def test_decision():
    vd = VectorDecision()
    if len(vd.options) == 0:
        vd.add("本地化", "使用本地向量模型优化推理效率")
        vd.add("混合化", "结合本地与远程API实现灵活决策")
        vd.add("云端化", "依赖云端API实现所有计算")

    print(f"\n决策选项: {len(vd.options)}个\n")
    for r in vd.optimize("最大化效率"):
        print(f"  #{r['rank']}: {r['option']['name']} ({r['alignment']})")


if __name__ == "__main__":
    print("=== APEX向量记忆系统 ===")
    print(f"模型: nomic-embed-text (274MB, {DIMENSION}维)")
    print(f"Ollama: localhost:11434\n")
    test_memory()
    test_genes()
    test_decision()
    bv = 68.20
    dg = sum([9.0, 9.5, 10.0, 8.5, 8.0, 7.5])
    ev = bv + dg
    print(f"\nAPEX公式: BV({bv}) + DG({dg}) = EV({ev})")
    print("\n=== 就绪 ===")
