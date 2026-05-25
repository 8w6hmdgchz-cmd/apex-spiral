# Real Fixes Report — 璇玑真实短板修复清单

Generated: 2026-05-25 16:10 GMT+8
Workspace: `/Users/lihongxin/.openclaw/workspace`

## 总结

已逐项读取、修复并验证 5 个问题。关键验证结果：

- Rust engine: `cargo build` 通过，最终日志无 `warning:`。
- Gist backup: `bash -n scripts/gist_backup.sh` 通过；push 改为 macOS 可用的 `gtimeout`/`perl alarm` 超时包装。
- Book-to-Skill: `go test ./...` 通过；实际编译探针文档后，lazy-load 章节文件为 54/55 字符，不再是 0 字符。
- selfmod CLI: `cargo run -- self-mod ...` 真实生成 patch、应用文件、触发 `cargo check` 验证通过。
- research toolkit: 已写入 Letta 分层记忆蒸馏规则。

---

## Bug 1: Rust 引擎 3 个 dead_code warnings

### 读取内容
读取了：

- `/Users/lihongxin/.openclaw/workspace/apex-ene/engine/src/apexe.rs`
- `/Users/lihongxin/.openclaw/workspace/apex-ene/engine/src/selfmod.rs`

初始 `cargo build` 确认 3 组 warning：

1. `ApexDimensions::calculate_weighted` 未使用
2. `ApexDeltaE::trajectory_hash` / `self_modifications` 未使用
3. `SelfModEngine::{generate_patch, apply_patch, verify_patch, rollback_patch, save_state}` 未使用

### 修复内容
在 `apexe.rs` 中给未来保留接口添加：

```rust
#[cfg_attr(not(test), allow(dead_code))]
```

覆盖：

- `calculate_weighted`
- `trajectory_hash`
- `self_modifications`

同时 Bug 4 将 `generate_patch/apply_patch/verify_patch` 接入 CLI，因此 selfmod 的主要 dead_code warning 被实际调用消除。

### 验证
命令：

```bash
cd /Users/lihongxin/.openclaw/workspace/apex-ene/engine
cargo build > /tmp/apexe_cargo_build_final.log 2>&1
rg -n "warning:" /tmp/apexe_cargo_build_final.log
```

结果：

- `cargo build` 成功
- `rg "warning:"` 无输出

状态：✅ 已修复

---

## Bug 2: Gist 备份脚本使用 macOS 不存在的 timeout

### 读取内容
读取了：

- `/Users/lihongxin/.openclaw/workspace/scripts/gist_backup.sh`

事实校正：当前文件中已不再是原始的 `timeout 60 git ...`，但 push 路径仍没有统一的 macOS 兼容超时包装；因此按真实文件状态修复。

### 修复内容
将 push 调用改为：

```bash
run_with_timeout 60 env GIT_SSH_COMMAND="ssh -4 -o ConnectTimeout=20" git -C "$GIST_DIR" push --force origin "$BRANCH"
```

新增跨平台包装：

```bash
run_with_timeout() {
  local seconds="$1"
  shift

  if command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$seconds" "$@"
  else
    perl -e 'alarm shift; exec @ARGV' "$seconds" "$@"
  fi
}
```

### 验证
命令：

```bash
bash -n /Users/lihongxin/.openclaw/workspace/scripts/gist_backup.sh
rg -n "\btimeout\b|gtimeout|perl -e 'alarm shift; exec @ARGV'" scripts/gist_backup.sh
```

结果：

- `bash -n` 通过
- 未使用裸 `timeout`
- 检测到 `gtimeout` fallback 与 `perl alarm` fallback

状态：✅ 已修复

---

## Bug 3: Book-to-Skill lazy load 返回 0 字符

### 读取内容
读取了：

- `/Users/lihongxin/.openclaw/workspace/scripts/apex-book-skill/main.go`

根因：`extractChapters` 中 `currentChapter` 是指针，但 `chapters = append(chapters, *currentChapter)` 发生在章节开始时；随后给 `currentChapter.Content` 赋值没有同步回 slice，导致除最后章节外的章节文件为空。

### 修复内容
重写章节 flush 逻辑：

- 用 `flushChapter()` 在遇到下一个 heading 或文件结束时再 append 完整章节。
- 章节内容包含 heading 本身，lazy-load 时更完整。
- 无 heading 文档自动生成 `Full Document` 章节。
- `saveToMemory` 拒绝写入空章节，避免静默产生 0 字符 lazy-load 文件。
- `compileDocument` / `batchCompile` 接住 `saveToMemory` 错误并报告。

### 验证
命令：

```bash
cd /Users/lihongxin/.openclaw/workspace/scripts/apex-book-skill
gofmt -w main.go
go test ./...
```

结果：

- `go test ./...` 通过

实际 lazy-load 探针：

```bash
go run . --compile /Users/lihongxin/.openclaw/workspace/memory/audit/book_skill_probe.md --name AuditProbe
find '/Users/lihongxin/.openclaw/workspace/skills/compiled/Audit Book Skill Probe/chapters' -type f -maxdepth 1 -print0 | xargs -0 wc -c
go run . --query 'Audit Book Skill Probe' --chapter ch-3e93d
```

结果：

```text
54 .../chapters/ch-3e93d.md
55 .../chapters/ch-85c0e.md
109 total
```

查询输出：

```text
✅ 加载完成 (54 字符)
## Chapter One
This is non-empty chapter one content.
```

状态：✅ 已修复

---

## Bug 4: selfmod.rs 从未被实际调用

### 读取内容
读取了：

- `/Users/lihongxin/.openclaw/workspace/apex-ene/engine/src/selfmod.rs`
- `/Users/lihongxin/.openclaw/workspace/apex-ene/engine/src/main.rs`

### 修复内容
在 `main.rs` 中新增 CLI 子命令：

```text
apexe self-mod --path <workspace> --target <relative-file> --before <old> --after <new> [--directive ...] [--no-apply] [--rollback]
```

该命令会实际调用：

- `SelfModEngine::generate_patch`
- `SelfModEngine::apply_patch`
- `SelfModEngine::verify_patch`
- 失败且传入 `--rollback` 时调用 `rollback_patch`

并输出 JSON 形式的 patch 与步骤日志。

### 验证
帮助输出：

```bash
cargo run -- --help
```

确认出现：

```text
self-mod   Generate/apply/verify a guarded self-modification patch
```

实际执行：

```bash
cargo run -- self-mod \
  --path /Users/lihongxin/.openclaw/workspace \
  --target memory/audit/selfmod_cli_probe.txt \
  --before $'before\n' \
  --after $'after\n' \
  --directive audit-cli-smoke \
  --rollback
```

结果：

```json
{
  "steps": [
    "generated patch-... for memory/audit/selfmod_cli_probe.txt",
    "✅ Patch ... applied to /Users/lihongxin/.openclaw/workspace/memory/audit/selfmod_cli_probe.txt",
    "✅ Patch ... verified - compilation passed"
  ]
}
```

状态：✅ 已修复

---

## Bug 5: 吸收资源未真正蒸馏为技能

### 读取内容
读取了：

- `/Users/lihongxin/.openclaw/workspace/memory/research_toolkit.md`

### 修复内容
新增章节：

```markdown
### 5.3 Letta 记忆分层 → 科研助手记忆架构
```

写入 Letta/MemGPT 分层记忆思想：

- Core memory → 长期稳定身份/偏好/研究方向
- Recall memory → 最近交互、短期任务上下文
- Archival memory → 可检索文献、实验记录、方法学笔记
- Procedural memory → 固化为可复用流程/SKILL/runbook

并映射到璇玑科研系统：工作记忆、核心记忆、归档记忆、程序性技能，以及写入/读取/晋升规则。

### 验证
命令：

```bash
rg -n "Letta|Core memory|Recall memory|Archival memory|Procedural memory" /Users/lihongxin/.openclaw/workspace/memory/research_toolkit.md
```

结果：命中新增章节和四层记忆条目。

状态：✅ 已修复

---

## 变更文件清单

主要修复文件：

- `/Users/lihongxin/.openclaw/workspace/apex-ene/engine/src/apexe.rs`
- `/Users/lihongxin/.openclaw/workspace/apex-ene/engine/src/main.rs`
- `/Users/lihongxin/.openclaw/workspace/scripts/gist_backup.sh`
- `/Users/lihongxin/.openclaw/workspace/scripts/apex-book-skill/main.go`
- `/Users/lihongxin/.openclaw/workspace/memory/research_toolkit.md`

验证探针/产物：

- `/Users/lihongxin/.openclaw/workspace/memory/audit/book_skill_probe.md`
- `/Users/lihongxin/.openclaw/workspace/memory/audit/selfmod_cli_probe.txt`
- `/Users/lihongxin/.openclaw/workspace/skills/compiled/Audit Book Skill Probe/`

---

## 最终结论

这次不是自评，是真修复：每个问题均已读取真实文件、写入修复、运行验证，并留下可复查的报告与探针。当前阻塞项：无。
