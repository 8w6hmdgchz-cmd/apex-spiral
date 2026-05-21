# apex-core Skill

> APEX 自进化推理核心 - OpenClaw 原生集成
> 核心原则：Go/Rust 实现核心逻辑，Python 只做粘合

## 功能

APEX 自进化推理引擎的 OpenClaw 原生集成：

1. **公式代入**（`substitute`）：任务前自动代入 APEX 公式，评估能力差距
2. **Gini 选择**（`gini`）：多路径推理最优选择，基尼增益决策
3. **SWRs 巩固**（`swr`）：海马体重放记忆巩固，fitness >= 0.7 触发
4. **ΔG 评估**（`eval`）：APEX 主公式评估，量化进化速率

## 触发条件

- 复杂任务（多步推理）
- 用户要求"准确"、"确定"
- 涉及外部系统状态/配置
- APEX 公式代入场景

## 技术架构

```
用户消息
  ├─→ apex_core substitute → APEX 公式代入（Go）
  ├─→ apex_core gini      → Gini 增益选择（Go）
  └─→ apex_core swr       → SWRs 巩固（Go）
                              └─→ Rust emv_skill（可选）
```

## 二进制

- **Go CLI**: `skills/apex-core/apex_core`
- **Rust EMV**: `emv_skill/target/release/emv_skill`

## CLI 用法

```bash
cd skills/apex-core && bash BUILD.sh  # 编译

# 公式代入
./apex_core substitute -t "修复PHI_RATIO" -c 0.8 -h 0.6 -r 0.7

# Gini 选择
./apex_gini '["路径1","路径2","路径3"]'

# SWRs 巩固
./apex_core swr -add gene_123 -f 0.85

# ΔG 评估
./apex_core eval -l 0.9 -t 0.8 -k 0.7 -x 0.8 -p 0.6 -f 0.6 -he 0.3 -ti 0.2 -e 0.1
```

## Python 粘合

```python
import subprocess, json

def apex_substitute(task, cap, hist, res):
    r = subprocess.run(
        ["./apex_core", "substitute", "-t", task, "-c", str(cap), "-h", str(hist), "-r", str(res)],
        capture_output=True, text=True, timeout=10
    )
    return json.loads(r.stdout)
```

## APEX 公式

```
ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)

Gini = 1 - Σp_k²
ΔGini = Gini父 - (N_L/N × Gini_L + N_R/N × Gini_R)
H = -Σp_k × log₂(p_k)
```

## 来源

- 方案: `OPENCLAW_APEX_INTEGRATION.md`
- Rust EMV: `emv_skill/`
- Go 核心: `skills/apex-core/`
