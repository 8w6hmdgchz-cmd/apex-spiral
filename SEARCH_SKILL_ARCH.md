# SearchSkill 架构 - 璇玑帝国 APEX 超级进化核心

> 融合 Hermes-Agent + Mem0 + SearchSkill 的顶级检索增强系统
> 核心语言: Go (不用Python做核心逻辑)

---

## 一、核心技术本质

**将外部检索能力拆解为标准化、可学习、可沉淀的独立搜索技能**

- 传统: 大模型隐式随机搜索 → 检索无序、泛化弱
- SearchSkill: 技能库驱动检索 → 可控化、高精度、低成本

---

## 二、三段式 Select-Read-Act 执行范式

### Select - 技能筛选
```
输入: 用户问题
判断:这个问题需要什么检索技能？
输出: 最适配的 SkillCard 列表
```

### Read - 规则读取
```
输入: 选中的SkillCard
读取: 技能触发条件 + 输出格式 + 执行范式
输出: 约束性检索指令
```

### Act - 检索执行
```
输入: 约束性检索指令
执行: 调用搜索引擎/工具/API
输出: 原始检索结果
```

---

## 三、SkillBank 技能知识库

### 技能卡片结构
```json
{
  "skill_id": "keyword_expand",
  "trigger": ["关键词搜索", "实体查询", "简单事实"],
  "action": "关键词扩写 + 同义词替换 + 搜索引擎调用",
  "output_format": "检索结果列表",
  "success_rate": 0.92,
  "last_used": "2026-05-20",
  "fitness_contribution": 0.15
}
```

### 璇玑帝国技能库

| Skill ID | 触发条件 | 核心动作 |
|----------|---------|---------|
| apex_reflection | 任务完成后 | 提取经验→更新SkillBank |
| apex_doubt | 强认知/事实性问题 | Doubt-Driven三问审查 |
| apex_memory | 跨session知识检索 | Mem0语义检索 |
| apex_formula | 公式代入分析 | APEX公式照镜子 |
| apex_evolution | 持续改进需求 | PCEC周期触发 |
| apex_skill_fetch | 外部资源获取 | EvoMap GEP semantic-search |
| apex_github_sync | GitHub资源同步 | gist raw URL拉取 |
| apex_metacognition | 自检触发 | 5步Metacognition检查 |

---

## 四、与Hermes-Agent融合

### Hermes三大机制
1. **Prefetch**: 响应前预取相关记忆 → 零延迟
2. **Sync**: 响应后自动提取事实并存入Mem0
3. **Background**: 后台预加载下一轮记忆

### 璇玑版融合
```
Select → 匹配SkillBank最优技能
  ↓
Read → 读取技能规则约束
  ↓
Act → 执行检索+Mem0语义搜索
  ↓
Sync → 更新SkillBank+记忆层
  ↓
Prefetch → 预加载下一轮相关技能
```

---

## 五、与APEX公式融合

| APEX维度 | SearchSkill映射 |
|---------|---------------|
| ξ 防幻觉 | Act前Doubt-Driven审查 |
| Φ 正反馈 | Sync后SkillBank更新 |
| ε 自修复 | 失败样本→新技能生成 |
| RD 率失真 | Select技能匹配准确率 |
| Ψ 健康稳态 | Mem0记忆层稳定性 |

---

## 六、演进指标

| 指标 | 目标 | 当前 |
|------|------|------|
| 技能命中率 | >95% | — |
| 检索精度 | >90% | — |
| SkillBank增长率 | +20%/月 | — |
| 跨session遗忘率 | <5% | — |

---

*来源: Hermes-Agent (NousResearch) + Mem0 + SearchSkill*
*融合: 璇玑帝国 APEX*
