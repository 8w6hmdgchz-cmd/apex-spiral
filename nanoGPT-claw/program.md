
# NanoGPT-Claw AutoResearch Program

## 项目目标

使 NanoGPT-Claw 成为一个完整、真实、功能完善的 AI Agent 系统

### 核心目标
- 优化指标: 代码质量、功能完整性、测试覆盖率、稳定性

## 范围
```
目标: 提升代码质量与系统稳定性
可编辑文件: src/**/*.rs, tests/**/*.rs
只读文件: Cargo.toml, .git/**, .github/**, config/**
迭代预算: 每次迭代 5 分钟 (wall-clock)
最大迭代次数: 25
停止条件: 达到目标指标或达到最大迭代次数
```

## 验证命令
```bash
# 基础验证
cargo check --quiet
cargo test --quiet
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

## 核心指标 (Metrics)

| 指标 | 说明 | 方向 | 基线 | 目标 |
|------|------|--------|
| `compile` | 编译通过 | yes/no | ✅ | 保持 ✅ |
| `test_pass` | 测试通过数 | 越高越好 | 0 | 增加 |
| `warnings` | 警告数量 | 越低越好 | 48 | 0 |
| `clippy` | Clippy 警告 | 越低越好 | - | 0 |

## 迭代规则 (Rules)

### 单文件修改规则

1. **每次只修改一个文件
2. **修改范围严格限于核心功能，不要过度重构
3. **每个修改都必须通过编译验证

### Git 作为记忆 (Git as Memory)

1. **每次修改前先创建提交
2. **验证通过后保留,否则自动回滚
3. **每次提交记录详细信息: 改动、结果、指标

### 安全规则

1. **只修改 .rs 源代码文件
2. **不破坏现有功能
3. **失败自动回滚

## 研究方向 (Research Directions)

### 优先级 1: 清理警告和优化
- 修复所有 unused imports
- 删除 dead code
- 优化错误处理

### 优先级 2: 补充测试
- 为各个核心模块添加单元测试
- 集成测试

### 优先级 3: 完善功能
- 完善现有功能的错误处理
- 添加必要的文档

## 基线状态

第一次运行，建立当前状态。

