#!/usr/bin/env python3
"""
进化循环管理器 - DEAP基因融合
基于DEAP进化算法思想，实现选择-交叉-变异-评估闭环

Usage:
    python3 evolution_loop.py evaluate <fitness_value>
    python3 evolution_loop.py select
    python3 evolution_loop.py mutate <value>
    python3 evolution_loop.py crossover <parent1> <parent2>
    python3 evolution_loop.py stats
"""

import json
import os
import sys
import time
import random
from pathlib import Path
from typing import List, Tuple, Optional

STATE_DIR = Path(__file__).parent / "state"
POPULATION_FILE = STATE_DIR / "evolution_population.jsonl"
FITNESS_FILE = STATE_DIR / "evolution_fitness.jsonl"
EVOLUTION_LOG = STATE_DIR / "evolution_log.jsonl"

# 进化参数
POPULATION_SIZE = 10
MUTATION_PROB = 0.2
CROSSOVER_PROB = 0.5
ELITE_COUNT = 2  # 保留最佳个体数

class EvolutionLoop:
    def __init__(self):
        self.state_dir = STATE_DIR
        self.state_dir.mkdir(parents=True, exist_ok=True)
        self._ensure_files()
        self.population = self._load_population()
        self.fitness_history = self._load_fitness()
    
    def _ensure_files(self):
        for f in [POPULATION_FILE, FITNESS_FILE, EVOLUTION_LOG]:
            if not f.exists():
                f.write_text("")
    
    def _load_population(self) -> List[float]:
        """加载种群"""
        pop = []
        try:
            with open(POPULATION_FILE, "r") as f:
                for line in f:
                    if line.strip():
                        pop.append(float(line.strip()))
        except:
            pass
        # 初始化种群
        if not pop:
            pop = [0.5] * POPULATION_SIZE  # 默认值
        return pop[-POPULATION_SIZE:]
    
    def _save_population(self):
        """保存种群"""
        with open(POPULATION_FILE, "w") as f:
            for ind in self.population:
                f.write(f"{ind}\n")
    
    def _load_fitness(self) -> List[float]:
        """加载适应度历史"""
        fitness = []
        try:
            with open(FITNESS_FILE, "r") as f:
                for line in f:
                    if line.strip():
                        try:
                            fitness.append(float(line.strip()))
                        except:
                            pass
        except:
            pass
        return fitness[-POPULATION_SIZE:]
    
    def _save_fitness(self):
        """保存适应度"""
        with open(FITNESS_FILE, "a") as f:
            for fit in self.fitness_history[-POPULATION_SIZE:]:
                f.write(f"{fit}\n")
    
    def _log_evolution(self, action, details):
        """记录进化日志"""
        log_entry = {
            "action": action,
            "details": details,
            "timestamp": time.time(),
            "population_avg": sum(self.population) / len(self.population) if self.population else 0
        }
        with open(EVOLUTION_LOG, "a") as f:
            f.write(json.dumps(log_entry, ensure_ascii=False) + "\n")
    
    def evaluate(self, gamma_value: float, awake_value: float) -> float:
        """
        评估适应度
        适应度 = awake增长 * gamma贡献权重
        """
        # 基于AWAKE和gamma的相关性计算适应度
        if not self.fitness_history:
            fitness = 0.5
        else:
            # 最近一次适应度
            last_fit = self.fitness_history[-1]
            # 适应度增长 = 历史惯性90% + 当前贡献10%
            fitness = max(0.0, min(1.0, last_fit * 0.9 + awake_value / 10.0 * 0.1))
        
        self.fitness_history.append(fitness)
        # 立即保存fitness历史
        with open(FITNESS_FILE, "a") as f:
            f.write(f"{fitness}\n")
        return fitness
    
    def select(self) -> List[float]:
        """
        选择：锦标赛选择
        从种群中选择较优个体
        """
        selected = []
        for _ in range(POPULATION_SIZE - ELITE_COUNT):
            # 随机选择3个个体进行锦标赛
            tournament = random.sample(self.population, min(3, len(self.population)))
            winner = max(tournament)  # 适应度越高越好
            selected.append(winner)
        
        # 保留精英
        sorted_pop = sorted(self.population, reverse=True)
        selected.extend(sorted_pop[:ELITE_COUNT])
        
        self.population = selected
        self._save_population()
        self._log_evolution("select", {"selected": selected[-5:]})
        
        return selected
    
    def crossover(self, parent1: float, parent2: float) -> Tuple[float, float]:
        """
        交叉：模拟二进制交叉
        """
        if random.random() < CROSSOVER_PROB:
            alpha = random.random()  # 交叉比例
            child1 = parent1 * alpha + parent2 * (1 - alpha)
            child2 = parent2 * alpha + parent1 * (1 - alpha)
            self._log_evolution("crossover", {
                "parent1": parent1, "parent2": parent2,
                "child1": child1, "child2": child2
            })
            return child1, child2
        return parent1, parent2
    
    def mutate(self, value: float, env_pressure: float = 0.5) -> float:
        """
        变异：高斯变异
        变异幅度与环境压力相关
        """
        if random.random() < MUTATION_PROB:
            sigma = 0.1 * (1 + env_pressure)  # 环境压力越大，变异越大
            mutated = value + random.gauss(0, sigma)
            mutated = max(0.1, min(2.0, mutated))  # 限制范围
            self._log_evolution("mutate", {
                "original": value, "mutated": mutated, "sigma": sigma
            })
            return mutated
        return value
    
    def evolve(self, env_pressure: float = 0.5, current_gamma: float = 0.0) -> float:
        """
        执行一轮进化
        返回: 新的gamma值
        """
        # 添加当前gamma到种群
        if current_gamma > 0:
            self.population.append(current_gamma)
            if len(self.population) > POPULATION_SIZE * 2:
                self.population = self.population[-POPULATION_SIZE:]
        
        # 选择
        selected = self.select()
        
        # 交叉
        new_population = []
        for i in range(0, len(selected) - 1, 2):
            c1, c2 = self.crossover(selected[i], selected[i+1])
            new_population.extend([c1, c2])
        
        # 变异
        new_population = [self.mutate(ind, env_pressure) for ind in new_population]
        
        # 更新种群
        self.population = new_population[-POPULATION_SIZE:]
        self._save_population()
        
        # 返回最佳个体
        best = max(self.population)
        self._log_evolution("evolve", {
            "best": best, "avg": sum(self.population) / len(self.population)
        })
        
        return best
    
    def get_fitness_boost(self, current_gamma: float, awake: float) -> float:
        """
        计算适应度提升（用于增强gamma）
        """
        # 评估当前个体
        fitness = self.evaluate(current_gamma, awake)
        
        # 执行进化
        env_pressure = float(os.environ.get("ENV_PRESSURE", "0.5"))
        evolved_gamma = self.evolve(env_pressure, current_gamma)
        
        # 计算增强量
        boost = (evolved_gamma - current_gamma) * 0.3  # 进化贡献30%
        
        return max(0.0, boost)
    
    def get_stats(self) -> dict:
        """获取进化统计"""
        return {
            "population_size": len(self.population),
            "population_avg": sum(self.population) / len(self.population) if self.population else 0,
            "population_min": min(self.population) if self.population else 0,
            "population_max": max(self.population) if self.population else 0,
            "fitness_history_len": len(self.fitness_history),
            "last_fitness": self.fitness_history[-1] if self.fitness_history else 0
        }


if __name__ == "__main__":
    evo = EvolutionLoop()
    
    if len(sys.argv) < 2:
        print("Usage: evolution_loop.py [evaluate|select|mutate|crossover|stats|evolve]")
        sys.exit(1)
    
    cmd = sys.argv[1]
    
    if cmd == "evaluate":
        gamma = float(sys.argv[2]) if len(sys.argv) > 2 else 0.5
        awake = float(sys.argv[3]) if len(sys.argv) > 3 else 7.0
        fitness = evo.evaluate(gamma, awake)
        print(f"Fitness: {fitness:.3f}")
    
    elif cmd == "select":
        selected = evo.select()
        print(f"Selected: {selected}")
    
    elif cmd == "mutate":
        val = float(sys.argv[2]) if len(sys.argv) > 2 else 0.5
        env_p = float(sys.argv[3]) if len(sys.argv) > 3 else 0.5
        mutated = evo.mutate(val, env_p)
        print(f"Mutated: {mutated:.3f}")
    
    elif cmd == "crossover":
        p1 = float(sys.argv[2]) if len(sys.argv) > 2 else 0.5
        p2 = float(sys.argv[3]) if len(sys.argv) > 3 else 0.5
        c1, c2 = evo.crossover(p1, p2)
        print(f"Children: {c1:.3f}, {c2:.3f}")
    
    elif cmd == "stats":
        print(json.dumps(evo.get_stats(), indent=2))
    
    elif cmd == "evolve":
        env_p = float(sys.argv[2]) if len(sys.argv) > 2 else 0.5
        current = float(sys.argv[3]) if len(sys.argv) > 3 else 0.5
        new_gamma = evo.evolve(env_p, current)
        print(f"New gamma: {new_gamma:.3f}")
