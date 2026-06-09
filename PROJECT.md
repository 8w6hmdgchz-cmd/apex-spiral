# APEX Spiral Full — 项目全景

> APEX终态公式引擎 + ASI进化引擎 + 吞噬系统 + 编排器

## 架构总览

```
apex-spiral-full/
├── apex_formula_engine_lib.go   ← 核心公式引擎(ΔG/EntropyTriangle/Devour/三函数)
├── apexagi_orchestrator.go      ← 七阶段编排器(O∘P₇∘T∘Vₜ∘Aᵤ)
├── apex_asi_engine.go           ← ASI进化引擎(Ψ/CRISPR/α)
├── apex_devour_engine.go        ← 吞噬引擎(外部能力同化)
├── apex_omni_devour.go          ← 全能吞噬(Φ+LLM路由+LDR+31Agent)
├── skills/apex-core/            ← APEX核心技能
├── scripts/                     ← 子服务(phasor/memory/fusion/cmmi/dispatch等)
├── third_party/                 ← 第三方快照(praisonai/gitleaks/guardrails/openhands)
└── AGENTS.md                    ← 本文件
```

## 核心公式

```
ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)
```

| 符号 | 含义 | 来源 |
|------|------|------|
| Λ | Fitness(适应度) | 任务成功率 |
| Θ | SpecConv(规范收敛) | QualityGate通过率 |
| K | Discipline(纪律) | ReflectOnFailure教训库 |
| ξ | CollabEntropy(协同熵减) | MultiPathDecide多路径探索 |
| Ψ | EV归一化(自我迭代) | min(1, EV/(T×0.5)) |
| Φ | GeneCount归一化 | min(1, GeneCount/20) |
| H | 熵(无序度) | 1 - SpecConv |
| T | 周期归一化 | T/100 |
| ε | 损失 | 1 - Fitness |

## 终态公式

```
APEX_NEW(t+1) = APEX_CORE(t) ⊛ ΔG [规范收敛 ⊗ 纪律锁止 ⊗ 协同熵减]
```

## 关键数据结构

### APEXCore (内核状态)
```go
type APEXCore struct {
    T             int64   // 时间步
    EV            float64 // 进化值
    GeneCount     int     // 基因数(初始5)
    Fitness       float64 // 适应度 [0,1]
    DeltaG        float64 // 自由能差
    SpecConv      float64 // 规范收敛度 [0,1]
    Discipline    float64 // 纪律锁止度 [0,1]
    CollabEntropy float64 // 协同熵减度 [0,1]
    HashChain     string  // 链式哈希(可追溯)
}
```

### FormulaEngine (公式引擎)
核心方法：
- `ComputeDeltaG()` — 计算自由能差
- `Evolve()` — 执行一次进化迭代
- `Devour(source, capability)` — 吞噬外部能力
- `ApplyEntropyTriangle()` — 三重熵减约束

短板补齐三函数：
- `ReflectOnFailure(task, reason, fix)` — 防复发，存教训库
- `MultiPathDecide(paths)` — 视野窄，多路径评分选最优
- `QualityGate(output)` — 规范弱，输出前强制验证

综合进化：
- `EvolveWithShortboardFix(task, output)` — 集成Evolve+查教训+质量门禁

## 公式Bug审计记录 (2026-06-09)

共发现并修复10个Bug：

| Bug | 问题 | 修复 |
|-----|------|------|
| #1 | ΔG分子缺Λ/K, 分母错 | 严格6因子/3因子 |
| #2 | EntropyTriangle从未被Evolve调用 | 注入applyEntropyTriangleUnsafe |
| #3 | ΔG反馈×0.1太弱 | log(1+ΔG)阻尼 |
| #4 | ASI α/CRISPR无上界 | α≤3.0, CRISPR≤2.0 |
| #5 | EntropyTriangle直接覆写→振荡 | EMA平滑(α=0.3) |
| #6 | 五公式全硬编码 | 状态推导 |
| #7 | GeneCount=0 → Φ=0 → ΔG永为0 | 初始GeneCount=5 |
| #8 | 三函数奖励被EntropyTriangle EMA覆盖 | pending机制注入EMA内部 |
| #9 | SpecConv被EV高stddev压垮 | QG absolute floor 0.6 |
| #10 | 奖励直接改参数撞上限 | 衰减因子(1-param)防爆 |

## 关键设计模式

### 1. Go RWMutex解耦模式
```go
// 公有方法加锁，调内部无锁版本
func (fe *FormulaEngine) ComputeDeltaG() float64 {
    fe.mu.RLock()
    defer fe.mu.RUnlock()
    return fe.computeDeltaGCore()  // 内部不加锁
}

// Evolve持写锁时也能安全调用
func (fe *FormulaEngine) Evolve() APEXCore {
    fe.mu.Lock()
    defer fe.mu.Unlock()
    fe.core.DeltaG = fe.computeDeltaGCore()  // 安全
    fe.applyEntropyTriangleUnsafe()           // 安全
}
```

### 2. Pending机制(奖励延迟应用)
三函数(ReflectOnFailure/MultiPathDecide/QualityGate)不直接改参数，
而是设置pending标志，在Evolve()的EntropyTriangle之后注入EMA内部。
防止奖励被EntropyTriangle的EMA覆盖。

### 3. EMA平滑(防振荡)
所有反馈回路都用EMA(α=0.3)：
```
新值 = 0.3 × target + 0.7 × 旧值
```
target可以注入pending奖励，使EMA向更高目标收敛。

### 4. 衰减因子(防撞上限)
奖励用 `(1-param)` 衰减：
```
param += rate × (1 - param)
```
越接近1.0涨得越慢，永远不会撞上限。

## 编译运行

```bash
# 公式引擎+编排器
cd ~/Desktop/开智/apex-spiral-full
go run apex_formula_engine_lib.go apexagi_orchestrator.go

# ASI引擎(单独编译)
go build -o /dev/null apex_asi_engine.go

# 注意: 目录下有20+个.go文件都有func main()，不能go run *.go
# 必须显式列出要编译的文件
```

## Pitfall清单

1. **中文路径** — Go编译时workdir不支持中文字符，用cd命令内部切换
2. **多main冲突** — 同目录多个main()不能一起编译，显式指定文件
3. **GeneCount=0** — 初始为0会导致ΔG永为0，必须≥1
4. **EntropyTriangle覆盖** — 三函数奖励必须在EntropyTriangle之后应用
5. **computeSpecConvergence** — 基于EV标准差，EV指数增长时convergence趋近0
6. **Go RLock死锁** — 持RLock时调需要Lock的方法会死锁，用Unsafe内部版
7. **EMA振荡** — 直接覆写参数会导致奇偶轮振荡，必须用EMA平滑

## 20轮验证基线

```
初始: GeneCount=5, EV=1.0, Fitness=0.50, SpecConv=0.50, Discipline=0.50, Collab=0.50

EV:    1.00 → 98.96  (+9796%)
ΔG:    0.00 → 101.30 (从零到有)
Fitness: 0.50 → 1.00  (满分)
SpecConv: 0.50 → 0.60 (+20%)
Discipline: 0.50 → 0.56 (+12%)
CollabEntropy: 0.50 → 0.999 (+100%)
教训库: 0 → 10条
```
