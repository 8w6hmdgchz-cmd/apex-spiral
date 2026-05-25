# 🔬 科研技能库 — APEX StraTA 吸收蒸馏

> 来源: GitHub 科研仓库深度分析 → 提炼为可直接调用的科研工具链

---

## 1. 医学统计核心链 (statsmodels)

### 1.1 广义线性模型 — GLM (医学研究主工具)

**用途**: 连续/二分类/计数结局的回归分析

```
可用模型:
├── GLM(family=Binomial)    → 二分类(OR/RR)
├── GLM(family=Poisson)     → 计数(RR/IRR)
├── GLM(family=Gaussian)    → 连续(均数差)
├── GLM(family=Gamma)       → 偏态连续(费用/时长)
└── GEE()                   → 重复测量(GEE方程)

可用连接函数:
├── Logit → OR
├── Log   → RR/IRR  
├── Identity → 均数差
└── Probit → 概率单位
```

### 1.2 生存分析 — Survival

```
可用模型:
├── PHReg()       → Cox比例风险(HR)
├── SurvfuncRight → Kaplan-Meier曲线
├── survdiff()    → Log-rank检验
└── CumIncidenceRight → 累积发生率
```

### 1.3 Meta分析专用

```
├── genmod/families → 固定效应/随机效应
├── regression/OLS  → 加权回归(meta-regression)
└── stats/ → 异质性检验(I²/Q统计)
```

### 1.4 诊断与验证

```
├── stats/stattools → 假设检验
├── graphics/qqplot → 正态性诊断
├── graphics/ → 森林图/漏斗图
└── robust/ → 稳健标准误
```

---

## 2. 蛋白质 & 生物信息学 (ESM / AlphaFold)

### 2.1 ESM 蛋白质语言模型

```
├── esm.pretrained → 预训练模型加载
│   ├── esm2_t33_650M_UR50D()  → 650M参数
│   ├── esm2_t36_3B_UR50D()    → 3B参数
│   └── esmfold_v1()           → 结构预测
├── esm.data → 序列数据处理
├── esm.model → Transformer架构
└── esm.inverse_folding → 逆折叠设计
```

### 2.2 AlphaFold / RoseTTAFold

```
├── AlphaFold → 蛋白质结构预测(SOTA)
├── RoseTTAFold → 快速结构预测
└── 用途: 麻醉药物靶点结构分析
```

---

## 3. 生物统计专用工具

### 3.1 流行病学 (epinowcast)

```
├── nowcasting → 实时发病率估计
├── growth_rate → 增长率计算
└── 用途: 院内感染/ICU负荷预测
```

### 3.2 单细胞分析 (scanpy)

```
├── scanpy.tl → 聚类/差异分析
├── scanpy.pp → 预处理/标准化
└── 用途: 麻醉相关基因表达分析
```

---

## 4. 科研论文辅助

### 4.1 统计图表

```
├── matplotlib → 发表级图表
│   ├── 森林图(for meta-analysis)
│   ├── 漏斗图(发表偏倚)
│   ├── 生存曲线
│   └── 亚组森林图
└── statsmodels.graphics → 统计诊断图
```

### 4.2 数据管理

```
├── pandas → 数据清洗/合并
├── statsmodels.datasets → 示例数据
└── statsmodels.iolib → 数据导入导出
```

---

## 5. 与我已有系统的融合

### 5.1 科研工作流

```
用户提问 → StraTA分层
├── [T1] 策略: 分析科研问题类型(观察性/RCT/Meta)
├── [T2] Agent并行: 
│   ├── Agent-统计: 选模型(GLM/Cox/Meta)
│   ├── Agent-分析: 执行分析并诊断
│   └── Agent-验证: 敏感性/异质性/发表偏倚
├── [T3] GRPO: 多模型比较(AIC/BIC/似然比)
└── [T4] MemLLM: 存档分析策略 → 下次复用
```

### 5.2 快速参考

| 科研场景 | 推荐模型 | 关键参数 | 输出指标 |
|---------|---------|---------|---------|
| RCT疗效(二分类) | GLM+Bernoulli+Logit | family=Binomial() | OR, 95%CI, p |
| RCT疗效(连续) | OLS/GLM+Gaussian | family=Gaussian() | MD, 95%CI, p |
| 生存分析(OS/PFS) | CoxPH | PHReg() | HR, 95%CI, p |
| Meta二分类 | Mantel-Haenszel/statsmodel | GLM+随机效应 | OR/RR, I² |
| 诊断试验 | Logistic | GLM+Binomial | AUC, Sen, Spe |
| 重复测量 | GEE | cov_struct=Exchangeable | OR/MD, QIC |
| 倾向性评分 | Logistic→WLS | PS→IPTW/Matching | ATE, ATT |

### 5.3 Letta 记忆分层 → 科研助手记忆架构

**吸收结论**: Letta/MemGPT 的关键价值不是“把所有上下文塞进提示词”，而是把记忆分成不同寿命、不同检索成本、不同写入纪律的层。科研助手应该显式区分工作记忆、核心记忆、归档记忆和程序性技能，避免把一次性聊天内容污染长期技能库。

```
Letta式分层:
├── Core memory        → 长期稳定身份/偏好/研究方向
├── Recall memory      → 最近交互、短期任务上下文
├── Archival memory    → 可检索文献、实验记录、方法学笔记
└── Procedural memory  → 固化为可复用流程/SKILL/runbook
```

**映射到璇玑科研系统**:

| 记忆层 | 写入内容 | 读取时机 | 淘汰/晋升规则 |
|-------|---------|---------|---------------|
| 工作记忆 | 当前论文、数据表、问题约束 | 单轮任务内 | 任务结束后只保留结论 |
| 核心记忆 | 用户研究方向、统计偏好、禁忌 | 每次科研对话前 | 低频更新，需事实确认 |
| 归档记忆 | 文献摘要、代码片段、失败案例 | 检索命中时 | 可大量保存，但必须带来源 |
| 程序性技能 | 统计模型选择、质控清单、分析模板 | 执行同类任务前 | 多次验证后从归档晋升 |

**操作规则**:
- 新文献先入归档记忆，不直接改核心记忆。
- 至少 2 次任务复用成功后，才把流程提炼为 `skills/` 或 runbook。
- 每条长期记忆必须记录来源、适用范围、失效条件。
- 当上下文冲突时，优先级: 用户最新指令 > 核心记忆 > 程序性技能 > 归档检索片段 > 工作记忆猜测。
