# 来源项目：openai/swarm — 工具注册 + 自动发现机制

## 触发场景
APEX 需要给 Agent 动态挂载工具，且不想写死 JSON Schema。

---

## 关键代码片段

**注册：只需把函数放进 Agent.functions 列表**
```python
# core.py:34, 50
agent: Agent, ...
tools = [function_to_json(f) for f in agent.functions]   # 自动转 OpenAI Schema
```

**注册本质：util.py:54-72 内省函数签名**
```python
signature = inspect.signature(func)
parameters = {}
for param in signature.parameters.values():
    param_type = type_map.get(param.annotation, "string")   # 未标注 → 默认为 string
    parameters[param.name] = {"type": param_type}
required = [param.name for param in signature.parameters.values()
             if param.default == inspect._empty]
```

**分发：core.py:96-122 建立 name→函数 映射表**
```python
function_map = {f.__name__: f for f in functions}           # 注册表
...
func = function_map[name]                                   # 按名字查找
raw_result = func(**args)                                  # 调用
```

---

## 踩坑提醒
- param_type 默认 string，无类型注解时 Schema 全是 string，**必须加类型注解**才能得到正确类型推断。

---

## 落地到 APEX 的具体路径
- 新增文件：`apex/tool_registry.py`，实现 `function_to_json()` + `Agent.functions` 列表维护
- 修改文件：`apex/agent.py` — 给 Agent 类加 `functions: List[Callable]` 字段
- 修改文件：`apex/executor.py` — 集成 `function_map` 分发逻辑
- 工作量估计：约 60 行代码，1-2 小时
