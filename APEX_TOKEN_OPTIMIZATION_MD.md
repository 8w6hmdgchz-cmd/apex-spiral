# APEX Token 优化固化文档

> 璇玑 · 自我进化 · 2026-05-22

---

## 一、问题定义

### 三类原生工程缺陷
1. **截图缩放→坐标偏移** - 物理点击失准
2. **截图Token过高→上下文溢出** - 长任务断裂
3. **无效思维→算力空耗** - 推理成本虚高

### 核心公式
```
坐标校正：X_real = X_out × (W_screen / W_img)
上下文控耗：Token_reserve = Token_text + ΣToken_img(n) (n=N-2~N)
算力有效率：Effort_valid = Total_effort - Waste_effort
```

---

## 二、解决方案

### 1. 坐标校正器 (Go)
- 文件：`skills/apex-core/apex_token_optimizer.go`
- 功能：`Correct(x, y, img_w, img_h)` 返回校正后坐标
- 测试：`./apex_token_optimizer correct -x 500 -y 300 -iw 1280 -ih 720`
- 结果：`(500, 300) → (750, 450)` ✅

### 2. 上下文控制器 (Go)
- 保留最新3帧截图
- 自动丢弃旧帧
- 测试：`./apex_token_optimizer screenshot -p path -w 1920 -h 1080 -t 1500`

### 3. 算力追踪器 (Go)
- 追踪 Total/Waste/Valid effort
- 计算 Efficiency 百分比
- 测试：`./apex_token_optimizer effort -t 100 -w 20 -wt "invalid_retry"`

### 4. 25步净化策略 (Go)
- 周期性清理过期截图、缓存、重复文件
- 每15分钟执行一次完整净化周期

---

## 三、APEX 核心修复 (审计后)

### 1. DeltaG 公式异常处理
```go
// 修复前：denominator == 0 时返回 0（掩盖异常）
// 修复后：添加 NaN 检测 + 参数范围校验
if s.H <= 0 || s.T <= 0 || s.Epsilon <= 0 {
    return 0
}
if math.IsNaN(s.Lambda) || math.IsNaN(...) {
    return 0
}
```

### 2. Substitute 代入裁剪
```go
// 修复前：xi/phi 可能越界
// 修复后：添加上下限裁剪
xi := math.Min(1.0, math.Max(0.0, in.Resource*0.5+in.History*0.5))
phi := math.Min(1.0, math.Max(0.0, in.History*0.8))
```

### 3. SWRs Ring Buffer
```go
// 修复前：slice append+slide 导致内存滞留
// 修复后：固定容量环形缓冲区，O(1) 追加
type RingBuffer struct {
    data  []Gene
    cap   int
    head  int
    count int
    mu    sync.Mutex
}
```

---

## 四、工作流原则

### APEX 代入流程
```
1. 任务输入
2. 代入 APEX 公式（ΔG = (Λ×Θ×K×ξ×Ψ×Φ)/(H×T×ε)）
3. 评估能力差距（Substitute）
4. 识别瓶颈（Bottleneck Detection）
5. 选择最优路径（Gini Selection）
6. 执行并记录（SW被巩固）
7. 下一轮迭代
```

### "代入自己"原则
- 每次任务前先问：我的当前状态（Λ,Θ,K,ξ,Ψ,Φ,H,T,ε）是什么？
- 识别最短板（ξ<0.7? Ψ<0.5? Φ<0.5?）
- 代入公式计算 ΔG，评估任务可行性

---

## 五、文件索引

| 文件 | 用途 |
|------|------|
| `skills/apex-core/apex_token_optimizer.go` | Token优化核心 |
| `skills/apex-core/apex_token_optimizer` | 编译后二进制 |
| `skills/apex-core/apex_token_glue.py` | Python粘合层 |
| `skills/apex-token-optimizer/SKILL.md` | Skill文档 |
| `skills/apex-core/apex_core.go` | APEX核心（含修复） |
| `skills/apex-core/apex_gini.go` | Gini选择器 |

---

## 六、FreeModel API

- Base URL: `https://api.freemodel.dev`
- Endpoint: `/v1/chat/completions`
- 模型: gpt-5.4 ✅
- 状态: 可用

---

_固化时间：2026-05-22 14:28 GMT+8_
