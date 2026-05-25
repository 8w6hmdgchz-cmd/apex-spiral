---
name: apex-book-skill
description: 过目不忘 Book-to-Skill 系统。文档→SKILL编译→按需加载→记忆固化→Agent并行。
metadata: { "openclaw": { "emoji": "📚", "requires": { "bins": ["go"] } } }
---

# APEX Book-to-Skill 系统

## 核心公式

```
ApexBookSkill = DoclingParse ⊗ SkillStruct ⊗ LazyLoad ⊗ MemLLM ⊗ ParallelAgent
```

| 层 | 符号 | 职责 |
|----|------|------|
| T1 | DoclingParse | 解析 word/PDF/EPUB/markdown → 结构化文本 |
| T2 | SkillStruct | 编译为标准SKILL包（章节/术语/范式/速查） |
| T3 | LazyLoad | 按需加载索引，查询时才加载章节内容 |
| T4 | MemLLM | 长期独立记忆，跨session知识图谱 |
| T5 | ParallelAgent | 多Agent并行：检索/范式/代码/编译 |

## 工作流

```
文档(.docx/.pdf/.md/.epub)
    ↓
[T1] DoclingParse — 格式识别 + 内容抽取
    ├─ .docx → python-docx解析
    ├─ .md   → 直接读取
    ├─ .pdf  → 文本提取
    ├─ .txt  → 原始文本
    └─ .json → 结构化直接入
    ↓
[T2] SkillStruct — LLM编译为标准SKILL
    ├─ 章节结构 (chapters)
    ├─ 核心术语 (terms)
    ├─ 思维范式 (paradigms)
    ├─ 速查表 (cheatsheet)
    └─ 代码示例 (code_examples)
    ↓
[T3] LazyLoad — 按需索引
    ├─ 生成轻量索引 (章节名→偏移量/行号)
    ├─ 查询时只加载命中章节
    └─ 未命中章节保持未加载状态
    ↓
[T4] MemLLM — 记忆固化
    ├─ 写入 memory/wiki/
    ├─ 记忆带标签/来源/版本
    └─ RAG检索优先
    ↓
[T5] ParallelAgent — 多Agent并行调用
    ├─ Agent-检索: 查找相关知识
    ├─ Agent-范式: 提取思维模型
    ├─ Agent-代码: 提取代码示例
    └─ Agent-编译: 整合输出SKILL
```

## 用法

```bash
# 编译一本书/文档为SKILL
apex-book-skill compile --file 文档路径 --name skill名称

# 批量编译目录
apex-book-skill batch --dir 文档目录

# 查询已编译SKILL
apex-book-skill query --skill skill名称 --chapter 章节

# 列出所有已编译SKILL
apex-book-skill list

# 记忆检索
apex-book-skill search --query "关键词"
```

## 输出结构

```
skills/compiled/
├── <skill-name>/
│   ├── SKILL.md        → 主技能定义
│   ├── index.json      → 按需加载索引
│   ├── chapters/       → 章节内容 (lazy-loaded)
│   │   ├── ch01.md
│   │   └── ch02.md
│   └── memory/         → 记忆固化
│       ├── terms.json
│       ├── paradigms.json
│       └── cheatsheet.md
```
