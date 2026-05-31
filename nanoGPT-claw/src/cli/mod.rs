

//! CLI Runtime Module - Terminal Control Core
//!
//! Provides command-line interface for NanoGPT-Claw operations.
//! Supports daemon mode, interactive commands, and log querying.

pub mod commands;
pub mod daemon;

use crate::daemon_service::TaskType;
use tracing::{info, warn, error};

/// CLI command types
#[derive(Debug, Clone)]
pub enum CliCommand {
    Start,
    Stop,
    Status,
    Send(String),
    Memory(MemoryCmd),
    Task(TaskCmd),
    Skill(SkillCmd),
    Version,
    Help,
}

/// Memory subcommands
#[derive(Debug, Clone)]
pub enum MemoryCmd {
    Show,
    Clear,
    Purge,
    Stats,
}

/// Task subcommands
#[derive(Debug, Clone)]
pub enum TaskCmd {
    Add(TaskType, String),
    List,
    Get(String),
    Cancel(String),
    Worker,
}

/// Skill subcommands
#[derive(Debug, Clone)]
pub enum SkillCmd {
    List,
    Run(String),
}

/// Parse CLI arguments into command
pub fn parse_args(args: &[String]) -> Option<CliCommand> {
    if args.is_empty() {
        return Some(CliCommand::Help);
    }

    match args[0].as_str() {
        "start" => Some(CliCommand::Start),
        "stop" => Some(CliCommand::Stop),
        "status" => Some(CliCommand::Status),
        "send" => {
            let msg = args.get(1).cloned().unwrap_or_default();
            Some(CliCommand::Send(msg))
        }
        "memory" => {
            let subcmd = args.get(1).map(|s| s.as_str()).unwrap_or("show");
            let mem_cmd = match subcmd {
                "clear" => MemoryCmd::Clear,
                "purge" => MemoryCmd::Purge,
                "stats" => MemoryCmd::Stats,
                _ => MemoryCmd::Show,
            };
            Some(CliCommand::Memory(mem_cmd))
        }
        "task" => parse_task_cmd(&args[1..]),
        "skill" => parse_skill_cmd(&args[1..]),
        "version" => Some(CliCommand::Version),
        "help" => Some(CliCommand::Help),
        _ => {
            warn!("Unknown command: {}", args[0]);
            Some(CliCommand::Help)
        }
    }
}

fn parse_task_cmd(args: &[String]) -> Option<CliCommand> {
    if args.is_empty() {
        return Some(CliCommand::Task(TaskCmd::List));
    }

    match args[0].as_str() {
        "add" => {
            if args.len() < 3 {
                error!("Usage: task add <type> <description>");
                info!("Available types: todo, fix, research, benchmark, github, autoresearch, openhands");
                return Some(CliCommand::Help);
            }
            
            let task_type = match args[1].to_lowercase().as_str() {
                "todo" => TaskType::TodoComplete,
                "fix" => TaskType::CodeFix,
                "research" => TaskType::Research,
                "benchmark" => TaskType::Benchmark,
                "github" => TaskType::GitHubSearch,
                "autoresearch" => TaskType::AutoResearch,
                "openhands" => TaskType::OpenHands,
                _ => {
                    error!("Unknown task type: {}", args[1]);
                    return Some(CliCommand::Help);
                }
            };
            
            let description = args[2..].join(" ");
            Some(CliCommand::Task(TaskCmd::Add(task_type, description)))
        }
        "list" => Some(CliCommand::Task(TaskCmd::List)),
        "get" => {
            let task_id = args.get(1).cloned().unwrap_or_default();
            if task_id.is_empty() {
                error!("Usage: task get <task_id>");
                return Some(CliCommand::Help);
            }
            Some(CliCommand::Task(TaskCmd::Get(task_id)))
        }
        "cancel" => {
            let task_id = args.get(1).cloned().unwrap_or_default();
            if task_id.is_empty() {
                error!("Usage: task cancel <task_id>");
                return Some(CliCommand::Help);
            }
            Some(CliCommand::Task(TaskCmd::Cancel(task_id)))
        }
        "worker" => Some(CliCommand::Task(TaskCmd::Worker)),
        _ => {
            warn!("Unknown task subcommand: {}", args[0]);
            Some(CliCommand::Help)
        }
    }
}

fn parse_skill_cmd(args: &[String]) -> Option<CliCommand> {
    if args.is_empty() {
        return Some(CliCommand::Skill(SkillCmd::List));
    }

    match args[0].as_str() {
        "list" => Some(CliCommand::Skill(SkillCmd::List)),
        "run" => {
            let skill_id = args.get(1).cloned().unwrap_or_default();
            if skill_id.is_empty() {
                error!("Usage: skill run <skill_id>");
                info!("Available skills: cargo-check, cargo-test, cargo-clippy, code-fix, echo, help, status");
                return Some(CliCommand::Help);
            }
            Some(CliCommand::Skill(SkillCmd::Run(skill_id)))
        }
        _ => {
            warn!("Unknown skill subcommand: {}", args[0]);
            Some(CliCommand::Skill(SkillCmd::List))
        }
    }
}

/// Print colored banner
pub fn print_banner() {
    info!("╔══════════════════════════════════════════════════════════════╗");
    info!("║  NanoGPT-Claw v0.9.1 - Lightweight Multi-LLM Agent         ║");
    info!("║  1 Main + N Auxiliary LLM | CoT | Dual Memory | Self-Evol   ║");
    info!("╚══════════════════════════════════════════════════════════════╝");
}
