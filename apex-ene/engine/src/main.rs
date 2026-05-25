/// APEX ΔE Engine — Main Entry Point
///
/// CLI interface for the APEX evolution system.
///
/// Usage:
///   apexe calc --alpha 85 --beta 60 --lambda 40 --nabla 25 --evol 30
///   apexe calc-v10 --json --safe
///   apexe evolve --state /path/to/state.json
///   apexe diagnose --state /path/to/state.json
///   apexe directive --state /path/to/state.json
///   apexe analyze --path /path/to/workspace
///   apexe self-mod --path /path/to/workspace --target relative/file --before old --after new
///   apexe clone --repo git@github.com:ORG/REPO.git

mod apexe;
mod evolution;
mod selfmod;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "apexe", version = "0.1.0", about = "APEX ΔE Evolution Engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Calculate APEX ΔE score
    Calc {
        #[arg(long, default_value = "85.0")]
        alpha: f64,
        #[arg(long, default_value = "60.0")]
        beta: f64,
        #[arg(long, default_value = "40.0")]
        lambda: f64,
        #[arg(long, default_value = "25.0")]
        nabla: f64,
        #[arg(long, default_value = "30.0")]
        evol: f64,
        #[arg(long)]
        state: Option<PathBuf>,
    },
    /// Calculate APEX V10.1 ΔG score
    CalcV10 {
        #[arg(long, default_value_t = false)]
        json: bool,
        #[arg(long, default_value_t = false)]
        safe: bool,
    },
    /// Run one evolution cycle
    Evolve {
        #[arg(long, default_value = ".apex-state.json")]
        state: PathBuf,
    },
    /// Diagnose system state
    Diagnose {
        #[arg(long, default_value = ".apex-state.json")]
        state: PathBuf,
    },
    /// Get evolution directive
    Directive {
        #[arg(long, default_value = ".apex-state.json")]
        state: PathBuf,
    },
    /// Analyze codebase
    Analyze {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Generate/apply/verify a guarded self-modification patch
    SelfMod {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        target: String,
        #[arg(long)]
        before: String,
        #[arg(long)]
        after: String,
        #[arg(long, default_value = "manual self-mod CLI patch")]
        directive: String,
        #[arg(long, default_value_t = false)]
        no_apply: bool,
        #[arg(long, default_value_t = false)]
        rollback: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // Detect whether --help was used without error
    match cli.command {
        Commands::CalcV10 { json, safe } => {
            use apex_v10_core::*;

            let params = ApexParamsV8 {
                lambda_root: 0.95,
                xi_anti_hallucination: 1.0,
                h_real: 0.5,
                t_iteration: 2.0,
                llm_agent: LlmAgentParams {
                    lambda_single_call: 0.9,
                    mu_multi_task: 0.85,
                    sigma_high_quality: 0.88,
                    gamma_llm_cost: 0.1,
                },
                master: MasterParams {
                    k_code: 1.0,
                    tau_transfer: vec![0.1, 0.05, 0.08],
                    upsilon_apply: 0.9,
                },
                self_repair: SelfRepairParams {
                    g_target: 100.0,
                    g_actual: 95.0,
                    delta_error_locate: 1.0,
                    psi_thorough_fix: 1.0,
                    kappa_no_repeat: 1.0,
                },
                cycle: CycleParams {
                    eta_skill_up: 1.0,
                    rho_result_feedback: 0.6,
                },
                host: HostHealthParams {
                    psi_mem: 0.98,
                    psi_app: 0.99,
                    psi_disk: 0.97,
                    omega_dawn: 1.0,
                },
            };

            if safe {
                match calculate_delta_g_ultimate_safe(&params) {
                    Ok(dg) => {
                        if json {
                            let result = calculate_v10_full(&params, None);
                            let mut output = serde_json::to_value(&result).unwrap();
                            output["mode"] = serde_json::Value::String("safe".into());
                            println!("{}", serde_json::to_string_pretty(&output).unwrap());
                        } else {
                            println!("ΔG (V10.1 safe) = {:.6}", dg);
                            println!("evolution_score = {:.6}", evolution_score(dg, params.h_real));
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            } else if json {
                let result = calculate_v10_full(&params, None);
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                match calculate_delta_g_ultimate(&params) {
                    Ok(dg) => {
                        println!("ΔG (V10.1) = {:.6}", dg);
                        println!("evolution_score = {:.6}", evolution_score(dg, params.h_real));
                        println!("theta_llm = {:.6}", calculate_llm_agent_efficiency(&params.llm_agent));
                        println!("k_master = {:.6}", calculate_k_master(&params.master));
                        println!("epsilon = {:.6}", calculate_self_repair(&params.self_repair));
                        println!("phi_cycle = {:.6}", calculate_cycle_gain(&params.cycle));
                        println!("psi_host = {:.6}", calculate_host_health(&params.host));
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        Commands::Calc { alpha, beta, lambda, nabla, evol, state } => {
            let dims = apexe::ApexDimensions::new(alpha, beta, lambda, nabla, evol);

            let previous_total = if let Some(path) = &state {
                if path.exists() {
                    let content = std::fs::read_to_string(path).unwrap_or_default();
                    let state: evolution::EvolutionState =
                        serde_json::from_str(&content).unwrap_or_else(|_| evolution::EvolutionState::new());
                    state.history.last().map(|h| h.total).unwrap_or(0.0)
                } else { 0.0 }
            } else { 0.0 };

            let result = apexe::ApexDeltaE::new(dims, 0, previous_total);
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }

        Commands::Evolve { state } => {
            let mut controller = evolution::EvolutionController::new(state.clone());

            // Get current state from history or defaults
            let current = controller.state.history.last()
                .map(|h| h.dimensions.clone())
                .unwrap_or_else(|| apexe::ApexDimensions::new(50.0, 50.0, 50.0, 50.0, 50.0));

            // Run cycle with slight random perturbation for exploration
            let perturbed = apexe::ApexDimensions::new(
                current.alpha_psi + (rand() * 10.0 - 5.0),
                current.beta_omega + (rand() * 10.0 - 5.0),
                current.lambda_phi + (rand() * 10.0 - 5.0),
                current.nabla_theta + (rand() * 5.0 - 2.5),
                current.evol_code + (rand() * 8.0 - 4.0),
            );

            let result = controller.run_cycle(perturbed);
            let directive = controller.get_directive(&result.dimensions);

            let output = serde_json::json!({
                "apex_delta_e": result,
                "directive": directive,
                "cycle_hours": controller.state.current_cycle_hours,
                "next_run": controller.state.next_run,
                "success_rate": format!("{:.1}%", controller.state.success_rate(20)),
            });

            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }

        Commands::Diagnose { state } => {
            let content = std::fs::read_to_string(&state).unwrap_or_default();
            let state: evolution::EvolutionState =
                serde_json::from_str(&content).unwrap_or_else(|_| evolution::EvolutionState::new());

            let mut diag = serde_json::Map::new();
            diag.insert("version".into(), serde_json::Value::Number(state.version.into()));
            diag.insert("total_runs".into(), serde_json::Value::Number(state.total_runs.into()));
            diag.insert("success_rate".into(), format!("{:.1}%", state.success_rate(50)).into());
            diag.insert("cycle_hours".into(), state.current_cycle_hours.into());
            diag.insert("bottleneck_focus".into(), state.bottleneck_focus.clone().into());

            if let Some(last) = state.history.last() {
                diag.insert("last_apex_de".into(), last.total.into());
                diag.insert("dimensions".into(), serde_json::json!({
                    "αΨ": last.dimensions.alpha_psi,
                    "βΩ": last.dimensions.beta_omega,
                    "λΦ": last.dimensions.lambda_phi,
                    "∇Θ": last.dimensions.nabla_theta,
                    "Evol_code": last.dimensions.evol_code,
                }));
                diag.insert("issues".into(), serde_json::json!(last.diagnosis()));
            }

            println!("{}", serde_json::to_string_pretty(&diag).unwrap());
        }

        Commands::Directive { state } => {
            let content = std::fs::read_to_string(&state).unwrap_or_default();
            let mut st: evolution::EvolutionState =
                serde_json::from_str(&content).unwrap_or_else(|_| evolution::EvolutionState::new());

            let dimensions = st.history.last()
                .map(|h| h.dimensions.clone())
                .unwrap_or_else(|| apexe::ApexDimensions::new(50.0, 50.0, 50.0, 50.0, 50.0));

            let directive = st.evolution_directive(&dimensions);
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "bottleneck": st.bottleneck_focus,
                "directive": directive,
                "next_cycle_hours": st.current_cycle_hours,
            })).unwrap());
        }

        Commands::Analyze { path } => {
            let engine = selfmod::SelfModEngine::new(path);
            let issues = engine.analyze_codebase();
            let diag = engine.code_diagnosis();

            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "issues": issues,
                "diagnosis": diag,
            })).unwrap());
        }

        Commands::SelfMod { path, target, before, after, directive, no_apply, rollback } => {
            let mut engine = selfmod::SelfModEngine::new(path);
            let patch = engine.generate_patch(
                &target,
                selfmod::PatchType::BugFix,
                &before,
                &after,
                &directive,
            );

            let mut steps = Vec::new();
            steps.push(format!("generated {} for {}", patch.id, patch.target_file));

            if !no_apply {
                match engine.apply_patch(&patch.id) {
                    Ok(msg) => steps.push(msg),
                    Err(err) => {
                        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                            "patch": patch,
                            "steps": steps,
                            "error": err,
                        })).unwrap());
                        std::process::exit(1);
                    }
                }

                match engine.verify_patch(&patch.id) {
                    Ok(msg) => steps.push(msg),
                    Err(err) => {
                        steps.push(err.clone());
                        if rollback {
                            match engine.rollback_patch(&patch.id) {
                                Ok(msg) => steps.push(msg),
                                Err(rollback_err) => steps.push(format!("rollback failed: {}", rollback_err)),
                            }
                        }
                        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                            "patch": patch,
                            "steps": steps,
                            "error": err,
                        })).unwrap());
                        std::process::exit(1);
                    }
                }
            }

            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "patch": patch,
                "steps": steps,
            })).unwrap());
        }
    }
}

/// Simple deterministic random for perturbation
fn rand() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as f64;
    (nanos / 1_000_000_000.0) * 20.0 - 10.0
}
