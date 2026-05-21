# Power分析

## 一、基本概念

### 定义
Power = 1 - β = 当H1为真时，正确拒绝H0的概率

### 含义
- Power = 0.80 → 当真实存在差异时，有80%概率检测到差异
- Power = 0.90 → 更严格，需要更大样本量

### 与样本量的关系
```
Power↑ → n↑
α↑ → Power↑（但假阳性风险↑）
δ↑ → Power↑（更大效应更易检出）
σ↑ → Power↓（变异越大越难检出）
```

---

## 二、效应量（Effect Size）

### 为什么重要
- p值受n影响，不直接反映效应大小
- 同样p<0.05，d=0.2 vs d=0.8意义完全不同
- 先确定临床有意义的最小效应，再算n

### Cohen's d 效应量分级
| d | 解释 |
|----|------|
| 0.2 | 小效应 |
| 0.5 | 中等效应 |
| 0.8 | 大效应 |

### 效应量来源
- 预试验数据
- 文献估计
- 临床经验
- 规范性标准（Cohen建议）

---

## 三、R实现

### 两组独立t检验
```r
library(pwr)

# 已知效应量0.5，求样本量
pwr.t.test(d = 0.5,        # Cohen's d
            sig.level = 0.05,
            power = 0.80,
            type = "two.sample",
            alternative = "two.sided")

# 输出：每组n=64

# 已知n，求power
pwr.t.test(n = 64, 
            d = 0.5,
            sig.level = 0.05,
            type = "two.sample")

# 已知n和power，求效应量
pwr.t.test(n = 50,
            power = 0.80,
            sig.level = 0.05,
            type = "two.sample")
```

### 两组率比较
```r
# 两组率比较
pwr.2p.test(h = 0.3,       # Cohen's h = arcsin(sqrt(p1)) - arcsin(sqrt(p2))
            sig.level = 0.05,
            power = 0.80,
            alternative = "two.sided")
```

### ANOVA
```r
# 单因素ANOVA（k组）
pwr.anova.test(k = 3,        # 组数
               f = 0.25,    # Cohen's f
               sig.level = 0.05,
               power = 0.80)
```

### 相关分析
```r
# Pearson相关
pwr.r.test(r = 0.3,        # 相关系数
            sig.level = 0.05,
            power = 0.80)
```

### 线性回归
```r
# 多元回归
pwr.f2.test(u = 3,          # 自变量个数
             v = 100,         # 误差自由度 = n - u - 1
             f2 = 0.15,      # Cohen's f²
             sig.level = 0.05)
```

### Logistic回归
```r
library(pwr)
# 二项逻辑回归
pwr.2.log.test(n = 100,
                OR = 2.0,
                sig.level = 0.05,
                power = 0.80)
```

### 生存分析
```r
library(powerSurvEpi)
# Cox回归
n <- qsurv(power = 0.80,
            hr = 0.6,       # 期望HR（<1表示治疗降低风险）
            alpha = 0.05,
            T = 3,           # 随访时间
            lambda0 = 0.1)   # 基线事件率
```

---

## 四、效应量估算

### 预试验数据
```r
# 从预试验估算效应量
pre_treatment <- c(5.2, 4.8, 5.5, 4.9, 5.0)
pre_control <- c(4.9, 4.7, 5.1, 4.6, 4.8)

# 计算d
library(effsize)
d <- cohen.d(pre_treatment, pre_control)
print(d)

# d = 0.45 → 中等效应
```

### 从文献估算
```r
# 已知文献的均值和SD
mean1 <- 10.5; sd1 <- 2.0
mean2 <- 9.0; sd2 <- 2.0
n1 <- 50; n2 <- 50

# 合并SD
pooled_sd <- sqrt(((n1-1)*sd1^2 + (n2-1)*sd2^2) / (n1+n2-2))
d <- (mean1 - mean2) / pooled_sd
print(d)
```

---

## 五、敏感性分析

### 样本量敏感性
```r
# 探索不同效应量下的样本量需求
effect_sizes <- c(0.2, 0.3, 0.5, 0.8)
results <- sapply(effect_sizes, function(d) {
  result <- pwr.t.test(d = d, sig.level = 0.05, power = 0.80, type = "two.sample")
  return(result$n)
})
names(results) <- effect_sizes
print(results)
```

---

## 六、非劣效性试验Power

### 非劣效性 vs 优效性
```r
# 非劣效性（NI）样本量
library(power)
power.ns = 0.80
alpha = 0.025  # 单侧

# 关键参数
delta_NI = 0.1    # 非劣效边界
sigma = 0.5        # 标准差
delta_tru = 0       # 假设真实差异=0（零假设：干预劣效）

# 样本量公式（近似）
n_NI <- 2 * ((qnorm(1-alpha) + qnorm(power.ns))^2 * sigma^2) / delta_NI^2
print(n_NI)
```

---

## 七、Cluster RCT样本量

### 设计效应
```r
# ICC = 组内相关系数
# m = 每cluster平均人数
ICC <- 0.02  # 文献估计
m <- 50       # 每cluster人数

DE <- 1 + (m - 1) * ICC
print(DE)  # 设计效应

# 简单随机样本量 × DE = 实际需要样本量
simple_n <- 100
cluster_n <- simple_n * DE
print(cluster_n)
```

---

## 八、常见问题

### Q：为什么我的研究Power只有0.5？
A：样本量不足，效应量预估过大，或变异估计不足

### Q：已经做了研究，P=0.06，没意义吗？
A：P接近0.05时power可能不足，建议重新评估效应量置信区间

### Q：可以不计算样本量吗？
A：不行。无充分理由的样本量是伦理问题（样本过少检出不到真效应=浪费受试者贡献）

---

*璇玑医学知识体系 · Power分析*
