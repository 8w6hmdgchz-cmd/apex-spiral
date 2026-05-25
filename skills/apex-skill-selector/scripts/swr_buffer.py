#!/usr/bin/env python3
"""SWRs RingBuffer — fitness >= threshold 时 O(1) 追加固化"""
import json, sys, time
from pathlib import Path

DEFAULT_BUFFER = {
    "buffer":    [],      # [{skill, fitness, timestamp}]
    "head":      0,       # 下一条写入位置 (O(1) 指针)
    "capacity":  128,
    "threshold": 0.7,
}

def load_buffer(path: str) -> dict:
    try:
        with open(path) as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        return DEFAULT_BUFFER.copy()

def save_buffer(path: str, buf: dict):
    with open(path, "w") as f:
        json.dump(buf, f, indent=2, ensure_ascii=False)

def ring_push(buf: dict, skill: str, fitness: float, timestamp: float = None) -> dict:
    """
    O(1) 追加到 RingBuffer:
    1. fitness >= threshold 才写入
    2. 自动覆盖最旧条目 (head 指针循环)
    """
    if fitness < buf["threshold"]:
        return {"archived": False, "reason": f"fitness {fitness:.3f} < {buf['threshold']}", "buffer_len": len(buf["buffer"])}

    ts = timestamp or time.time()
    entry = {"skill": skill, "fitness": fitness, "timestamp": ts}

    if len(buf["buffer"]) < buf["capacity"]:
        buf["buffer"].append(entry)
    else:
        # O(1) 覆盖: head 即是最旧条目位置
        buf["buffer"][buf["head"]] = entry

    buf["head"] = (buf["head"] + 1) % buf["capacity"]
    return {"archived": True, "entry": entry, "buffer_len": len(buf["buffer"])}

def ring_evict_low_fitness(buf: dict, min_fitness: float = 0.5) -> list:
    """剔除 fitness < min_fitness 的条目 (LRU 辅助)"""
    evicted = []
    new_buffer = []
    for entry in buf["buffer"]:
        if entry["fitness"] < min_fitness:
            evicted.append(entry)
        else:
            new_buffer.append(entry)
    buf["buffer"] = new_buffer
    buf["head"] = len(new_buffer) % buf["capacity"]
    return evicted

def ring_query(buf: dict, skill: str = None, top_k: int = 5) -> list:
    """查询 buffer: 特定 skill 或 top-k fitness"""
    if skill:
        return [e for e in buf["buffer"] if e["skill"] == skill]
    sorted_buf = sorted(buf["buffer"], key=lambda x: x["fitness"], reverse=True)
    return sorted_buf[:top_k]

def ring_stats(buf: dict) -> dict:
    return {
        "buffer_len":  len(buf["buffer"]),
        "capacity":    buf["capacity"],
        "head":        buf["head"],
        "threshold":   buf["threshold"],
        "avg_fitness": round(sum(e["fitness"] for e in buf["buffer"]) / max(len(buf["buffer"]), 1), 4),
        "max_fitness": round(max((e["fitness"] for e in buf["buffer"]), default=0.0), 4),
    }

# ---- CLI ----
if __name__ == "__main__":
    args = json.load(sys.stdin)
    cmd  = args.get("cmd", "push")
    path = args.get("buffer_file", "")

    buf = load_buffer(path) if path else DEFAULT_BUFFER.copy()

    if cmd == "push":
        result = ring_push(buf, args["skill"], args["fitness"], args.get("timestamp"))
        if path:
            save_buffer(path, buf)
        print(json.dumps(result, indent=2, ensure_ascii=False))

    elif cmd == "evict":
        evicted = ring_evict_low_fitness(buf, args.get("min_fitness", 0.5))
        if path:
            save_buffer(path, buf)
        print(json.dumps({"evicted": evicted, "buffer_len": len(buf["buffer"])}, indent=2, ensure_ascii=False))

    elif cmd == "query":
        res = ring_query(buf, args.get("skill"), args.get("top_k", 5))
        print(json.dumps(res, indent=2, ensure_ascii=False))

    elif cmd == "stats":
        print(json.dumps(ring_stats(buf), indent=2, ensure_ascii=False))

    else:
        print(json.dumps({"error": f"unknown cmd: {cmd}"}), file=sys.stderr)
        sys.exit(1)
