//! APEX Token Optimizer CLI
use apex_token_rs::{SlidingContext, FrameSummary, TargetBrief, PromptBuilder, Sanitizer, image_to_screen, Size, Rect, Point, BudgetConfig};
use serde_json;

fn main() {
    println!("=== APEX Token Optimizer ===\n");

    // 1. 坐标校正测试
    println!("[1] 坐标校正测试");
    let p = Point { x: 100.0, y: 100.0 };
    let img = Size { w: 1920.0, h: 1080.0 };
    let roi = Rect { x: 0.0, y: 0.0, w: 1920.0, h: 1080.0 };
    let screen = image_to_screen(p, img, roi);
    println!("  输入: ({}, {})", p.x, p.y);
    println!("  输出: ({}, {})", screen.x, screen.y);

    // 2. 滑动窗口测试
    println!("\n[2] 滑动窗口测试 (max=3)");
    let mut ctx = SlidingContext::new(3);
    for i in 0..5 {
        ctx.push(FrameSummary {
            frame_id: i,
            timestamp_ms: i as i64 * 100,
            targets: vec![],
            player_state: format!("state_{}", i),
            event_flags: vec![],
        });
    }
    println!("  帧数: {} (预期3)", ctx.len());
    let snap = ctx.snapshot();
    println!("  帧ID: {:?}", snap.iter().map(|f| f.frame_id).collect::<Vec<_>>());

    // 3. Token预算测试
    println!("\n[3] Token预算测试");
    let base = "你是APEX助手".to_string();
    let builder = PromptBuilder::new(base).with_budget(BudgetConfig {
        soft_cap: 200,
        hard_cap: 300,
    });

    let frames = vec![
        FrameSummary {
            frame_id: 1,
            timestamp_ms: 100,
            targets: vec![
                TargetBrief { id: 1, cls: "enemy".to_string(), x: 100, y: 200, vx: 1, vy: -1, conf: 85 },
                TargetBrief { id: 2, cls: "loot".to_string(), x: 300, y: 400, vx: 0, vy: 0, conf: 72 },
            ],
            player_state: "fighting".to_string(),
            event_flags: vec!["shot".to_string(), "damage".to_string()],
        },
    ];

    let (prompt, tokens) = builder.build_with_budget(&frames);
    println!("  Token数: {}", tokens);
    println!("  Prompt:\n{}", prompt);

    // 4. 25步净化测试
    println!("\n[4] 25步净化测试");
    let frame = FrameSummary {
        frame_id: 1,
        timestamp_ms: 100,
        targets: vec![
            TargetBrief { id: 1, cls: "enemy".to_string(), x: 100, y: 200, vx: 1, vy: -1, conf: 30 }, // 低置信度
            TargetBrief { id: 2, cls: "enemy".to_string(), x: 105, y: 205, vx: 1, vy: -1, conf: 85 }, // 重复目标
        ],
        player_state: "fighting".to_string(),
        event_flags: vec!["shot".to_string(), "shot".to_string()], // 重复
    };

    let cleaned = Sanitizer::sanitize(frame.clone());
    println!("  原始目标数: 2, 净化后: {}", cleaned.targets.len());
    println!("  原始事件: {:?}, 净化后: {:?}", frame.event_flags, cleaned.event_flags);

    println!("\n=== 完成 ===");
}
