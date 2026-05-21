# R语言医学统计实战

## 一、环境准备

### 安装必要包
```r
# 安装
install.packages(c("tidyverse", "survival", "meta", "MatchIt", 
                   "pwr", "ggplot2", "ggpubr", "car", "rms"))

# 加载
library(tidyverse)
library(survival)
library(meta)
library(MatchIt)
library(pwr)
library(ggplot2)
```

---

## 二、数据导入与整理

### 导入数据
```r
# CSV
data <- read_csv("data.csv")

# Excel
library(readxl)
data <- read_excel("data.xlsx", sheet = 1)

# SPSS
library(haven)
data <- read_sav("data.sav")
```

### 数据概览
```r
str(data)           # 数据结构
summary(data)       # 描述统计
head(data)          # 前几行
View(data)          # 表格视图
```

### 数据整理
```r
library(dplyr)

# 筛选
data_filtered <- data %>% filter(age >= 18 & stage != "unknown")

# 选择列
data_select <- data %>% select(id, age, sex, treatment, outcome)

# 新增列
data <- data %>% mutate(
  bmi = weight / (height/100)^2,
  age_group = cut(age, breaks = c(0, 40, 60, 100), 
                   labels = c("young", "middle", "elder"))
)

# 分组汇总
summary_table <- data %>%
  group_by(group) %>%
  summarise(
    n = n(),
    mean_age = mean(age, na.rm = TRUE),
    sd_age = sd(age, na.rm = TRUE),
    event = sum(outcome == 1),
    .groups = "drop"
  )
```

---

## 三、基础统计分析

### t检验
```r
# 两组独立t检验
t.test(continuous_var ~ group, data = mydata, var.equal = TRUE)

# Welch's t检验（方差不齐）
t.test(continuous_var ~ group, data = mydata)

# 配对t检验
t.test(before, after, paired = TRUE)
```

### 方差分析（ANOVA）
```r
# 单因素ANOVA
aov_result <- aov(continuous_var ~ factor, data = mydata)
summary(aov_result)

# 两因素ANOVA
aov2 <- aov(continuous_var ~ factor1 * factor2, data = mydata)
summary(aov2)

# 事后比较（Tukey HSD）
library(multcomp)
 TukeyHSD(aov_result)
```

### 卡方检验
```r
# 四格表
chisq.test(table(data$treatment, data$outcome))

# Fisher精确概率（样本量<40或期望频数<5）
fisher.test(table(data$treatment, data$outcome))

# 配对四格表（McNemar）
mcnemar.test(matrix(c(a, b, c, d), nrow = 2))
```

### 非参数检验
```r
# Mann-Whitney U检验
wilcox.test(continuous_var ~ group, data = mydata)

# Kruskal-Wallis检验
kruskal.test(continuous_var ~ group, data = mydata)

# Wilcoxon配对符号秩检验
wilcox.test(before, after, paired = TRUE)
```

---

## 四、回归分析

### Logistic回归
```r
# 单因素
log1 <- glm(outcome ~ var1, data = mydata, family = binomial)
summary(log1)
exp(coef(log1))  # OR

# 多因素
log_multi <- glm(outcome ~ var1 + var2 + age + sex, 
                 data = mydata, family = binomial)
summary(log_multi)

# OR及95%CI
library(broom)
tidy(log_multi, exponentiate = TRUE, conf.int = TRUE)
```

### Cox回归
```r
library(survival)

# 构建生存对象
mydata <- mydata %>% mutate(
  time_status = Surv(time, status)
)

# 单因素Cox
cox1 <- coxph(time_status ~ treatment, data = mydata)
summary(cox1)

# 多因素Cox
cox_multi <- coxph(time_status ~ treatment + age + stage, 
                    data = mydata)
summary(cox_multi)

# 提取HR
tidy(cox_multi, exponentiate = TRUE, conf.int = TRUE)
```

### 生存曲线
```r
library(survminer)

fit <- survfit(Surv(time, status) ~ treatment, data = mydata)

# 绘制
ggsurvplot(fit, data = mydata,
            pval = TRUE,           # 显示p值
            risk.table = TRUE,      # 风险表
            conf.int = TRUE)        # 置信区间
```

---

## 五、Meta分析

### 二分类Meta分析
```r
library(meta)

# 随机效应模型
meta_res <- metabin(
  event.e = events_treatment,  # 治疗组事件数
  n.e = n_treatment,           # 治疗组总数
  event.c = events_control,     # 对照组事件数
  n.c = n_control,             # 对照组总数
  data = mydata,
  method = "Inverse",
  sm = "OR",                  # 效应量：OR/RR/RD
  random = TRUE,               # 随机效应模型
  prediction = TRUE            # 显示预测区间
)

# 森林图
forest(meta_res)

# 异质性
meta_res$I2                      # I²
metabias(meta_res, method = "Egger")  # Egger检验
```

### 连续变量Meta分析
```r
meta_cont <- metacont(
  n.e = n_treatment, mean.e = mean_treatment, sd.e = sd_treatment,
  n.c = n_control, mean.c = mean_control, sd.c = sd_control,
  data = mydata,
  random = TRUE,
  sm = "SMD"                   # 标准化均数差
)
forest(meta_cont)
```

---

## 六、倾向性评分匹配

```r
library(MatchIt)

# 估计倾向性评分并匹配
psm <- matchit(
  treatment ~ age + sex + bmi + baseline_severity,
  data = mydata,
  method = "nearest",     # 最近邻匹配
  ratio = 1               # 1:1匹配
)

# 查看匹配结果
summary(psm)

# 提取匹配后数据
matched_data <- match.data(psm)

# 在匹配数据上进行后续分析
log_psm <- glm(outcome ~ treatment, 
               data = matched_data, 
               family = binomial)
```

---

## 七、Power分析与样本量

```r
library(pwr)

# 两组均数比较（t检验）
pwr.t.test(d = 0.5,        # 效应量 Cohen's d
           sig.level = 0.05,
           power = 0.80,
           type = "two.sample",
           alternative = "two.sided")

# 两组率比较
pwr.2p.test(h = 0.3,      # Cohen's h
            sig.level = 0.05,
            power = 0.80)

# Logistic回归
pwr.f2.test(u = 3,        # 分子自由度（自变量数）
             v = 100,      # 分母自由度
             f2 = 0.15)    # 效应量
```

---

## 八、统计推断

### 正态性检验
```r
shapiro.test(mydata$continuous_var)

# QQ图
ggplot(mydata, aes(sample = continuous_var)) +
  stat_qq() + stat_qq_line()
```

### 方差齐性检验
```r
leveneTest(continuous_var ~ group, data = mydata)
```

### Bootstrap置信区间
```r
library(boot)

boot_mean <- function(data, indices) {
  d <- data[indices]
  return(mean(d))
}

boot_result <- boot(data = mydata$continuous_var, 
                   statistic = boot_mean, 
                   R = 1000)

boot.ci(boot_result, type = "bca")  # BCa区间
```

---

## 九、可视化

### 森林图（手动）
```r
ggplot(meta_data, aes(y = study, x = effect, xmin = lower, xmax = upper)) +
  geom_point() +
  geom_errorbarh() +
  geom_vline(xintercept = 0, linetype = "dashed") +
  labs(x = "Effect Size (OR)", y = "") +
  theme_minimal()
```

### 生存曲线
```r
library(survminer)
ggsurvplot(
  fit, 
  data = mydata,
  conf.int = TRUE,
  risk.table = TRUE,
  pval = TRUE,
  palette = c("red", "blue"),
  title = "Overall Survival by Treatment"
)
```

### 热图
```r
library(pheatmap)
pheatmap(correlation_matrix,
         display_numbers = TRUE,
         number_format = ".2f")
```

---

## 十、报告规范检查

```r
# 完整报告模板
library(report)

# 自动生成报告
report(t.test(continuous_var ~ group, data = mydata))
report(lm(outcome ~ var1 + var2, data = mydata))
```

---

*璇玑医学知识体系 · R语言实战*
