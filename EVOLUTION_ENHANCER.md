# Evolution Loop 增强基因方案

## 当前问题

| 问题 | 值 | 根因 |
|------|-----|------|
| fitness_history | 4条后停滞 | evolution只被B4调用 |
| Γ_awake | 5.3（退步） | 无持续适应度反馈 |
| population_avg | 0.488 | 无选择压力 |

## 从GitHub提取的进化增强基因

### 1. DEAP - 进化策略核心

```python
# DEAP 简单进化策略
creator.create("FitnessMax", base.Fitness, weights=(1.0,))
creator.create("Individual", list, fitness=creator.FitnessMax)

toolbox.register("evaluate", lambda ind: sum(ind))
toolbox.register("mate", tools.cxTwoPoint)
toolbox.register("mutate", tools.mutFlipBit, indpb=0.05)
toolbox.register("select", tools.selTournament, tournsize=3)

# 进化循环
for gen in range(100):
    # 评估
    fitnesses = list(map(toolbox.evaluate, population))
    for ind, fit in zip(population, fitnesses):
        ind.fitness.values = fit
    
    # 选择
    offspring = toolbox.select(population, len(population))
    offspring = list(map(toolbox.clone, offspring))
    
    # 交叉
    for child1, child2 in zip(offspring[::2], offspring[1::2]):
        if random.random() < CXPB:
            toolbox.mate(child1, child2)
            del child1.fitness.values
            del child2.fitness.values
    
    # 变异
    for mutant in offspring:
        if random.random() < MUTPB:
            toolbox.mutate(mutant)
            del mutant.fitness.values
    
    population = offspring
```

### 2. Optuna - 超参数优化采样

```python
# Optuna TPE (Tree-structured Parzen Estimator)
class TPESampler:
    def sample(self, study, trial):
        # 基于历史结果构建GMM
        completed = [t for t in study.trials if t.state == TrialState.COMPLETE]
        
        if len(completed) < 10:
            return random.uniform(0, 1)
        
        # 分离好和坏的样本
        good = [t.params.get('x', 0.5) for t in completed if t.value > median]
        bad = [t.params.get('x', 0.5) for t in completed if t.value <= median]
        
        # 用Parzen估计构建分布
        gx = self.estimate_parzen(good)
        ly = self.estimate_parzen(bad)
        
        # 采样
        return gx / (gx + ly)
```

### 3. CMA-ES - 协方差矩阵适应

```python
# CMA-ES 进化策略
class CMAEvolutionStrategy:
    def __init__(self, dim, sigma):
        self.mean = [0] * dim
        self.sigma = sigma
        self.C = identity(dim)
        self.ps = [0] * dim
        self.pc = [0] * dim
        self.dim = dim
        self.lambda_ = 4 + int(3 * math.log(dim))
    
    def ask(self):
        """生成候选解"""
        samples = []
        for _ in range(self.lambda_):
            z = numpy.random.normal(0, 1, self.dim)
            y = numpy.dot(self.sigma, numpy.dot(self.C, z))
            x = self.mean + y
            samples.append(x)
        return samples
    
    def tell(self, solutions, func_values):
        """更新分布参数"""
        # 排序
        sorted_idx = numpy.argsort(func_values)
        
        # 选择top μ
        weights = [math.log(self.lambda_/2 + 0.5) - math.log(i+1) 
                   for i in range(self.lambda_//2)]
        weights = [w / sum(weights) for w in weights]
        
        # 加权重组
        y_w = sum(w * solutions[i] for i, w in zip(sorted_idx[:self.lambda_//2], weights))
        
        # 更新均值
        self.mean += self.sigma * y_w
```

## 增强方案

### 方案A：增加fitness反馈频率
- 当前：只在B4时调用evolution
- 修复：每轮都评估fitness，但不改变population

### 方案B：引入CMA-ES自适应步长
- 用CMA-ES替代固定高斯变异
- 自动调整搜索步长

### 方案C：加入历史适应度学习
- 基于fitness_history学习趋势
- 预测最优解方向

## 实施

### 增强版evolution_loop.py

```python
import numpy as np
from scipy.stats import norm

class EnhancedEvolutionLoop:
    """增强版进化循环：结合DEAP + CMA-ES"""
    
    def __init__(self, dim=10, sigma=0.5):
        self.dim = dim
        self.sigma = sigma  # 搜索步长
        self.mean = np.random.random(dim)
        self.C = np.eye(dim)  # 协方差矩阵
        self.pc = np.zeros(dim)
        self.ps = np.zeros(dim)
        self.popsize = 4 + int(3 * np.log(dim))
        
    def ask(self):
        """生成候选解"""
        samples = []
        for _ in range(self.popsize):
            z = np.random.randn(self.dim)
            y = self.sigma * np.dot(np.linalg.cholesky(self.C), z)
            samples.append(self.mean + y)
        return samples
    
    def tell(self, fitnesses):
        """根据适应度更新分布"""
        # 排序
        sorted_idx = np.argsort(fitnesses)[::-1]  # 降序
        best = sorted_idx[0]
        
        # 更新步长
        self.sigma *= np.exp(0.3 * (np.mean(fitnesses) - fitnesses[best]) / max(fitnesses))
        self.sigma = max(0.01, min(1.0, self.sigma))
        
        # 更新均值
        self.mean = np.mean([self.ask()[i] for i in sorted_idx[:3]], axis=0)
        
        return best
    
    def evaluate(self, gamma, awake, env_pressure):
        """评估当前解的适应度"""
        # 基于环境压力和当前状态计算适应度
        base = 0.5
        gamma_contrib = (gamma - 1.0) * 0.2
        awake_contrib = awake / 20.0
        env_contrib = env_pressure * 0.1
        
        fitness = base + gamma_contrib + awake_contrib + env_contrib
        return max(0.0, min(1.0, fitness))
```

---
*提取来源：GitHub DEAP/Optuna/CMA-ES*
*时间：2026-05-19 18:50*
