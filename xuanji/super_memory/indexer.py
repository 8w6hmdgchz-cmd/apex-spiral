#!/usr/bin/env python3
"""
CLAW Indexer - 原生Markdown格式索引器 V2.0
==========================================
安全 · 精准 · 全格式支持

CLAW格式支持:
- 标题 (# ## ###)
- 代码块 (```lang)
- 列表 (- *)
- 引用 (>)
- 表格 (| |)
- 元数据 (key: value)
- 标签 (#tag)
- 链接 [text](url)
- 强调 (**bold** *italic*)
- 数学公式 ($...$ 或 $$...$$)

安全增强:
- XSS prevention
- Sanitized output
- No script injection

Author: 璇玑 Xuanji-58
"""

import re
import json
from typing import List, Dict, Optional, Tuple, Any, Set
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
import hashlib
import html
import threading

# 线程安全锁
_lock = threading.Lock()


@dataclass
class CLAWNode:
    """CLAW Markdown结构节点"""
    id: str
    node_type: str  # heading | code | list | quote | table | meta | tag | link | text | math
    level: int = 0  # 标题层级 (1-6)
    content: str = ""
    raw: str = ""
    children: List['CLAWNode'] = field(default_factory=list)
    parent_id: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    language: Optional[str] = None  # 代码语言


@dataclass
class CLAWDocument:
    """CLAW文档结构"""
    title: str = ""
    headings: List[Tuple[int, str]] = field(default_factory=list)  # [(level, text), ...]
    code_blocks: List[Dict] = field(default_factory=list)  # [{lang, code}, ...]
    lists: List[Dict] = field(default_factory=list)  # [{level, items}, ...]
    tables: List[List[List[str]]] = field(default_factory=list)  # [[[header], [row1], [row2], ...]]
    quotes: List[str] = field(default_factory=list)
    tags: Set[str] = field(default_factory=set)
    metadata: Dict[str, str] = field(default_factory=dict)
    links: List[Dict] = field(default_factory=list)  # [{text, url}, ...]
    math_blocks: List[str] = field(default_factory=list)
    summary: str = ""
    word_count: int = 0


class SecurityValidator:
    """CLAW安全验证器"""
    
    DANGEROUS_TAGS = {'script', 'style', 'iframe', 'object', 'embed', 'form', 'input'}
    DANGEROUS_ATTRS = {'onerror', 'onload', 'onclick', 'onmouseover', 'onfocus', 'onblur'}
    
    @staticmethod
    def sanitize_html(text: str) -> str:
        """
        清理HTML防止XSS
        
        正确顺序:
        1. 先移除危险标签 (转义前匹配原始<tag>)
        2. 再转义剩余内容 (处理普通文本中的< > &等)
        
        注意: 如果输出用作HTML渲染，需要bleach库深度净化。
        此函数主要用于纯文本转义场景。
        """
        if not text:
            return ""
        
        # 1. 先移除危险的标签 (转义前，匹配原始HTML)
        for tag in SecurityValidator.DANGEROUS_TAGS:
            text = re.sub(f'<{tag}[^>]*>.*?</{tag}>', '', text, flags=re.IGNORECASE | re.DOTALL)
            text = re.sub(f'<{tag}[^>]*/?>', '', text, flags=re.IGNORECASE)
        
        # 2. 移除危险的属性 (在转义前)
        for attr in SecurityValidator.DANGEROUS_ATTRS:
            text = re.sub(f'\\s*{attr}\\s*=', ' ', text, flags=re.IGNORECASE)
        
        # 3. 最后转义HTML实体
        text = html.escape(text, quote=True)
        
        return text
    
    @staticmethod
    def validate_url(url: str) -> bool:
        """验证URL安全性"""
        if not url:
            return False
        
        # 只允许http/https/mailto
        allowed_schemes = {'http', 'https', 'mailto'}
        
        try:
            # 简单的scheme检查
            if '://' in url:
                scheme = url.split('://')[0].lower()
                return scheme in allowed_schemes
            return True
        except:
            return False
    
    @staticmethod
    def sanitize_markdown(text: str) -> str:
        """清理Markdown内容"""
        if not text:
            return ""
        
        # 移除<script>标签
        text = re.sub(r'<script[^>]*>.*?</script>', '', text, flags=re.IGNORECASE | re.DOTALL)
        text = re.sub(r'<script[^>]*/?>', '', text, flags=re.IGNORECASE)
        
        # 清理on*属性
        text = re.sub(r'\bon\w+\s*=', ' data-orig-', text, flags=re.IGNORECASE)
        
        return text


class CLAWParser:
    """
    CLAW Markdown解析器 V2.0
    ======================
    
    安全解析CLAW格式并建立结构化索引
    支持完整CLAW规范
    """
    
    def __init__(self):
        self.nodes: List[CLAWNode] = []
        self.root_id = "root"
        self._lock = threading.Lock()
    
    def parse(self, content: str) -> List[CLAWNode]:
        """解析CLAW Markdown内容"""
        with self._lock:
            # 安全清理
            content = SecurityValidator.sanitize_markdown(content)
            
            lines = content.split('\n')
            self.nodes = []
            
            current_heading = None
            in_code_block = False
            code_block_content = []
            code_block_lang = ""
            
            for i, line in enumerate(lines):
                # 代码块处理
                if line.strip().startswith('```'):
                    if not in_code_block:
                        # 开始代码块
                        in_code_block = True
                        code_block_lang = line.strip()[3:].strip()
                        code_block_content = []
                    else:
                        # 结束代码块
                        in_code_block = False
                        node_id = hashlib.md5(f"code_{i}".encode()).hexdigest()[:12]
                        self.nodes.append(CLAWNode(
                            id=node_id,
                            node_type="code",
                            content='\n'.join(code_block_content),
                            raw=f"```{code_block_lang}\n...\n```",
                            language=code_block_lang
                        ))
                    continue
                
                if in_code_block:
                    code_block_content.append(line)
                    continue
                
                node = self._parse_line(line, i)
                if node:
                    self.nodes.append(node)
                    
                    if node.node_type == "heading":
                        current_heading = node.content
    
    def _parse_line(self, line: str, line_num: int) -> Optional[CLAWNode]:
        """解析单行"""
        node_id = hashlib.md5(f"{line}_{line_num}".encode()).hexdigest()[:12]
        
        # 空行跳过
        if not line.strip():
            return None
        
        # 标题
        heading_match = re.match(r'^(#{1,6})\s+(.+)$', line)
        if heading_match:
            return CLAWNode(
                id=node_id,
                node_type="heading",
                level=len(heading_match.group(1)),
                content=heading_match.group(2).strip(),
                raw=line
            )
        
        # 代码块开始
        if re.match(r'^```', line):
            lang = line.strip()[3:].strip()
            return CLAWNode(
                id=node_id,
                node_type="code",
                content="",
                raw=line,
                language=lang if lang else None
            )
        
        # 列表
        list_match = re.match(r'^(\s*)[-*+]\s+(.+)$', line)
        if list_match:
            level = len(list_match.group(1)) // 2
            return CLAWNode(
                id=node_id,
                node_type="list",
                level=level,
                content=list_match.group(2).strip(),
                raw=line
            )
        
        # 数字列表
        numbered_match = re.match(r'^(\s*)\d+\.\s+(.+)$', line)
        if numbered_match:
            level = len(numbered_match.group(1)) // 2
            return CLAWNode(
                id=node_id,
                node_type="ordered_list",
                level=level,
                content=numbered_match.group(2).strip(),
                raw=line
            )
        
        # 引用
        if line.startswith('>'):
            return CLAWNode(
                id=node_id,
                node_type="quote",
                content=line[1:].strip(),
                raw=line
            )
        
        # 表格
        if re.match(r'^\|.*\|$', line) and line.count('|') >= 2:
            # 判断是否是分隔行
            if re.match(r'^\|[\s\-:|]+\|$', line):
                return CLAWNode(
                    id=node_id,
                    node_type="table_separator",
                    content=line,
                    raw=line
                )
            return CLAWNode(
                id=node_id,
                node_type="table_row",
                content=line,
                raw=line
            )
        
        # 元数据 (key: value)
        meta_match = re.match(r'^(\w+(?:[\w-]*\w)?)\s*:\s*(.+)$', line)
        if meta_match:
            return CLAWNode(
                id=node_id,
                node_type="meta",
                content=meta_match.group(2).strip(),
                raw=line,
                metadata={'key': meta_match.group(1)}
            )
        
        # 标签 #tag
        tag_matches = re.findall(r'#(\w+)', line)
        if tag_matches:
            return CLAWNode(
                id=node_id,
                node_type="tag",
                content=','.join(tag_matches),
                raw=line
            )
        
        # 链接 [text](url)
        link_matches = re.findall(r'\[([^\]]+)\]\(([^\)]+)\)', line)
        if link_matches:
            for text, url in link_matches:
                if not SecurityValidator.validate_url(url):
                    continue
            return CLAWNode(
                id=node_id,
                node_type="link",
                content=json.dumps(link_matches),
                raw=line
            )
        
        # 数学公式 $$...$$
        math_block_match = re.findall(r'\$\$(.+?)\$\$', line, re.DOTALL)
        if math_block_match:
            return CLAWNode(
                id=node_id,
                node_type="math_block",
                content='\n'.join(math_block_match),
                raw=line
            )
        
        # 数学公式 $...$
        math_inline_match = re.findall(r'\$([^\$]+)\$', line)
        if math_inline_match:
            return CLAWNode(
                id=node_id,
                node_type="math_inline",
                content=' '.join(math_inline_match),
                raw=line
            )
        
        # 水平线
        if re.match(r'^[-*_]{3,}$', line.strip()):
            return CLAWNode(
                id=node_id,
                node_type="hr",
                content="",
                raw=line
            )
        
        # 普通文本
        return CLAWNode(
            id=node_id,
            node_type="text",
            content=line.strip(),
            raw=line
        )
    
    def extract_entities(self) -> List[Dict]:
        """提取实体"""
        entities = []
        
        for node in self.nodes:
            if node.node_type == "meta":
                entities.append({
                    'type': 'metadata',
                    'key': node.metadata.get('key'),
                    'value': node.content,
                })
            elif node.node_type == "tag":
                for tag in node.content.split(','):
                    entities.append({
                        'type': 'tag',
                        'name': tag.strip(),
                    })
        
        return entities
    
    def extract_summary(self, max_length: int = 200) -> str:
        """提取摘要"""
        parts = []
        
        for node in self.nodes:
            if node.node_type == "heading" and node.level <= 2:
                parts.append(f"## {node.content}")
            elif node.node_type == "text" and parts:
                text = node.content[:max_length]
                if len(node.content) > max_length:
                    text += "..."
                parts.append(text)
                break
        
        return "\n".join(parts[:5])


class CLAWDocumentBuilder:
    """
    CLAW文档构建器
    ==============
    
    从解析结果构建结构化文档
    """
    
    def __init__(self):
        self.parser = CLAWParser()
    
    def build(self, content: str) -> CLAWDocument:
        """构建CLAW文档"""
        self.parser.parse(content)
        
        doc = CLAWDocument()
        
        # 统计字数
        doc.word_count = len(content.split())
        
        current_list = None
        list_level = 0
        
        for node in self.parser.nodes:
            if node.node_type == "heading":
                doc.headings.append((node.level, node.content))
                if node.level == 1 and not doc.title:
                    doc.title = node.content
            
            elif node.node_type == "code":
                doc.code_blocks.append({
                    'language': node.language,
                    'code': node.content
                })
            
            elif node.node_type == "list":
                # 处理列表
                if current_list is None or node.level < list_level:
                    if current_list:
                        doc.lists.append(current_list)
                    current_list = {'level': node.level, 'items': [node.content]}
                    list_level = node.level
                else:
                    current_list['items'].append(node.content)
            
            elif node.node_type == "ordered_list":
                if current_list is None or node.level < list_level:
                    if current_list:
                        doc.lists.append(current_list)
                    current_list = {'level': node.level, 'items': [node.content], 'ordered': True}
                    list_level = node.level
                else:
                    current_list['items'].append(node.content)
            
            elif node.node_type == "table_row":
                # 解析表格行
                cells = [c.strip() for c in node.content.split('|')[1:-1]]
                doc.tables.append(cells)
            
            elif node.node_type == "quote":
                doc.quotes.append(node.content)
            
            elif node.node_type == "tag":
                for tag in node.content.split(','):
                    doc.tags.add(tag.strip())
            
            elif node.node_type == "meta":
                doc.metadata[node.metadata.get('key', '')] = node.content
            
            elif node.node_type == "link":
                try:
                    links = json.loads(node.content)
                    for text, url in links:
                        if SecurityValidator.validate_url(url):
                            doc.links.append({'text': text, 'url': url})
                except:
                    pass
            
            elif node.node_type == "math_block":
                doc.math_blocks.append(node.content)
        
        # 添加最后一个列表
        if current_list:
            doc.lists.append(current_list)
        
        # 生成摘要
        doc.summary = self.parser.extract_summary()
        
        return doc


class CLAWIndexer:
    """
    CLAW索引器 V2.0
    ==============
    
    为CLAW Markdown格式内容建立结构化索引
    
    安全增强:
    - XSS prevention
    - URL validation
    - HTML sanitization
    - No script injection
    
    对比mem0:
    - mem0: 纯文本存储，无结构
    - SuperMemory: CLAW原生格式，结构化索引，完整安全验证
    """
    
    def __init__(self):
        self.builder = CLAWDocumentBuilder()
        self._lock = threading.Lock()
    
    def index(self, content: str, memory_id: str) -> Dict[str, Any]:
        """
        为内容建立索引
        
        Args:
            content: CLAW Markdown内容
            memory_id: 记忆ID
        
        Returns:
            {
                'memory_id': str,
                'title': str,
                'headings': [(level, title), ...],
                'code_blocks': [{lang, code}, ...],
                'lists': [{level, items}, ...],
                'tables': [[[header], [row1], ...], ...],
                'quotes': [quote, ...],
                'tags': [tag, ...],
                'metadata': {key: value},
                'links': [{text, url}, ...],
                'math_blocks': [math, ...],
                'summary': str,
                'entities': [entity, ...],
                'word_count': int,
            }
        """
        with self._lock:
            doc = self.builder.build(content)
            
            return {
                'memory_id': memory_id,
                'title': doc.title,
                'headings': doc.headings,
                'code_blocks': doc.code_blocks,
                'lists': doc.lists,
                'tables': doc.tables,
                'quotes': doc.quotes,
                'tags': list(doc.tags),
                'metadata': doc.metadata,
                'links': doc.links,
                'math_blocks': doc.math_blocks,
                'summary': doc.summary,
                'entities': self.builder.parser.extract_entities(),
                'word_count': doc.word_count,
                'node_count': len(self.builder.parser.nodes),
            }
    
    def search_by_tag(self, tag: str, content: str) -> bool:
        """按标签搜索"""
        return f"#{tag}" in content or f" #{tag} " in content
    
    def extract_code_languages(self, content: str) -> List[str]:
        """提取代码语言"""
        doc = self.builder.build(content)
        languages = []
        
        for block in doc.code_blocks:
            lang = block.get('language', '')
            if lang:
                languages.append(lang)
        
        return list(set(languages))
    
    def extract_headings(self, content: str) -> List[Tuple[int, str]]:
        """提取标题层级"""
        doc = self.builder.build(content)
        return doc.headings
    
    def to_safe_html(self, content: str) -> str:
        """转换为安全的HTML（用于预览）"""
        content = SecurityValidator.sanitize_markdown(content)
        content = SecurityValidator.sanitize_html(content)
        return content


class CLAWBuilder:
    """
    CLAW Markdown构建器
    ==================
    
    帮助构建符合CLAW规范的Markdown文档
    """
    
    def __init__(self):
        self.lines: List[str] = []
        self._in_code_block = False
        self._in_math_block = False
    
    def heading(self, text: str, level: int = 1) -> 'CLAWBuilder':
        """添加标题"""
        if 1 <= level <= 6:
            self.lines.append(f"{'#' * level} {text}")
        return self
    
    def text(self, text: str) -> 'CLAWBuilder':
        """添加文本"""
        self.lines.append(text)
        return self
    
    def paragraph(self, text: str) -> 'CLAWBuilder':
        """添加段落"""
        self.lines.append("")
        self.lines.append(text)
        self.lines.append("")
        return self
    
    def code(self, code: str, language: str = "") -> 'CLAWBuilder':
        """添加代码块"""
        if language:
            self.lines.append(f"```{language}")
        else:
            self.lines.append("```")
        self.lines.append(code)
        self.lines.append("```")
        return self
    
    def list_item(self, item: str, ordered: bool = False, number: int = None) -> 'CLAWBuilder':
        """添加列表项"""
        if ordered and number is not None:
            self.lines.append(f"    {number}. {item}")
        elif ordered:
            self.lines.append(f"- {item}")
        else:
            self.lines.append(f"- {item}")
        return self
    
    def quote(self, text: str) -> 'CLAWBuilder':
        """添加引用"""
        for line in text.split('\n'):
            self.lines.append(f"> {line}")
        return self
    
    def meta(self, key: str, value: str) -> 'CLAWBuilder':
        """添加元数据"""
        self.lines.append(f"{key}: {value}")
        return self
    
    def tag(self, tag: str) -> 'CLAWBuilder':
        """添加标签"""
        if not tag.startswith('#'):
            tag = f"#{tag}"
        self.lines.append(tag)
        return self
    
    def link(self, text: str, url: str) -> 'CLAWBuilder':
        """添加链接"""
        if SecurityValidator.validate_url(url):
            self.lines.append(f"[{text}]({url})")
        return self
    
    def table(self, headers: List[str], rows: List[List[str]]) -> 'CLAWBuilder':
        """添加表格"""
        # 表头
        self.lines.append("| " + " | ".join(headers) + " |")
        # 分隔线
        self.lines.append("| " + " | ".join(['---'] * len(headers)) + " |")
        # 数据行
        for row in rows:
            self.lines.append("| " + " | ".join(row) + " |")
        return self
    
    def math(self, formula: str, block: bool = True) -> 'CLAWBuilder':
        """添加数学公式"""
        if block:
            self.lines.append(f"$${formula}$$")
        else:
            self.lines.append(f"${formula}$")
        return self
    
    def divider(self) -> 'CLAWBuilder':
        """添加分隔线"""
        self.lines.append("---")
        return self
    
    def bold(self, text: str) -> 'CLAWBuilder':
        """粗体"""
        self.lines.append(f"**{text}**")
        return self
    
    def italic(self, text: str) -> 'CLAWBuilder':
        """斜体"""
        self.lines.append(f"*{text}*")
        return self
    
    def build(self) -> str:
        """构建CLAW文档"""
        return "\n".join(self.lines)
    
    def reset(self) -> 'CLAWBuilder':
        """重置"""
        self.lines = []
        return self


class CLAWValidator:
    """CLAW格式验证器"""
    
    @staticmethod
    def validate(content: str) -> Tuple[bool, List[str]]:
        """
        验证CLAW格式
        
        Returns:
            (is_valid, errors)
        """
        errors = []
        
        # 检查标题层级
        headings = re.findall(r'^(#{1,6})\s+', content, re.MULTILINE)
        if len(headings) > 20:
            errors.append("Too many headings (max 20)")
        
        # 检查代码块闭合
        code_blocks = content.count('```')
        if code_blocks % 2 != 0:
            errors.append("Unclosed code block")
        
        # 检查表格格式
        table_lines = [l for l in content.split('\n') if re.match(r'^\|.*\|$', l)]
        if len(table_lines) > 1:
            first_cells = len(table_lines[0].split('|')) - 2
            for i, line in enumerate(table_lines[1:], 1):
                cells = len(line.split('|')) - 2
                if cells != first_cells:
                    errors.append(f"Table row {i+1} has inconsistent cell count")
        
        # 检查危险内容
        if '<script' in content.lower():
            errors.append("Script tag detected")
        
        if len(content) > 1_000_000:
            errors.append("Content too large (max 1MB)")
        
        return (len(errors) == 0, errors)