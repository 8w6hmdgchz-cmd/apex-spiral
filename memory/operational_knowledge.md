# Operational Knowledge — 璇玑操作知识库

> 每次新操作写入，下次不再现学。  
> 来源：实际跑通的代码 + 真实踩过的坑。

---

## 工具自注册机制（2026-06-02 吸收）

**来源**：hermes-agent `tools/registry.py`  
**落地**：`apex-spiral/py/apex_spiral/tools/`

**核心要点**：
- 工具文件模块顶层 `@tool("name", "desc")` 即注册
- 外部无需维护工具列表，registry 单例自动发现
- 同步 + 异步函数混用，必须拆 `invoke()`（同步入口）+ `__call__()`（异步入口）
- tag 用于按场景过滤（`reg.list(tag='math')`）
- 线程安全：`threading.Lock` 保护内部 dict

**踩过的坑**：
- ❌ 同步/异步入口混用 → coroutine 未 await → RuntimeWarning
- ✅ 已修复：见 `memory/failure_cases.jsonl` 2026-06-02 第一条

**已验证**：`python3 -m apex_spiral.tools.registry_demo` 跑通，3 工具 / 混合同步异步 / tag 过滤 / 真实结果。

**下次不要再重新设计这个**，**直接复用** `apex_spiral/tools/registry.py`。
