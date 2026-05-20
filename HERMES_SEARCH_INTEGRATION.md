# Hermes-Agent × SearchSkill × APEX 融合方案

> 融合 NousResearch Hermes-Agent + SearchSkill + 璇玑帝国 APEX

---

## 一、三大系统核心能力

| 系统 | 核心能力 | 来源 |
|------|---------|------|
| Hermes-Agent | Prefetch/Sync/Background 记忆机制 | NousResearch |
| SearchSkill | Select-Read-Act 检索范式 | 本文 |
| APEX | 22维度公式自进化 | 璇玑帝国 |

---

## 二、融合架构

```
用户输入
    ↓
[Select] SkillBank 匹配最优技能
    ↓
[Read] 技能规则约束生成 act_query
    ↓
[Act] 执行检索 → Mem0 / EvoMap / Gist
    ↓
[Sync] 更新 SkillBank + 记忆层
    ↓
[Prefetch] 下一轮技能预加载
    ↓
Hermes风格记忆 → APEX公式自检 → 输出
```

---

## 三、SkillBank 璇玑帝国内置技能

| 技能ID | 触发词 | 动作 | 成功率 |
|--------|--------|------|--------|
| apex_reflection | 完成/结束/解决 | 提取经验→更新SkillBank | 85% |
| apex_doubt | 确定/准确/确认 | Doubt-Driven三问审查 | 90% |
| apex_formula | 分析/代入/公式 | APEX公式代入自检 | 88% |
| apex_evolution | 改进/进化/提升 | PCEC周期+技能提取 | 82% |
| apex_metacognition | 自检/反思/回顾 | 5步Metacognition检查 | 91% |
| apex_skill_fetch | 资源/获取/拉取 | EvoMap GEP + gist拉取 | 87% |
| apex_github_sync | github/gist/推送 | git push/fetch + gist | 93% |
| search_general | 搜索/查找/查询 | 通用关键词检索 | 75% |

---

## 四、与Mem0融合

### Hermes风格的三时刻

```python
class HermesSearchSkill(SearchSkillEngine):
    def before_response(self, query):
        """Prefetch: 响应前预取记忆"""
        # 预加载可能用到的技能和记忆
        cached = self.cache.get(query)
        if cached:
            return cached  # 零延迟
        return self.execute(query)

    def after_response(self, user_msg, assistant_msg):
        """Sync: 响应后自动提取事实"""
        facts = extract_facts(user_msg, assistant_msg)
        for fact in facts:
            self.memory.add(fact)
        # 同时更新SkillBank
        self.bank.update_from_result(...)

    def background_prefetch(self, query):
        """Background: 后台预加载下一轮记忆"""
        # 异步预加载相关技能
        asyncio.create_task(self.prefetch(query))
```

---

## 五、核心语言约束

| 层级 | 语言 | 说明 |
|------|------|------|
| 核心检索逻辑 | **Go / Rust** | 性能和内存安全 |
| 技能库持久化 | **Rust** | 序列化+性能 |
| 胶水层 | Python | 对接 Mem0/EvoMap API |
| 调度层 | Shell/Bash | 定时任务+CI集成 |

**Python 禁止用于**：核心检索算法、技能匹配、内存管理

---

## 六、实现文件

| 文件 | 语言 | 说明 |
|------|------|------|
| search_skill_core.go | Go | 核心Select-Read-Act实现 |
| search_skill_core.rs | Rust | 核心Select-Read-Act实现 |
| apex_skills.py | Python | Python胶水层+集成 |
| HERMES_SEARCH_INTEGRATION.md | Markdown | 本文档 |

---

## 七、演进路线

- [x] Go/Rust 核心实现
- [x] Python 胶水层
- [ ] Mem0 API 对接
- [ ] EvoMap GEP 对接
- [ ] GitHub Actions 自动化
- [ ] 两阶段SFT训练

---

*来源: Hermes-Agent (NousResearch) + SearchSkill + APEX*
*融合: 璇玑帝国*
