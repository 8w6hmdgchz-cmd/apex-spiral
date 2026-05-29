#!/usr/bin/env python3
"""
APEX EMV 融合客户端 - Python 调用 Rust EMV Core
来源: EMV_CLAW_SWRs_GINI_FUSION_PLAN.md
"""
import subprocess
import json
import os
import math
from datetime import datetime

# 路径配置
EMV_BINARY = "/Users/lihongxin/.openclaw/workspace/a2a-resources/emv_skill/target/release/emv_skill"
SKILLBANK_PATH = "/tmp/emv_skillbank.json"
GINI_SKILL_PATH = "/Users/lihongxin/.openclaw/workspace/a2a-resources/emv_skill/target/release/gini_skill"

class EMVOrchestrator:
    """
    EMV 熵 Skill 编排器
    
    流程:
    1. APEX 3秒自检
    2. SelfConsistency 多路径推理
    3. Rust EMV Core (Gini选择 + SWRs巩固)
    4. 防幻觉检查
    """
    
    def __init__(self):
        self.api_key = os.environ.get("FREEMODEL_API_KEY", "")
        self.skillbank = self._load_skillbank()
    
    def _load_skillbank(self):
        """从 JSON 加载技能库"""
        if not os.path.exists(SKILLBANK_PATH):
            return []
        try:
            with open(SKILLBANK_PATH) as f:
                data = json.load(f)
                if isinstance(data, list):
                    return data
                elif isinstance(data, dict) and "genes" in data:
                    return data["genes"]
                return []
        except:
            return []
    
    def run(self, document: str, task: str, use_rust: bool = True) -> dict:
        """
        执行 EMV 融合推理
        
        Args:
            document: 上下文文档
            task: 任务描述
            use_rust: 是否使用 Rust EMV Core（需要API）
        
        Returns:
            {
                "success": bool,
                "best_gene": dict,
                "genes": list,
                "gini_gain": float,
                "swr_triggered": bool,
                "skillbank_len": int
            }
        """
        result = {
            "success": False,
            "best_gene": None,
            "genes": self.skillbank,
            "gini_gain": 0.0,
            "swr_triggered": False,
            "skillbank_len": len(self.skillbank),
            "timestamp": datetime.now().isoformat()
        }
        
        if use_rust and os.path.exists(EMV_BINARY):
            # 调用 Rust EMV Core
            run_result = self._call_rust(document, task)
            result.update(run_result)
        elif use_rust:
            # Rust 二进制不存在，输出警告
            result["warning"] = "Rust EMV binary not found"
        
        # 从技能库选最佳（按 fitness 排序）
        if self.skillbank:
            sorted_genes = sorted(
                self.skillbank,
                key=lambda g: g.get("fitness", g.get("total_reward", 0)),
                reverse=True
            )
            result["best_gene"] = sorted_genes[0] if sorted_genes else None
        
        return result
    
    def _call_rust(self, document: str, task: str) -> dict:
        """调用 Rust EMV Core subprocess"""
        env = os.environ.copy()
        if self.api_key:
            env["FREEMODEL_API_KEY"] = self.api_key
        
        cmd = [EMV_BINARY, "--test", document, task]
        
        try:
            r = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=10,  # 10秒超时
                env=env
            )
            
            output = r.stdout
            
            # 解析 Gini 增益
            gini_gain = 0.0
            for line in output.split("\n"):
                if "最优分裂增益:" in line:
                    try:
                        gini_gain = float(line.split(":")[1].strip())
                    except:
                        pass
            
            # 解析 SWRs
            swr_triggered = False
            for line in output.split("\n"):
                if "SWRs触发检测 (fitness=0.9):" in line:
                    swr_triggered = "true" in line.lower()
            
            # 解析最佳技能
            best_gene = None
            for line in output.split("\n"):
                if "最佳技能:" in line:
                    parts = line.split(":")
                    if len(parts) >= 2:
                        best_gene = {"name": parts[1].strip().split(" ")[0]}
            
            return {
                "success": True,
                "gini_gain": gini_gain,
                "swr_triggered": swr_triggered,
                "rust_output": output[-500:]  # 最后500字符
            }
        except subprocess.TimeoutExpired:
            return {"success": False, "error": "Rust subprocess timeout"}
        except Exception as e:
            return {"success": False, "error": str(e)[:100]}


class GiniSelector:
    """
    Gini 基尼增益选择器
    
    功能:
    - Gini 不纯度计算
    - Gini 增益计算
    - 随机森林软投票
    """
    
    @staticmethod
    def gini_impurity(counts: list) -> float:
        """
        Gini = 1 - Σp_k²
        """
        total = sum(counts)
        if total <= 0:
            return 0.0
        proportions = [c / total for c in counts if c > 0]
        return 1.0 - sum(p * p for p in proportions)
    
    @staticmethod
    def gini_gain(parent_counts: list, left_counts: list, right_counts: list) -> float:
        """
        ΔGini = Gini父 - (N_L/N × Gini_L + N_R/N × Gini_R)
        """
        parent_gini = GiniSelector.gini_impurity(parent_counts)
        total = sum(parent_counts)
        if total <= 0:
            return 0.0
        
        left_total = sum(left_counts)
        right_total = sum(right_counts)
        left_weight = left_total / total
        right_weight = right_total / total
        
        return parent_gini - (
            left_weight * GiniSelector.gini_impurity(left_counts) +
            right_weight * GiniSelector.gini_impurity(right_counts)
        )
    
    @staticmethod
    def information_gain(parent_counts: list, child_groups: list) -> float:
        """
        IG = H父 - Σ(N_v/N × H_v)
        """
        def entropy(counts):
            total = sum(counts)
            if total <= 0:
                return 0.0
            proportions = [c / total for c in counts if c > 0]
            return -sum(p * math.log2(p) if p > 0 else 0 for p in proportions)
        
        parent_ent = entropy(parent_counts)
        total = sum(parent_counts)
        if total <= 0:
            return 0.0
        
        weighted_child_ent = 0.0
        for child in child_groups:
            weight = sum(child) / total
            weighted_child_ent += weight * entropy(child)
        
        return parent_ent - weighted_child_ent
    
    @staticmethod
    def soft_vote_probability(gene_predictions: list) -> dict:
        """
        软投票: p_c = (1/B) × Σp_{b,c}(x)
        返回各类概率
        """
        if not gene_predictions:
            return {}
        
        # 简单实现：取平均
        n = len(gene_predictions)
        result = {}
        for key in gene_predictions[0]:
            vals = [g.get(key, 0) for g in gene_predictions]
            result[key] = sum(vals) / n
        return result


class SWRsBuffer:
    """
    SWRs 海马体重放缓冲
    
    机制:
    - 高fitness (>= threshold) 经验进入缓冲
    - 达到一定数量后巩固到长期记忆
    - 低fitness 经验被过滤
    """
    
    def __init__(self, max_size: int = 100, threshold: float = 0.7):
        self.max_size = max_size
        self.threshold = threshold
        self.buffer = []  # list of {"gene_id": str, "fitness": float, "task": str}
    
    def add(self, gene_id: str, fitness: float, task: str = ""):
        """添加经验，高fitness才进入缓冲"""
        if fitness < self.threshold:
            return False  # 被过滤
        
        entry = {"gene_id": gene_id, "fitness": fitness, "task": task}
        self.buffer.append(entry)
        
        # 超出容量，移除最老的
        if len(self.buffer) > self.max_size:
            self.buffer.pop(0)
        
        return True  # 已添加
    
    def swr_triggered(self, fitness: float) -> bool:
        """fitness >= threshold 时触发SWReplay"""
        return fitness >= self.threshold
    
    def consolidate(self) -> list:
        """
        巩固：高fitness经验写入长期记忆
        返回需要巩固的gene_id列表
        """
        # 按fitness降序，取前50%
        mid = len(self.buffer) // 2
        consolidated = []
        if mid > 0:
            sorted_buf = sorted(self.buffer, key=lambda x: x["fitness"], reverse=True)
            to_consolidate = sorted_buf[:mid]
            consolidated = [e["gene_id"] for e in to_consolidate]
            # 清除已巩固的
            self.buffer = sorted_buf[mid:]
        return consolidated
    
    def len(self) -> int:
        return len(self.buffer)


if __name__ == "__main__":
    print("=== APEX EMV 融合测试 ===\n")
    
    # 测试 EMVOrchestrator
    print("1. EMVOrchestrator 测试:")
    orch = EMVOrchestrator()
    result = orch.run("APEX公式代入自检", "测试任务", use_rust=True)
    print(f"   skillbank_len: {result['skillbank_len']}")
    print(f"   gini_gain: {result.get('gini_gain', 'N/A')}")
    print(f"   swr_triggered: {result.get('swr_triggered', 'N/A')}")
    print(f"   best_gene: {result.get('best_gene', {}).get('name', 'N/A')}")
    
    # 测试 GiniSelector
    print("\n2. GiniSelector 测试:")
    gs = GiniSelector()
    gini = gs.gini_impurity([3, 1])  # 3正例1负例
    print(f"   Gini([3,1]): {gini:.4f}")
    gain = gs.gini_gain([4], [3], [1])
    print(f"   ΔGini: {gain:.4f}")
    ig = gs.information_gain([4], [[3], [1]])
    print(f"   IG: {ig:.4f}")
    
    # 测试 SWRsBuffer
    print("\n3. SWRsBuffer 测试:")
    swr = SWRsBuffer(threshold=0.7)
    added1 = swr.add("gene_1", 0.9, "高优先级任务")
    added2 = swr.add("gene_2", 0.5, "低优先级任务")
    print(f"   添加 fitness=0.9: {added1} (应为True)")
    print(f"   添加 fitness=0.5: {added2} (应为False，被过滤)")
    print(f"   swr_triggered(0.9): {swr.swr_triggered(0.9)}")
    print(f"   swr_triggered(0.5): {swr.swr_triggered(0.5)}")
    print(f"   buffer长度: {swr.len()}")
    
    print("\n=== 测试完成 ===")
