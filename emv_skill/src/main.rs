// emv_skill/src/main.rs
// EMV熵Skill CLI - 基因网络选择命令行工具
// 璇玑帝国 APEX · Rust实现

use emv_skill::{EMVCycle, GiniSelector, ReplayBuffer, SkillGene};

fn main() {
    println!("=== EMV熵Skill基因网络选择器 ===");
    println!("EMV框架: Challenger出题 → Reasoner解题 → Judge判题");
    println!("选择机制: Gini增益 + 信息熵 + 随机森林投票\n");

    // 初始化EMV循环
    let mut emv = EMVCycle::new();

    // 示例文档
    let document = r#"
    APEX公式代入自检流程：
    步骤1: 代入自己 - 分析任务需要的能力和自己的差距
    步骤2: 代入公式 - 用APEX四要素(Ψ∇ΞΓ)照镜子
    步骤3: 举一反三 - 检查claim是否混淆
    步骤4: 查记忆 - memory_search检索
    步骤5: 选择路由 - REPAIR/OPTIMIZE/EXPLORE/INNOVATE/CURATE
    "#;

    let doc_preview: String = document.chars().take(30).collect();
    println!("文档片段: {}...", doc_preview);

    // 运行EMV循环
    let task = "APEX公式代入自检";
    println!("\n任务: {}", task);

    let (success, best_gene) = emv.run_cycle(document, task);
    println!("结果: success={}, best_gene={}", success, best_gene);

    // 显示所有技能
    println!("\n当前技能库:");
    for (id, gene) in emv.all_genes() {
        println!("  {}: {} (成功率:{:.2}, 增益:{:.3})",
            id, gene.name, gene.success_rate(), gene.fitness());
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
        task: "APEX自检".to_string(),
        best_gene_id: best_gene.clone(),
        success: true,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs(),
    });
    println!("重放缓冲: {} 个任务", buffer.len());

    println!("\n✅ EMV熵Skill运行完成");
}
