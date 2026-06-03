# 机制卡片：EvalPlus 自动评测

## 核心机制
1. **Ground Truth 预计算**：用 `trusted_exec` 在 canonical_solution 上跑出期望输出，pickle 缓存
2. **双测试集**：base（原版输入）+ plus（EvalPlus 扩增输入），双重评分
3. **untrusted_check**：在沙箱 tempdir 中执行代码，比较输出（含 atol 浮点容差），超时控制
4. **pass@k 估算**：无偏估计量 `1 - comb(n-c,k)/comb(n,k)`，支持多样本评估
5. **并行评测**：ProcessPoolExecutor 并行，每个 completion_id 一个进程

## 任务格式（JSONL）
- task_id / prompt / entry_point / canonical_solution / base_input / plus_input / atol

## 评分函数
```python
# untrusted_check 返回 (status, [bool])，status ∈ {PASS, FAIL, TIMEOUT, ERROR}
# estimate_pass_at_k(num_samples, num_correct, k) → float
```

## 可抄要素
- 任务定义与执行解耦：data/ 负责下载解析，eval/ 负责评分
- 缓存 ground truth 避免重复计算
- 沙箱执行 + reliability_guard 防恶意代码
