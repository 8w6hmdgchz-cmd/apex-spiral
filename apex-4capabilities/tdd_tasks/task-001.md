# TDD Task 001: Output Control Benchmark Runner

## 需求 (Requirement)
创建一个命令行工具，用于运行本地 APEX eval cases 并输出 pass/fail 结果。

## 验收标准 (Acceptance Criteria)
1. 接受 --eval 参数指定 eval 类型 (output_control / repair / all)
2. 输出 JSON 格式结果
3. 返回码 0 表示全部通过，非 0 表示有失败
4. 结果包含 passed, summary, results 字段

## TDD 流程

### Phase 1: 红 (Write Failing Test)
```bash
python3 apex-4capabilities/tdd_tasks/test_runner.py --eval output_control
# 预期: 失败，因为代码还未写
```

### Phase 2: 绿 (Write Minimal Code)
```bash
# 写最简单的实现让测试通过
```

### Phase 3: 重构 (Refactor)
```bash
# 优化代码，保持测试通过
```
