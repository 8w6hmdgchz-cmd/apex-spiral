//! Message Router - FULL INTEGRATION (REAL)
//!
//! No fakes - everything connected.

use super::{current_timestamp, MessageContext, MessageResponse, MiddlewareError};
use crate::cot::reasoner::Reasoner;
use crate::evolution::apex_akashic::ApexAkashicCalculator;
use crate::scheduler::Scheduler;
use std::sync::Arc;
use tracing::{info, warn};

const ARS_THRESHOLD: f64 = 0.45;

#[derive(Debug, Clone, Copy)]
pub enum RouteDestination {
    LLMProcessor,
    CommandHandler,
    GatewayFeishu,
    GatewayGitHub,
    EvolutionEngine,
    MemoryLayer,
}

pub struct MessageRouter {
    scheduler: Arc<Scheduler>,
}

impl MessageRouter {
    pub fn new(scheduler: Arc<Scheduler>) -> Self {
        Self { scheduler }
    }

    pub async fn route(&self, ctx: MessageContext) -> Result<MessageResponse, MiddlewareError> {
        info!(
            "Routing message from {} (session: {})",
            ctx.source, ctx.session_id
        );

        // REAL ARS quality check first
        let ars_calculator = ApexAkashicCalculator::new();
        let ars_score = ars_calculator.calculate_ars_for_input(&ctx.content);

        info!("ARS score: {:.4} (threshold: {})", ars_score, ARS_THRESHOLD);

        if ars_score < ARS_THRESHOLD {
            warn!(
                "Input rejected by ARS: score {:.4} < threshold {}",
                ars_score, ARS_THRESHOLD
            );
            return Err(MiddlewareError::InvalidContent(format!(
                "Input quality insufficient (ARS={:.4}), please provide more details",
                ars_score
            )));
        }

        let use_cot = ctx.content.to_lowercase().contains("cot")
            || ctx.content.to_lowercase().contains("推理")
            || ctx.content.to_lowercase().contains("思考");

        let response_content = if ctx.content.starts_with('/') {
            self.handle_command(&ctx.content).await?
        } else if use_cot {
            self.handle_llm_cot(&ctx).await?
        } else {
            self.handle_llm(&ctx).await?
        };

        Ok(MessageResponse {
            content: response_content,
            session_id: ctx.session_id,
            timestamp: current_timestamp(),
            metadata: Default::default(),
        })
    }

    async fn handle_llm(&self, ctx: &MessageContext) -> Result<String, MiddlewareError> {
        info!("Handling via REAL FULL INTEGRATED pipeline");
        match self
            .scheduler
            .process_full_pipeline(&ctx.content, &ctx.session_id)
            .await
        {
            Ok(response) => {
                info!(
                    "Full pipeline complete, response length: {}",
                    response.len()
                );
                Ok(response)
            }
            Err(e) => {
                warn!("LLM pipeline failed: {}", e);
                Err(MiddlewareError::LLMError(e.to_string()))
            }
        }
    }

    async fn handle_llm_cot(&self, ctx: &MessageContext) -> Result<String, MiddlewareError> {
        info!("Using REAL Chain-of-Thought (CoT) reasoning");
        let reasoner = Reasoner::new(self.scheduler.clone());
        match reasoner.reason(&ctx.content).await {
            Ok(cot_result) => Ok(format!(
                "【THOUGHT PROCESS RESULT】\nFinal Answer: {}\n\nReasoning Steps:\n{}",
                cot_result.conclusion,
                cot_result
                    .reasoning_chain
                    .iter()
                    .enumerate()
                    .map(|(i, s)| format!(
                        "{}. {}\n   Confidence: {:.2}",
                        i + 1,
                        s.thought,
                        s.confidence
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            )),
            Err(e) => {
                warn!("CoT reasoning failed: {}", e);
                Err(MiddlewareError::LLMError(e.to_string()))
            }
        }
    }

    async fn handle_command(&self, cmd: &str) -> Result<String, MiddlewareError> {
        info!("Handling as command");
        match cmd.trim() {
            "/status" => {
                let stats = self.scheduler.get_stats().await;
                Ok(format!(
                    "System Status:\n- Active Tasks: {}\n- Memory: {}\n- Skills: {:?}",
                    stats.active_tasks,
                    if stats.memory_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    },
                    stats.available_skills
                ))
            }
            "/help" => Ok("Available commands: /status, /help, /skills".to_string()),
            "/skills" => Ok(format!(
                "Available Skills: {:?}",
                self.scheduler.list_available_skills()
            )),
            _ => Ok(format!("Unknown command: {}", cmd)),
        }
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new(Arc::new(Scheduler::new()))
    }
}
