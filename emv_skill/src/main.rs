// emv_skill/src/main.rs
// EMV熵Skill CLI - 基因网络选择命令行工具
// 璇玑帝国 APEX · Rust实现
//
// 用法:
//   emv_skill <document> <task>
//   FREEMODEL_API_KEY=xxx emv_skill <document> <task>

use emv_skill::{EMVCycle, GiniSelector, ReplayBuffer, SkillGene};

fn main() {
    println!("=== EMV熵Skill基因网络选择器 ===");
    println!("EMV框架: Challenger出题 → Reasoner解题 → Judge判题");
    println!("选择机制: Gini增益 + 信息熵 + 随机森林投票\n");

    // 从环境变量获取API key
    let api_key = std::env::var("FREEMODEL_API_KEY")
        .or_else(|_| std::env::var("FREEMODEL_API_KEY_BACKUP"))
        .unwrap_or_default();

    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    let (document, task) = if args.len() >= 3 {
        (args[1].as_str(), args[2].as_str())
    } else {
        // 默认文档和任务
        (
            r#"APEX公式代入自检：分析任务前先代入自身能力差距
APEX四要素：Ψ自我、∇梯度、Ξ修复、Γ进化
步骤1代入自己：明确当前任务需要的能力和自己现有能力的差距
步骤2代入公式：用APEX四要素(Ψ∇ΞΓ)照镜子找出当前短板
步骤3举一反三：检查claim是否混淆了事实和推断
步骤4查记忆：memory_search检索MEMORY.md和相关经验
步骤5选择路由：REPAIR修复短板/OPTIMIZE优化流程/EXPLORE探索新方案
APEX主公式：ΔG=(Λ×Θ×K×ξ×Ψ×Φ)/(H×T×ε)
Φ正反馈强化：成功行为增加权重形成正向循环
Ξ修复机制：失败案例触发自我修正和策略调整
Gini增益选择：用基尼不纯度评估技能质量选择最优"#,
            "APEX公式代入自检",
        )
    };

    // 初始化EMV循环
    let mut emv = if api_key.is_empty() {
        println!("⚠️ 无GPT API key，使用简化推理");
        EMVCycle::new()
    } else {
        println!("✅ GPT-5.5 API已接入");
        EMVCycle::new_with_gpt(&api_key)
    };

    let doc_preview: String = document.chars().take(40).collect();
    println!("文档片段: {}...", doc_preview);
    println!("\n任务: {}", task);

    // 尝试加载已有技能库
    let skillbank_path = "/tmp/emv_skillbank.json";
    if let Ok(count) = emv.load_skillbank(skillbank_path) {
        println!("📚 已加载 {} 个已有技能", count);
    }

    // 运行EMV循环
    let (success, best_gene) = emv.run_cycle(document, task);
    println!("结果: success={}, best_gene={}", success, best_gene);

    // 显示所有技能
    println!("\n当前技能库:");
    for (id, gene) in emv.all_genes() {
        println!("  {}: {} (成功率:{:.2}, 增益:{:.3})",
            &id[..8.min(id.len())], gene.name, gene.success_rate(), gene.fitness());
    }

    // 显示最佳技能
    if let Some(best) = emv.best_gene() {
        println!("\n最佳技能: {} (增益:{:.3})", best.name, best.fitness());
    }

    // 测试GiniSelector
    println!("\n=== GiniSelector测试 ===");
    let selector = GiniSelector::new();
    let genes: Vec<SkillGene> = emv.all_genes().values().cloned().collect();
    if !genes.is_empty() {
        let gain = selector.best_split(&genes, "success_rate", 0.5);
        println!("最优分裂增益: {:.4}", gain);
    }

    // 测试ReplayBuffer
    println!("\n=== 跨时间重放测试 ===");
    let mut buffer = ReplayBuffer::new(100);
    buffer.add(emv_skill::ReplayTask {
        task: task.to_string(),
        best_gene_id: best_gene.clone(),
        success: true,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs(),
    });
    println!("重放缓冲: {} 个任务", buffer.len());

    // 保存技能库
    if let Err(e) = emv.save_skillbank(skillbank_path) {
        println!("⚠️ 技能库保存失败: {}", e);
    } else {
        println!("💾 技能库已保存 ({} 个技能)", emv.all_genes().len());
    }

    println!("\n✅ EMV熵Skill运行完成");
}
