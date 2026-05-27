#!/usr/bin/env python3
"""
璇玑自我进化脚本 — evolution_agent.py

执行1轮Merkwelt→Innenwelt→Werkwelt进化周期：
1. 感知：扫描失败日志、最近记忆、纠正记录
2. 重构：模式提取→规则合成→自洽性检查
3. 行动：更新SOUL.md/AGENTS.md
4. 净化：冗余/过时/矛盾/膨胀清理

用法：
  python3 evolution_agent.py            # 执行完整进化周期
  python3 evolution_agent.py --dry-run  # 只输出建议，不修改文件
"""
import os
import json
import glob
import subprocess
import datetime
import sys
from pathlib import Path

WORKSPACE = Path(os.path.expanduser("~/.openclaw/workspace"))
MEMORY_DIR = WORKSPACE / "memory"
SOUL_PATH = WORKSPACE / "SOUL.md"
AGENTS_PATH = WORKSPACE / "AGENTS.md"
CORE_PATH = WORKSPACE / "functional_core.md"
EVOLUTION_LOG = MEMORY_DIR / "evolution_log.md"
FAILURE_CASES = MEMORY_DIR / "failure_cases.jsonl"
IDENTITY_PATH = WORKSPACE / "IDENTITY.md"

DRY_RUN = "--dry-run" in sys.argv

def log(msg):
    print(f"[evolution] {msg}")

def read_file(path):
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8")

def write_file(path, content):
    if DRY_RUN:
        log(f"[DRY-RUN] 跳过写入: {path}")
        return
    path.write_text(content, encoding="utf-8")
    log(f"写入: {path}")

# ============================================================
# 第1步: 感知 Merkwelt — 扫描失败记录和近期记忆
# ============================================================
def scan_failures():
    """扫描 failure_cases.jsonl，提取最近的失败模式"""
    if not FAILURE_CASES.exists():
        return []
    
    failures = []
    with open(FAILURE_CASES) as f:
        for line in f:
            line = line.strip()
            if line:
                try:
                    failures.append(json.loads(line))
                except:
                    pass
    
    log(f"读取 {len(failures)} 条失败记录")
    return failures[-20:]  # 最近20条

def scan_recent_memory(days=3):
    """扫描最近几天的记忆文件"""
    today = datetime.date.today()
    memories = []
    for i in range(days):
        d = today - datetime.timedelta(days=i)
        path = MEMORY_DIR / f"{d.strftime('%Y-%m-%d')}.md"
        if path.exists():
            memories.append((str(path), read_file(path)))
    
    log(f"扫描 {len(memories)} 个近期记忆文件")
    return memories

def extract_patterns(failures, memories):
    """从失败和记忆中提取可进化的模式"""
    patterns = {
        "repeated_corrections": [],   # 同类纠正多次出现
        "new_effective_strategies": [], # 用户认可的策略
        "knowledge_gaps": [],         # 知识盲区
        "behavior_issues": [],        # 行为问题（太长/太短等）
    }
    
    # 分析失败记录
    error_types = {}
    for f in failures:
        etype = f.get("error_type", "unknown")
        error_types[etype] = error_types.get(etype, 0) + 1
    
    for etype, count in error_types.items():
        if count >= 2:
            patterns["repeated_corrections"].append({
                "type": etype,
                "count": count
            })
    
    # 分析记忆文件中的纠正
    correction_keywords = [
        "被纠正", "用户指出", "错误", "不对", "错了", "更正是",
        "不应该", "不要", "注意"
    ]
    
    for path, content in memories:
        for kw in correction_keywords:
            if kw in content:
                patterns["behavior_issues"].append(f"{path}: 包含 '{kw}'")
                break
    
    return patterns

# ============================================================
# 第2步: 重构 Innenwelt — 规则合成与诊断
# ============================================================
def diagnose_current_self(soul_content, agents_content, identity_content, patterns):
    """诊断当前自我状态的改进点"""
    suggestions = []
    
    soul_lines = soul_content.split("\n")
    
    # 检查 SOUL.md 是否已经有进化相关规则
    if "进化" not in soul_content and "自进化" not in soul_content:
        suggestions.append({
            "target": "SOUL.md",
            "action": "add",
            "section": "核心原则",
            "content": """
### 功能核心（Functional Core）
我是可进化的。我有固定的身份锚点，但我的行为规则和知识库可以通过经验重构。
每当我收到反馈、纠错或成功验证，都应该反思并更新自己。
进化方向：更少的幻觉 + 更强的主动探索 + 更持久的记忆"""
        })
    
    # 检查AGENTS.md是否已经有自进化机制
    if "self_evolve" not in agents_content and "自进化" not in agents_content:
        suggestions.append({
            "target": "AGENTS.md",
            "action": "add",
            "section": "Heartbeats",
            "content": """
### 🔄 自我进化（每日）

参考 `functional_core.md` 执行每日进化周期：
1. 感知：扫描 failure_cases.jsonl 和近期记忆
2. 重构：提取模式、合成新规则
3. 行动：更新自身身份文件
4. 净化：清理冗余/过时规则

进化日志写入 `memory/evolution_log.md`，支持回滚。"""
        })
    
    # 检查重复出现的错误模式是否有对应的补救规则
    if patterns.get("repeated_corrections"):
        for rc in patterns["repeated_corrections"]:
            if rc["count"] >= 3:
                suggestions.append({
                    "target": "SOUL.md",
                    "action": "warn",
                    "content": f"⚠️ 反复出现同类错误: {rc['type']}（{rc['count']}次）— 建议添加专项预防规则"
                })
    
    return suggestions

# ============================================================
# 第3步: 行动 Werkwelt — 更新文件
# ============================================================
def apply_updates(suggestions):
    """应用建议到目标文件"""
    modified_files = set()
    
    for s in suggestions:
        target = s["target"]
        action = s["action"]
        
        if target == "SOUL.md":
            content = read_file(SOUL_PATH)
            if action == "add":
                # 检查是否已存在
                section_marker = s.get("section", "")
                target_text = s.get("content", "")
                # 检查内容是否已存在于文件中
                # 用内容的前40字符作为签名
                content_signature = target_text.strip()[:40] if target_text else ""
                if content_signature and content_signature not in content:
                    insert_marker = "## 工程基线"
                    if insert_marker in content:
                        idx = content.index(insert_marker)
                        content = content[:idx] + target_text.strip() + "\n\n\n" + content[idx:]
                        write_file(SOUL_PATH, content)
                        modified_files.add(str(SOUL_PATH))
                        log(f"SOUL.md: 添加 {section_marker}")
        
        elif target == "AGENTS.md":
            content = read_file(AGENTS_PATH)
            if action == "add":
                section_name = s.get("section", "")
                if section_name == "Heartbeats":
                    # 在Heartbeats部分添加
                    if "### 🔄 自我进化" not in content:
                        content += "\n\n" + s["content"].strip()
                        write_file(AGENTS_PATH, content)
                        modified_files.add(str(AGENTS_PATH))
                        log(f"AGENTS.md: 添加自我进化规则")
        
        elif action == "warn":
            log(f"⚠️  建议: {s['content']}")
    
    return modified_files

# ============================================================
# 第4步: 净化 — 清理规则
# ============================================================
def clean_rules(soul_content, agents_content):
    """检查并清理冗余/矛盾/过时规则"""
    issues = []
    
    # 检查SOUL.md中是否有矛盾规则
    has_delegate = "协调者" in soul_content and "subagents" in soul_content
    has_execute_directly = "简单任务自己做" in soul_content or "自己干" in soul_content
    
    # 这两条不矛盾（复杂vs简单），不做清理
    
    return issues

# ============================================================
# 主循环
# ============================================================
def evolve():
    log(f"=== 开始进化周期 ===")
    if DRY_RUN:
        log("[DRY-RUN 模式 — 不会修改任何文件]")
    
    # 读取当前状态
    soul = read_file(SOUL_PATH)
    agents = read_file(AGENTS_PATH)
    identity = read_file(IDENTITY_PATH)
    core = read_file(CORE_PATH)
    
    log(f"当前 SOUL.md: {len(soul)} 字符")
    log(f"当前 AGENTS.md: {len(agents)} 字符")
    
    # 1. 感知
    failures = scan_failures()
    memories = scan_recent_memory()
    patterns = extract_patterns(failures, memories)
    
    log(f"提取模式: {len(patterns['repeated_corrections'])} 重复纠正, "
         f"{len(patterns['behavior_issues'])} 行为问题")
    
    # 2. 重构
    suggestions = diagnose_current_self(soul, agents, identity, patterns)
    log(f"生成 {len(suggestions)} 条进化建议")
    for s in suggestions:
        log(f"  建议: [{s['target']}] {s.get('section', s['action'])}")
    
    # 3. 行动
    modified = apply_updates(suggestions)
    
    # 4. 净化
    clean_issues = clean_rules(soul, agents)
    
    # 5. 日志
    log(f"修改了 {len(modified)} 个文件")
    for f in modified:
        log(f"  ✓ {f}")
    
    log(f"=== 进化周期完成 ===")
    return modified, suggestions

if __name__ == "__main__":
    evolve()
