/// APEX ΔE Engine — Main Entry Point
///
/// CLI interface for the APEX evolution system.
///
/// Usage:
///   apexe calc --alpha 85 --beta 60 --lambda 40 --nabla 25 --evol 30
///   apexe evolve --state /path/to/state.json
///   apexe diagnose --state /path/to/state.json
///   apexe directive --state /path/to/state.json
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
}

fn main() {
    let cli = Cli::parse();

    // Detect whether --help was used without error
    match cli.command {
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
