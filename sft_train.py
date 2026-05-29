#!/usr/bin/env python3
"""
SFT训练数据生成器 + 训练启动脚本
从EMV技能库生成训练轨迹数据，准备SFT训练

用法:
  python3 sft_train.py generate    # 生成训练数据
  python3 sft_train.py prepare     # 准备训练配置
  python3 sft_train.py verify      # 验证训练数据
"""

import json
import os
import sys
from datetime import datetime

EMV_SKILLBANK = "/tmp/emv_skillbank.json"
OUTPUT_DIR = "/Users/lihongxin/.openclaw/workspace/a2a-resources/sft_data"
os.makedirs(OUTPUT_DIR, exist_ok=True)


def load_emv_skills():
    """加载EMV技能库"""
    if not os.path.exists(EMV_SKILLBANK):
        print(f"❌ EMV技能库不存在: {EMV_SKILLBANK}")
        return []
    with open(EMV_SKILLBANK) as f:
        return json.load(f)


def generate_trajectory(skill_gene, task, success, generation):
    """从EMV技能生成SFT训练轨迹"""
    trajectory = {
        "trajectory_id": f"emv_traj_{skill_gene['gene_id'][:12]}_{generation}",
        "query": task,
        "skill_id": f"emv_{skill_gene['gene_id'][:12]}",
        "skill_name": skill_gene.get("name", ""),
        "generation": generation,
        "steps": [
            {
                "phase": "Select",
                "input": task,
                "action": "skill_matching",
                "selected_skill": f"emv_{skill_gene['gene_id'][:12]}",
                "match_score": skill_gene.get("success_rate", 0.5),
                "reasoning": f"根据任务'{task}'匹配到技能'{skill_gene.get('name')}'"
            },
            {
                "phase": "Read",
                "input": f"skill: {skill_gene.get('name')}",
                "action": "read_skill_rules",
                "output": skill_gene.get("action", skill_gene.get("description", "")),
                "reasoning": f"读取技能规则: {skill_gene.get('description', '')[:100]}"
            },
            {
                "phase": "Act",
                "input": task,
                "action": "execute_with_skill",
                "results": [
                    f"使用技能 {skill_gene.get('name')} 执行任务",
                    f"触发词: {', '.join(skill_gene.get('trigger_patterns', [])[:3])}"
                ],
                "success": success,
                "reasoning": f"执行技能{'成功' if success else '失败'}，奖励={skill_gene.get('total_reward', 0):.3f}"
            }
        ],
        "final_answer": skill_gene.get("action", ""),
        "success": success,
        "fitness": skill_gene.get("success_rate", 0.5)
    }
    return trajectory


def generate_training_data():
    """生成完整训练数据"""
    skills = load_emv_skills()
    if not skills:
        print("❌ 没有EMV技能可训练")
        return

    print(f"📚 加载了 {len(skills)} 个EMV技能")

    trajectories = []
    for gene in skills:
        # 为每个技能生成多个训练样本
        task = gene.get("description", gene.get("name", ""))
        if not task:
            continue

        success_rate = gene.get("success_rate", 0.5)
        success = success_rate >= 0.5

        traj = generate_trajectory(gene, task, success, gene.get("generation", 0))
        trajectories.append(traj)

        # 额外生成一些困难样本
        if success_rate < 0.5:
            traj_fail = generate_trajectory(gene, f"困难任务: {task}", False, gene.get("generation", 0))
            trajectories.append(traj_fail)

    # 保存训练数据
    output_file = os.path.join(OUTPUT_DIR, "emv_trajectories.jsonl")
    with open(output_file, "w") as f:
        for traj in trajectories:
            f.write(json.dumps(traj, ensure_ascii=False) + "\n")

    print(f"✅ 生成 {len(trajectories)} 条训练轨迹 → {output_file}")

    # 生成统计
    stats = {
        "total": len(trajectories),
        "successful": sum(1 for t in trajectories if t["success"]),
        "failed": sum(1 for t in trajectories if not t["success"]),
        "skills_used": len(set(t.get("skill_id") for t in trajectories)),
        "avg_fitness": sum(t.get("fitness", 0) for t in trajectories) / len(trajectories) if trajectories else 0
    }

    stats_file = os.path.join(OUTPUT_DIR, "training_stats.json")
    with open(stats_file, "w") as f:
        json.dump(stats, f, indent=2)

    print(f"📊 统计: 成功率={stats['successful']}/{stats['total']}, "
          f"技能数={stats['skills_used']}, "
          f"平均fitness={stats['avg_fitness']:.3f}")

    return stats


def prepare_training_config():
    """准备训练配置文件"""
    config = {
        "model_name": "gpt-5",
        "training_type": "SFT_two_phase",
        "phase1": {
            "name": "trajectory_pretraining",
            "data_file": "emv_trajectories.jsonl",
            "epochs": 3,
            "lr": 1e-5,
            "batch_size": 4,
            "max_seq_len": 2048
        },
        "phase2": {
            "name": "skill_conditioned_finetuning",
            "data_file": "emv_trajectories.jsonl",
            "epochs": 5,
            "lr": 5e-6,
            "batch_size": 2,
            "max_seq_len": 4096,
            "skill_labels": True
        },
        "output_dir": OUTPUT_DIR,
        "wandb_project": "apex-sft"
    }

    config_file = os.path.join(OUTPUT_DIR, "sft_config.json")
    with open(config_file, "w") as f:
        json.dump(config, f, indent=2)

    print(f"✅ 训练配置已保存 → {config_file}")
    print("\n训练命令:")
    print(f"  # Phase 1: 轨迹预训练")
    print(f"  python3 train.py --config {config_file} --phase 1")
    print(f"  # Phase 2: 技能条件微调")
    print(f"  python3 train.py --config {config_file} --phase 2")


def verify_data():
    """验证训练数据质量"""
    output_file = os.path.join(OUTPUT_DIR, "emv_trajectories.jsonl")
    if not os.path.exists(output_file):
        print(f"❌ 训练数据不存在: {output_file}")
        return False

    with open(output_file) as f:
        lines = f.readlines()

    print(f"📂 训练数据验证: {len(lines)} 条轨迹")

    valid = 0
    for i, line in enumerate(lines[:5]):
        try:
            traj = json.loads(line)
            required = ["trajectory_id", "query", "skill_id", "steps", "success"]
            if all(k in traj for k in required):
                valid += 1
                print(f"  ✅ [{i+1}] {traj['trajectory_id']}: {traj['query'][:40]}...")
            else:
                print(f"  ❌ [{i+1}] 缺少字段")
        except:
            print(f"  ❌ [{i+1}] JSON解析失败")

    print(f"\n📊 前5条中 {valid} 条有效")
    return valid > 0


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "generate"

    if cmd == "generate":
        generate_training_data()
    elif cmd == "prepare":
        prepare_training_config()
    elif cmd == "verify":
        verify_data()
    else:
        print(f"未知命令: {cmd}")
        print("用法: sft_train.py [generate|prepare|verify]")
