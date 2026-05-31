//! NanoGPT-Claw - System Coordinator
//!
//! Coordinates all modules: Web UI ↔ LLM ↔ Skills ↔ Memory ↔ Evolution

use crate::skill::SkillRegistry;
use crate::skill::built_in;
use crate::evolution::apex_akashic::ApexAkashicResult;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    UserMessage { content: String, timestamp: i64 },
    LlmResponse { content: String, model: String },
}

#[derive(Clone)]
pub struct SystemCoordinatorState {
    pub skill_registry: Arc<SkillRegistry>,
    pub event_log: Arc<RwLock<Vec<SystemEvent>>>,
}

pub struct SystemCoordinator {
    state: SystemCoordinatorState,
}

impl SystemCoordinator {
    pub fn new() -> Self {
        let registry = SkillRegistry::new();
        built_in::register_all(&registry);
        
        Self {
            state: SystemCoordinatorState {
                skill_registry: Arc::new(registry),
                event_log: Arc::new(RwLock::new(Vec::new())),
            },
        }
    }

    pub fn get_state(&self) -> SystemCoordinatorState {
        self.state.clone()
    }

    pub async fn process_user_input(&self, content: String) -> String {
        self.log_event(SystemEvent::UserMessage {
            content: content.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        }).await;
        
        format!("[Coordinated Response] Processed: {}", content)
    }

    pub async fn get_apex_score(&self) -> ApexAkashicResult {
        ApexAkashicResult {
            final_score: 0.75,
            omega_a: 1.0,
            dimension_product_1: 0.8,
            dimension_product_2: 0.9,
            total_penalty: 0.1,
            penalties: Default::default(),
            factors: Default::default(),
            recommendations: vec![],
            confidence: 0.8,
        }
    }

    pub async fn get_memory_count(&self) -> usize {
        let log = self.state.event_log.read().await;
        log.len()
    }

    async fn log_event(&self, event: SystemEvent) {
        let mut log = self.state.event_log.write().await;
        log.push(event);
    }

    pub async fn get_event_log(&self) -> Vec<SystemEvent> {
        let log = self.state.event_log.read().await;
        log.clone()
    }
}

impl Default for SystemCoordinator {
    fn default() -> Self { Self::new() }
}
