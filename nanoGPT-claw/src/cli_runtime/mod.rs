pub mod terminal;
pub mod commands;

use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use std::io::Write;
use tracing::{error, info};

use crate::core_scheduler::LLM_SCHEDULER;
use crate::memory_layer::MEMORY_STORAGE;

pub static CLI_RUNTIME: once_cell::sync::Lazy<Arc<RwLock<CliRuntime>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(CliRuntime::new().expect("Failed to create CLI runtime"))));

pub struct CliRuntime {
    running: bool,
    session_id: String,
    command_count: u64,
    start_time: Instant,
}

impl CliRuntime {
    pub fn new() -> Result<Self> {
        Ok(Self {
            running: false,
            session_id: uuid::Uuid::new_v4().to_string(),
            command_count: 0,
            start_time: Instant::now(),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        info!("Starting CLI runtime...");
        self.running = true;
        self.start_time = Instant::now();
        info!("Session ID: {}", self.session_id);
        Ok(())
    }

    pub fn stop(&mut self) {
        info!("Stopping CLI runtime...");
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub async fn run_loop(&mut self) -> Result<()> {
        self.start()?;
        info!("CLI running in interactive mode");
        
        println!("\n=== NanoGPT-Claw Interactive CLI ===");
        println!("Type 'help' for available commands, 'quit' to exit\n");
        
        let mut input = String::new();
        
        loop {
            print!("nanogpt> ");
            if let Err(e) = std::io::stdout().flush() {
                error!("Failed to flush stdout: {}", e);
            }
            
            input.clear();
            match std::io::stdin().read_line(&mut input) {
                Ok(0) => {
                    println!("\nEOF received, exiting...");
                    break;
                }
                Ok(_) => {
                    let input = input.trim();
                    if input.is_empty() {
                        continue;
                    }
                    
                    self.command_count += 1;
                    let response = self.process_command(input).await;
                    
                    match response {
                        Ok(output) => {
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                        }
                        Err(e) => {
                            error!("Command error: {}", e);
                        }
                    }
                    
                    if input == "quit" || input == "exit" {
                        break;
                    }
                }
                Err(e) => {
                    error!("Read error: {}", e);
                    break;
                }
            }
        }
        
        self.stop();
        Ok(())
    }

    async fn process_command(&self, input: &str) -> Result<String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return Ok(String::new());
        }
        
        match parts[0].to_lowercase().as_str() {
            "help" => Ok(self.show_help()),
            "status" => Ok(self.show_status()),
            "ping" => Ok("Pong!".to_string()),
            "stats" => Ok(self.show_stats()),
            "clear" => {
                print!("\x1b[2J\x1b[H");
                Ok(String::new())
            }
            "quit" | "exit" => {
                Ok("Goodbye!".to_string())
            }
            "think" => {
                let query = parts[1..].join(" ");
                Ok(format!("[Thinking] Processing: {}", query))
            }
            "memory" => {
                let stats = MEMORY_STORAGE.get_stats();
                Ok(format!("Memory Stats: {} entries (short: {}, long: {})", 
                    stats.total_entries, stats.short_term_count, stats.long_term_count))
            }
            "version" | "--version" | "-v" => {
                Ok(format!("NanoGPT-Claw v{}", env!("CARGO_PKG_VERSION")))
            }
            _ => {
                Ok(format!("Unknown command: '{}'. Type 'help' for available commands.", parts[0]))
            }
        }
    }

    fn show_help(&self) -> String {
        r#"
Available Commands:
  help              Show this help message
  status            Show system status
  stats             Show detailed statistics
  ping              Test connectivity
  memory            Show memory statistics
  think <query>     Process a query with CoT reasoning
  clear             Clear the screen
  version, -v       Show version
  quit, exit        Exit the CLI
"#.to_string()
    }

    fn show_status(&self) -> String {
        let uptime = Instant::now().duration_since(self.start_time);
        format!(
            "NanoGPT-Claw Status\n  Running: {}\n  Uptime: {:.1}s\n  Session: {}\n  Commands: {}",
            if self.running { "Yes" } else { "No" },
            uptime.as_secs_f64(),
            &self.session_id[..8],
            self.command_count
        )
    }

    fn show_stats(&self) -> String {
        let scheduler_stats = LLM_SCHEDULER.get_stats();
        let memory_stats = MEMORY_STORAGE.get_stats();
        let uptime = Instant::now().duration_since(self.start_time);
        
        format!(
            "Statistics:\n  Uptime: {:.1}s\n  LLM Requests: {} ({} successful, {} failed)\n  Memory: {} entries\n  CLI Commands: {}",
            uptime.as_secs_f64(),
            scheduler_stats.total_requests,
            scheduler_stats.successful_requests,
            scheduler_stats.failed_requests,
            memory_stats.total_entries,
            self.command_count
        )
    }
}
