/// APEX ΔE Scavenger — λΦ 全网知识猎食器
///
/// GitHub 资源猎食 + 论文发现 + 知识吸收 + 冗余剔除
///
/// Usage:
///   scavenge github --list priority
///   scavenge github --list trending
///   scavenge github --clone mem0ai/mem0
///   scavenge papers --query "agent self-improvement" --max 10
///   scavenge absorb --report
///   scavenge diag --report

mod github;
mod papers;
mod absorb;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "scavenge", version = "0.1.0", about = "APEX ΔE Scavenger — λΦ Resource Hunter")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// GitHub 资源猎食
    Github {
        #[command(subcommand)]
        action: GithubAction,
    },
    /// 论文猎食
    Papers {
        #[arg(long, default_value = "agent self-improvement evolution")]
        query: String,
        #[arg(long, default_value_t = 5)]
        max: usize,
    },
    /// 知识吸收
    Absorb {
        #[arg(long)]
        report: bool,
        #[arg(long)]
        dedup: bool,
    },
    /// 诊断报告
    Diag {
        #[arg(long)]
        report: bool,
    },
}

#[derive(Subcommand)]
enum GithubAction {
    /// 猎食优先列表
    Priority,
    /// 猎食趋势列表
    Trending,
    /// 克隆指定 repo
    Clone {
        #[arg()]
        repo: String,
    },
    /// 分析已克隆 repo
    Analyze {
        #[arg()]
        repo: String,
    },
    /// 列出已吸收
    List,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Github { action } => {
            handle_github(action);
        }
        Commands::Papers { query, max } => {
            handle_papers(&query, max);
        }
        Commands::Absorb { report, dedup } => {
            handle_absorb(report, dedup);
        }
        Commands::Diag { report } => {
            handle_diag(report);
        }
    }
}

fn handle_github(action: GithubAction) {
    let cache_dir = PathBuf::from(
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    ).join(".openclaw").join("scavenger-cache");

    let mut scavenger = github::GitHubScavenger::new(cache_dir);

    match action {
        GithubAction::Priority => {
            println!("🔭 猎食优先列表...");
            let results = scavenger.scavenge_priority_list();
            print_github_results(&results, &scavenger);
        }
        GithubAction::Trending => {
            println!("🔥 猎食趋势列表...");
            let results = scavenger.scavenge_trending();
            print_github_results(&results, &scavenger);
        }
        GithubAction::Clone { repo } => {
            let parts: Vec<&str> = repo.split('/').collect();
            if parts.len() < 2 {
                eprintln!("格式: ORG/REPO");
                return;
            }
            match scavenger.scavenge_repo(parts[0], parts[1]) {
                Ok(r) => {
                    println!("✅ 发现: {} (SSH OK)", r.repo);
                    // Clone it
                    for i in 0..scavenger.resources.len() {
                        if scavenger.resources[i].repo == r.repo {
                            match scavenger.clone_for_analysis(i) {
                                Ok(()) => {
                                    println!("✅ 克隆成功");
                                    match scavenger.analyze_clone(i) {
                                        Ok(findings) => {
                                            println!("📋 分析结果:");
                                            for f in findings {
                                                println!("  • {}", f);
                                            }
                                        }
                                        Err(e) => eprintln!("分析失败: {}", e),
                                    }
                                }
                                Err(e) => eprintln!("克隆失败: {}", e),
                            }
                            break;
                        }
                    }
                }
                Err(e) => eprintln!("❌ 猎食失败: {}", e),
            }
        }
        GithubAction::Analyze { repo } => {
            for i in 0..scavenger.resources.len() {
                if scavenger.resources[i].repo == repo || 
                   scavenger.resources[i].repo.ends_with(&repo) {
                    match scavenger.analyze_clone(i) {
                        Ok(findings) => {
                            println!("📋 {} 分析结果:", repo);
                            for f in findings {
                                println!("  • {}", f);
                            }
                        }
                        Err(e) => eprintln!("分析失败: {}", e),
                    }
                    return;
                }
            }
            println!("❌ 未找到 repo: {}", repo);
        }
        GithubAction::List => {
            let absorbed = scavenger.absorbed();
            if absorbed.is_empty() {
                println!("📭 暂无已吸收资源");
            } else {
                println!("📦 已吸收资源:");
                for r in absorbed {
                    println!("  ✅ {}", r.repo);
                }
            }
            let pending = scavenger.pending();
            if !pending.is_empty() {
                println!("\n⏳ 待处理:");
                for r in pending {
                    println!("  ⏳ {}", r.repo);
                }
            }
        }
    }

    // Save state
    let state_path = PathBuf::from(
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    ).join(".openclaw").join("scavenger-state.json");
    if let Err(e) = scavenger.save_state(&state_path) {
        eprintln!("状态保存失败: {}", e);
    }
}

fn handle_papers(query: &str, max: usize) {
    println!("📄 猎食论文: \"{}\" (max {})", query, max);
    let mut searcher = papers::PaperScavenger::new();
    let results = searcher.query_papers(query, max);

    println!("\n📚 发现 {} 篇论文:", results.len());
    for p in &results {
        println!("  [{:.2}] {} ({})", p.relevance_score, p.title, p.published);
    }

    let relevant = searcher.filter_relevant(0.7);
    println!("\n🎯 高相关 (>0.7): {} 篇", relevant.len());
    for p in &relevant {
        let insights = searcher.extract_insights(p);
        println!("  • {}", p.title);
        for i in insights {
            println!("    → {}", i);
        }
    }
}

fn handle_absorb(report: bool, dedup: bool) {
    let mut engine = absorb::AbsorptionEngine::new(0.5, 0.3);

    // Simulate some fragments from GitHub findings
    if dedup {
        let removed = engine.deduplicate();
        println!("🧹 冗余剔除: {} 条", removed);
    }

    if report {
        let r = engine.report();
        println!("📊 吸收统计:");
        println!("  总碎片: {}", r.total_fragments);
        println!("  已吸收: {}", r.absorbed_count);
        println!("  唯一ID: {}", r.unique_ids);
        println!("  质量: avg={:.2} min={:.2} max={:.2}", 
            r.quality_stats.avg, r.quality_stats.min, r.quality_stats.max);
        println!("  标签: {}", r.tags.join(", "));
    }
}

fn handle_diag(report: bool) {
    if report {
        println!("🔍 APEX ΔE Scavenger 诊断");
        println!("  λΦ 猎食器状态:");
        println!("  • GitHub: SSH 通路 (绕过 HTTPS 封锁)");
        println!("  • 论文: 启发式检索 (arXiv)");
        println!("  • 吸收: 质量门 {:.2} + 新异门 {:.2}", 0.5, 0.3);
        println!("  建议: 运行 `scavenge github priority` 开始猎食");
    }
}

fn print_github_results(results: &[Result<github::GitHubResource, String>], s: &github::GitHubScavenger) {
    let mut ok = 0;
    let mut fail = 0;
    for r in results {
        match r {
            Ok(res) => {
                println!("  ✅ {} (SSH: {} | 主题: {})",
                    res.repo,
                    res.last_commit.as_deref().unwrap_or("no commit"),
                    res.topics.join(", ")
                );
                ok += 1;
            }
            Err(e) => {
                println!("  ❌ {}", e);
                fail += 1;
            }
        }
    }
    println!("\n📊 猎食结果: {} 成功, {} 失败", ok, fail);
    println!("⏳ 待处理: {} | ✅ 已吸收: {}", s.pending().len(), s.absorbed().len());
}
