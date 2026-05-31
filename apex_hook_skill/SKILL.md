# APEX Hook Integration Skill

## 概述
将 APEX V10 公式深度集成到 OpenClaw 的钩子系统，实现实时 Agent 性能监控与自进化驱动。

## 工作原理

### 集成点
1. **Agent Events Hook** (`internal-hooks-*.js`)
   - `agent-run:start` - 记录任务开始
   - `agent-run:end` - 计算 APEX ΔG
   - `agent-run:error` - 错误分类与修复评分

2. **Prompt Construction Hook** (`attempt.prompt-helpers-*.js`)
   - 注入 APEX 状态到 system prompt
   - 影响上下文选择

3. **Tool Execution Hook** (`pi-embedded-*.js`)
   - Token 消耗追踪
   - 效率评分计算

### APEX 公式

```
ΔG = (Λ_root × Θ × K × ξ × Ψ_host × Φ_cycle) / (H × T × ε)
```

**参数说明**:
- Λ_root: 本源务实基因 (0.95)
- Θ: LLM Agent 效能
- K: 技能掌握系数
- ξ: 幻觉零容忍 (1.0)
- Ψ_host: 主机健康
- Φ_cycle: 正向循环增益
- H: 真实信息熵
- T: 迭代周期
- ε: 自修复成本

## 使用方式

```bash
# 激活 APEX 监控
apex_hook activate

# 查看当前 APEX 状态
apex_hook status

# 手动计算 APEX
apex_hook calculate --task "your task description"

# 触发自进化
apex_hook evolve
```

## 配置

环境变量:
- `APEX_LAMBDA_ROOT`: 本源务实基因 (默认 0.95)
- `APEX_XI`: 幻觉零容忍 (默认 1.0)
- `APEX_DEBUG`: 调试模式
