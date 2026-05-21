# apex-emv-fusion Skill

## 触发条件
- 复杂任务（需要多步推理）
- 用户要求"准确"、"确定"
- 涉及外部系统状态/配置/版本的问题
- APEX 公式代入场景

## 执行流程
1. APEX 3秒自检（代入自己/代入公式/举一反三）
2. `SelfConsistencyChecker.check(question, n_paths=3)` — 多路径推理投票
3. `anti_hallucination_check(response)` — 防幻觉检查
4. `EMVOrchestrator.run(document, task)` — Rust EMV Core 调用
5. `GiniSelector.best_split(genes)` — 基尼增益选择最优技能
6. `SWRsBuffer.swr_triggered(fitness)` — 海马体重放检测
7. `verify_self_consistency(claim)` — 自我一致性验证

## 输入
- question: str（用户问题）
- document: str（可选，上下文文档）
- use_rust: bool（是否调用 Rust EMV Core，默认 True）

## 输出
```json
{
  "answer": "...",
  "phi_consistency": 0.85,
  "phi_anti": 0.9,
  "best_gene": {
    "name": "skill_0",
    "fitness": 1.005,
    "success_rate": 1.0
  },
  "gini_gain": 0.0218,
  "swr_triggered": true,
  "skillbank_len": 54
}
```

## 核心模块

### Rust EMV Core
- 二进制: `emv_skill/target/release/emv_skill`
- 调用方式: subprocess CLI（`--test` 跳过 API）
- Gini 增益: ΔGini = Gini父 - (N_L/N × Gini_L + N_R/N × Gini_R)
- 信息熵: H = -Σp_k × log₂(p_k)
- SWRs 阈值: 0.7

### Python 客户端
- 文件: `apex_emv_client.py`
- 类: `EMVOrchestrator`, `GiniSelector`, `SWRsBuffer`

## Gini 选择阈值
- min_gain: 0.01（最小分裂增益）
- swr_threshold: 0.7（SWRs 触发阈值）
- min_samples_leaf: 5（叶节点最小样本）

## 依赖
- Rust EMV: `~/.openclaw/workspace/apex-enlightenment/emv_skill/`
- Python: `apex_self_consistency.py`, `anti_hallucination_check.py`, `apex_emv_client.py`
- API Key: `FREEMODEL_API_KEY`
- 技能库: `/tmp/emv_skillbank.json`

## 来源
EMV_CLAW_SWRs_GINI_FUSION_PLAN.md (GPT-5.5 生成)
