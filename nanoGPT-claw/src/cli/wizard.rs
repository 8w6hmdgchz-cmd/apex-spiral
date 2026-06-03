use std::io::{self, Write};
use std::path::PathBuf;
use anyhow::Result;
use crate::config::get_config_dir;

#[derive(Default)]
pub struct ConfigWizard {
    #[allow(dead_code)]
    config_dir: PathBuf,
}

impl ConfigWizard {
    pub fn new() -> Self {
        Self {
            config_dir: get_config_dir(),
        }
    }

    pub fn run(&mut self) -> Result<bool> {
        self.print_banner();
        println!();
        self.step_directories()?;
        self.step_env_template()?;
        self.step_summary()?;
        self.save_config()
    }

    fn print_banner(&self) {
        println!();
        println!("\x1b[36m╔══════════════════════════════════════════════════════════════════════╗\x1b[0m");
        println!("\x1b[36m║          NanoGPT-Claw 首次配置向导 v0.9.1                           ║\x1b[0m");
        println!("\x1b[36m║  1 Main + N Auxiliary LLM | CoT | Dual Memory | Self-Evolution      ║\x1b[0m");
        println!("\x1b[36m╚══════════════════════════════════════════════════════════════════════╝\x1b[0m");
        println!();
        println!("\x1b[33m欢迎使用 NanoGPT-Claw！\x1b[0m");
        println!();
    }

    fn step_directories(&self) -> Result<()> {
        println!("\x1b[1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
        println!("\x1b[1m📁 创建必要目录\x1b[0m");
        println!("\x1b[1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
        println!();

        let data_dir = PathBuf::from("./data");
        let logs_dir = PathBuf::from("./logs");
        let skills_dir = PathBuf::from("./skills");

        for dir in [&self.config_dir, &data_dir, &logs_dir, &skills_dir] {
            std::fs::create_dir_all(dir)?;
            println!("\x1b[32m✅\x1b[0m {}", dir.display());
        }

        Ok(())
    }

    fn step_env_template(&mut self) -> Result<()> {
        println!();
        println!("\x1b[1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
        println!("\x1b[1m🔐 环境变量配置\x1b[0m");
        println!("\x1b[1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
        println!();

        println!("\x1b[33m敏感配置（API Keys, Tokens）通过环境变量读取，不写入文件。\x1b[0m");
        println!();

        let env_template = crate::config::EnvConfig::export_template();
        let env_file = self.config_dir.join(".env.template");

        std::fs::write(&env_file, &env_template)?;
        println!("\x1b[32m✅\x1b[0m 环境变量模板已保存: {}", env_file.display());
        println!();

        println!("\x1b[36m请复制模板并配置你的密钥：\x1b[0m");
        println!();
        println!("  \x1b[32m$ cp {} ~/.config/nanoGPT-claw/.env\x1b[0m", env_file.file_name().unwrap().to_string_lossy());
        println!("  \x1b[32m$ nano ~/.config/nanoGPT-claw/.env\x1b[0m  # 编辑并填入你的配置");
        println!();

        let use_example = self.ask_yes_no("是否查看环境变量模板内容?", false);
        if use_example {
            println!();
            for line in env_template.lines().take(30) {
                println!("  {}", line);
            }
            if env_template.lines().count() > 30 {
                println!("  ... (共 {} 行)", env_template.lines().count());
            }
        }

        Ok(())
    }

    fn step_summary(&self) -> Result<()> {
        println!();
        println!("\x1b[1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
        println!("\x1b[1m📊 配置摘要\x1b[0m");
        println!("\x1b[1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
        println!();

        println!("\x1b[36m配置目录:\x1b[0m {}", self.config_dir.display());
        println!("\x1b[36m数据目录:\x1b[0m ./data/");
        println!("\x1b[36m日志目录:\x1b[0m ./logs/");
        println!();

        println!("\x1b[33m⚠️  重要：设置环境变量后再启动服务\x1b[0m");
        println!();
        println!("  方式 1: 手动导出");
        println!("    \x1b[32m$ export OPENAI_API_KEY=sk-xxx\x1b[0m");
        println!("    \x1b[32m$ export FEISHU_APP_ID=cli_xxx\x1b[0m");
        println!();
        println!("  方式 2: 使用 .env 文件");
        println!("    \x1b[32m$ export $(cat ~/.config/nanoGPT-claw/.env | xargs)\x1b[0m");
        println!();
        println!("  方式 3: 在 shell 配置文件 (~/.bashrc, ~/.zshrc) 中添加");
        println!();

        Ok(())
    }

    fn save_config(&mut self) -> Result<bool> {
        println!();
        println!("\x1b[1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
        println!("\x1b[1m🚀 初始化完成\x1b[0m");
        println!("\x1b[1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
        println!();

        let has_env = crate::config::EnvConfig::load().has_any_llm_key();

        if has_env {
            println!("\x1b[32m✅ 检测到已配置的环境变量，可以直接启动！\x1b[0m");
        } else {
            println!("\x1b[90m⏳ 未检测到环境变量，请先配置。\x1b[0m");
        }
        println!();

        let auto_start = self.ask_yes_no("是否立即启动 Daemon?", false);

        Ok(auto_start)
    }

    fn ask_yes_no(&self, prompt: &str, default: bool) -> bool {
        let hint = if default { "[Y/n]" } else { "[y/N]" };
        print!("\x1b[36m❯\x1b[0m {} {}: ", prompt, hint);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input.is_empty() {
            default
        } else {
            input == "y" || input == "yes" || input == "是"
        }
    }
}

pub fn check_first_run() -> bool {
    let config_path = get_config_dir().join("config.yaml");
    !config_path.exists()
}

pub fn run_wizard() -> Result<bool> {
    let mut wizard = ConfigWizard::new();
    wizard.run()
}
