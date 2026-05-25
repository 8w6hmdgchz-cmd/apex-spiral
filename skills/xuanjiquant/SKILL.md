---
name: xuanjiquant
description: XuanjiQuant 本地自进化智能量化交易系统工作流。Use when user asks about XuanjiQuant, 璇玑量化, APEX 量化公式, 宏观金融周期研判, 本地量化数据库, 腾讯金融 API, 自动复盘/调参/进化, or wants to整理/落地量化策略体系. Focuses on formula-driven research workflow and architecture; pair with akquant-backtest or quant-backtest-strategy for executable backtests.
---

# XuanjiQuant Skill

XuanjiQuant 是一个“本地化 + APEX 公式体系 + 宏观金融周期 + 自动复盘进化”的量化研究框架。使用本 skill 时，优先把抽象公式落成可验证的研究流程；避免把未实测收益、未接入 API、未运行回测的内容说成事实。

## When to Use

Use this skill when the user asks to:

- 整理、解释、扩展或落地 **XuanjiQuant / 璇玑量化**。
- 用 APEX 公式做金融市场、行业、个股、资产配置分析。
- 构建本地量化数据库、腾讯金融 API 数据链路、盘前同步、收盘复盘。
- 设计“宏观周期 → 行业赛道 → 个股/ETF → 风控”的自上而下量化流程。
- 把公式体系转成可执行指标、评分卡、回测任务或日报模板。

Prefer other skills when:

- 用户明确要直接跑 A 股双均线 / RSI / AKShare 回测：use `akquant-backtest`.
- 用户明确要 Python Web 回测系统、PDF/Excel 报告、预设多因子回测：use `quant-backtest-strategy` if available.

## Source Material

Primary project directory:

```text
/Users/lihongxin/.openclaw/workspace/XuanjiQuant-main
```

Important files:

- `README.md` — 项目定位、APEX 主公式、核心子公式。
- `CONFIG.md` — 环境、运行周期、校验标准。
- `QUANT_FORMULAS.md` — APEX V10 总公式与子公式体系。
- `MACRO_FINANCE.md` — 宏观周期、资产趋势定价、金融演化公式。
- `BIO_FORMULA.md` — 生物内源调控分支；金融任务一般只做体系引用，不要强行用于投资结论。

Mirrored concise references are in `references/`.

## Core Positioning

XuanjiQuant 的定位：

- **本地化部署**：以腾讯金融 API/本地数据为基础构建私有量化数据库。
- **公式驱动研究**：用 APEX 公式体系组织宏观、行业、个股与风控变量。
- **自动闭环**：盘前数据同步、收盘复盘、参数调优、策略进化。
- **研究优先，实盘谨慎**：所有信号必须经过数据校验和回测，不能直接当作投资建议。

## Core Formulas

### APEX V10 总公式

```text
ΔG_total = (C_total · Λ_gene · Ω_entropy · Φ_all · Θ_bio · Φ_img · ΔG_finance) / (H_info · t)
```

For finance tasks, usually simplify to the finance branch:

```text
ΔG_final = (C_total · Λ_gene · Ω_entropy · Φ_all · ΔG_finance) / (H_info · t)
```

### 全能融合

```text
Φ_all = (K · H · P · ΔR · S_p) / (N · τ)
```

Practical mapping:

- `K`：金融知识/估值/技术指标/产业逻辑。
- `H`：历史行情与牛熊周期记忆。
- `P`：策略执行纪律、仓位计划、复盘习惯。
- `ΔR`：本轮复盘相对上一轮的收益/风险改善。
- `S_p`：信号稳定性或样本外稳定性。
- `N`：噪声、伪信号、过拟合。
- `τ`：迭代周期成本。

### 宏观周期

```text
Ψ_cycle = (D_debt · S_society · T_tech · E_economy) / (R_rate · I_inflation · F_confidence)
```

Interpretation:

- Higher `Ψ_cycle` means stronger broad market upside potential **only after data validation**.
- Inputs should be scored from observable data: credit, policy, industry tech cycle, GDP/PMI/export/consumption, rates, inflation, sentiment.

### 资产趋势定价

```text
Ω_asset = Ψ_cycle · α_track - β_risk
```

- `α_track`：个股/行业与当前主线行情的贴合度。
- `β_risk`：黑天鹅、政策利空、估值泡沫、流动性、回撤风险。

### 金融演化

```text
ΔG_finance = Ω_asset · (K_fin · H_hist / N_noise) · Φ_all
```

- `K_fin`：估值、财务、量价、筹码、行业知识。
- `H_hist`：历史同类行情、牛熊周期、因子表现回溯。
- `N_noise`：短期波动、诱多诱空、消息面噪声、数据缺失。

## Standard Workflow

### 1. Clarify Target

Identify:

- Market: A股 / ETF / 港股 / 美股 / crypto / macro assets.
- Objective: research, screening, backtest, risk review, daily report, automation.
- Horizon: intraday, swing, monthly allocation, long-term.
- Constraints: capital, max drawdown, turnover, data source, execution style.

### 2. Build Indicator Mapping

Convert formula symbols into measurable indicators.

Example scoring table:

| Module | Variable | Example measurable proxy |
|---|---|---|
| Macro | D_debt | credit impulse, M2, social financing |
| Macro | R_rate | 10Y bond yield, policy rate, SHIBOR |
| Macro | I_inflation | CPI/PPI trend |
| Track | α_track | industry momentum, relative strength, earnings trend |
| Risk | β_risk | volatility, drawdown, valuation percentile, policy risk |
| Noise | N_noise | signal disagreement, data gaps, news noise |

Use 0-1 or 0-100 scores. Always state assumptions.

### 3. Data Preparation

Preferred path:

1. Pull market/fundamental/macro data from available APIs or local files.
2. Store locally with date, source, symbol, adjusted prices, and update timestamp.
3. Validate missing values, duplicates, adjusted price consistency, and survivorship bias.

If Tencent Finance API is unavailable, say so and use available local/API alternatives only after noting the substitution.

### 4. Score and Screen

Compute:

1. `Ψ_cycle` for market regime.
2. `Ω_asset` for asset/industry attractiveness.
3. `ΔG_finance` for final ranking.
4. Risk flags: liquidity, drawdown, valuation, policy/news, data quality.

Output should separate:

- **Observation**: what data says.
- **Inference**: what the formula implies.
- **Action candidate**: what to test next.
- **Not investment advice**: if relevant.

### 5. Backtest / Validate

Never treat a formula ranking as proven. Validate with:

- Time split or rolling-window backtest.
- Benchmark comparison.
- Transaction costs and slippage.
- Max drawdown, Sharpe, Calmar, win rate, turnover.
- Parameter sensitivity.
- Out-of-sample or walk-forward check when possible.

For actual execution use `akquant-backtest` or project-specific scripts.

### 6. Review and Iterate

Daily/periodic loop:

- 盘前：同步数据，更新宏观/行业/个股评分。
- 盘中：只监控预设风险/信号，不频繁改策略。
- 收盘：复盘信号命中、收益归因、噪声来源、参数漂移。
- 周期：更新 `H_hist`，降低过拟合，提高样本外稳定性 `S_p`。

## Output Templates

### Quick Research Answer

```markdown
**XuanjiQuant 判断**
- 市场阶段 Ψ_cycle：x/100（依据：...）
- 资产趋势 Ω_asset：x/100（α_track=..., β_risk=...）
- 金融演化 ΔG_finance：x/100

**结论**
- 候选方向：...
- 主要风险：...
- 下一步验证：回测/数据补齐/参数敏感性测试
```

### Strategy Card

```markdown
**策略名称**：
**市场/标的池**：
**核心假设**：
**公式映射**：Ψ_cycle / Ω_asset / ΔG_finance 如何量化
**入场条件**：
**出场条件**：
**风控**：仓位、止损、最大回撤、黑名单
**回测要求**：区间、成本、基准、样本外验证
**失败条件**：什么结果说明策略无效
```

### Daily Review

```markdown
**今日数据状态**
- 数据源：
- 缺失/异常：

**市场状态**
- Ψ_cycle：
- 主要变化：

**候选资产排名**
1. 标的 — Ω_asset / ΔG_finance — 理由 — 风险

**复盘**
- 命中：
- 偏差：
- 噪声 N_noise：
- 下一轮参数调整：
```

## Guardrails

- 不要承诺收益，不要把公式输出当成买卖建议。
- 没有实际数据/回测时，必须标注“框架推演/待验证”。
- 对“全自动运行、无需人工调参”等原项目表述要谨慎：实际落地仍需 API、数据质量、回测、监控和风控。
- 生物公式、图像公式是 APEX 总体系的一部分；金融分析中除非用户明确要求，不要把它们硬塞进交易判断。
- 若涉及真实交易、下单、外部账户、API 写操作，必须先征得用户明确授权。

## Related Skills

- `akquant-backtest`：执行 A 股策略回测，尤其双均线/AKShare 数据。
- `quant-backtest-strategy`：Python 回测系统、报告导出、多因子/均值回归等。
- `search-skill`：需要多源市场信息检索与交叉验证时使用。
