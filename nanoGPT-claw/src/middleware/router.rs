//! Message Router - Smart Message Routing Logic
//!
//! Routes messages to appropriate handlers based on source,
//! content type, and system state.

use super::{MessageContext, MessageResponse, MiddlewareError, current_timestamp};
use std::sync::Arc;
use tracing::{info, warn};
use crate::scheduler::Scheduler;
use crate::cot::reasoner::Reasoner;

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
        info!("Routing message from {} (session: {})", ctx.source, ctx.session_id);

        let use_cot = ctx.content.to_lowercase().contains("cot") || 
                      ctx.content.to_lowercase().contains("推理") || 
                      ctx.content.to_lowercase().contains("思考");

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
        info!("Handling via LLM processor...");
        let prompt = format!("你是NanoGPT-Claw AI助手。用户在{}终端发送了：{}

请给出有帮助的回答：", ctx.source, ctx.content);
        match self.scheduler.submit_to_main(&prompt).await {
            Ok(response) => {
                info!("LLM response received: {} chars", response.len());
                Ok(response)
            }
            Err(e) => {
                warn!("LLM call failed: {}", e);
                Err(MiddlewareError::LLMError(e.to_string()))
            }
        }
    }

    async fn handle_llm_cot(&self, ctx: &MessageContext) -> Result<String, MiddlewareError> {
        info!("Using Chain-of-Thought (CoT) reasoning");
        let reasoner = Reasoner::new(self.scheduler.clone());
        match reasoner.reason(&ctx.content).await {
            Ok(cot_result) => {
                Ok(format!(
                    "【思维链推理结果】
最终结论：{}

推理步骤：
{}",
                    cot_result.conclusion,
                    cot_result.reasoning_chain.iter()
                        .enumerate()
                        .map(|(i, s)| format!("{}. {}
   置信度: {:.2}", i + 1, s.thought, s.confidence))
                        .collect::<Vec<_>>()
                        .join("
")
                ))
            }
            Err(e) => {
                warn!("CoT reasoning failed: {}", e);
                Err(MiddlewareError::LLMError(e.to_string()))
            }
        }
    }

    async fn handle_command(&self, cmd: &str) -> Result<String, MiddlewareError> {
        info!("Handling as command...");
        match cmd.trim() {
            "/status" => Ok("System status: OK".to_string()),
            "/help" => Ok("Available commands: /status, /help, /memory".to_string()),
            "/memory" => Ok("Memory usage: normal".to_string()),
            _ => Ok(format!("Unknown command: {}", cmd)),
        }
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new(Arc::new(Scheduler::new()))
    }
}
