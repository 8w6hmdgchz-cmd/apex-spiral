//! Skill System Module
//!
//! Implements a flexible skill system for NanoGPT-Claw,
//! allowing dynamic registration and execution of skills.

pub mod built_in;
pub mod auto_fix;
pub mod github_api;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, debug};
use parking_lot::RwLock;

/// Skill execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    pub success: bool,
    pub output: String,
    pub metadata: HashMap<String, String>,
    pub execution_time_ms: u64,
}

/// Skill metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: SkillCategory,
    pub enabled: bool,
    pub parameters: Vec<SkillParameter>,
}

/// Skill category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Code,
    Writing,
    Analysis,
    Automation,
    Research,
    Utility,
    Custom,
}

/// Skill parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillParameter {
    pub name: String,
    pub description: String,
    pub param_type: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Skill trait - all skills must implement this
#[async_trait]
pub trait Skill: Send + Sync {
    fn metadata(&self) -> &SkillMetadata;
    
    async fn execute(&self, params: HashMap<String, String>) -> Result<SkillResult, SkillError>;
    
    fn validate_params(&self, params: &HashMap<String, String>) -> Result<(), SkillError> {
        let metadata = self.metadata();
        for param in &metadata.parameters {
            if param.required && !params.contains_key(&param.name) {
                return Err(SkillError::MissingParameter(param.name.clone()));
            }
        }
        Ok(())
    }
}

/// Skill execution error
#[derive(thiserror::Error, Debug)]
pub enum SkillError {
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    
    #[error("Invalid parameter value: {0}")]
    InvalidParameter(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Skill not found: {0}")]
    NotFound(String),
    
    #[error("Skill disabled: {0}")]
    Disabled(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Skill registry - manages all available skills
pub struct SkillRegistry {
    skills: RwLock<HashMap<String, Arc<dyn Skill>>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn register(&self, skill: Arc<dyn Skill>) {
        let id = skill.metadata().id.clone();
        info!("Registering skill: {}", id);
        self.skills.write().insert(id, skill);
    }
    
    pub fn get(&self, id: &str) -> Option<Arc<dyn Skill>> {
        self.skills.read().get(id).cloned()
    }
    
    pub fn list_all(&self) -> Vec<SkillMetadata> {
        self.skills
            .read()
            .values()
            .map(|s| s.metadata().clone())
            .collect()
    }
    
    pub fn list_by_category(&self, category: SkillCategory) -> Vec<SkillMetadata> {
        self.skills
            .read()
            .values()
            .filter(|s| s.metadata().category == category)
            .map(|s| s.metadata().clone())
            .collect()
    }
    
    pub async fn execute_skill(&self, id: &str, params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        let skill = self.get(id).ok_or_else(|| SkillError::NotFound(id.to_string()))?;
        
        let metadata = skill.metadata();
        if !metadata.enabled {
            return Err(SkillError::Disabled(id.to_string()));
        }
        
        skill.validate_params(&params)?;
        
        debug!("Executing skill: {} with params: {:?}", id, params);
        let start = std::time::Instant::now();
        let result = skill.execute(params).await;
        let duration = start.elapsed();
        
        match result {
            Ok(mut r) => {
                r.execution_time_ms = duration.as_millis() as u64;
                info!("Skill {} executed successfully in {}ms", id, r.execution_time_ms);
                Ok(r)
            }
            Err(e) => {
                warn!("Skill {} execution failed: {}", id, e);
                Err(e)
            }
        }
    }
    
    pub async fn execute(&self, id: &str, params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        self.execute_skill(id, params).await
    }
    
    pub fn list_skills(&self) -> Vec<SkillMetadata> {
        self.list_all()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Example skill - Echo skill
pub struct EchoSkill {
    metadata: SkillMetadata,
}

impl EchoSkill {
    pub fn new() -> Self {
        Self {
            metadata: SkillMetadata {
                id: "echo".to_string(),
                name: "Echo".to_string(),
                version: "1.0.0".to_string(),
                description: "Echo back the input message".to_string(),
                author: "NanoGPT-Claw".to_string(),
                category: SkillCategory::Utility,
                enabled: true,
                parameters: vec![
                    SkillParameter {
                        name: "message".to_string(),
                        description: "Message to echo".to_string(),
                        param_type: "string".to_string(),
                        required: true,
                        default_value: None,
                    },
                ],
            },
        }
    }
}

#[async_trait]
impl Skill for EchoSkill {
    fn metadata(&self) -> &SkillMetadata {
        &self.metadata
    }
    
    async fn execute(&self, params: HashMap<String, String>) -> Result<SkillResult, SkillError> {
        let message = params.get("message")
            .ok_or_else(|| SkillError::MissingParameter("message".to_string()))?;
        
        let mut metadata = HashMap::new();
        metadata.insert("length".to_string(), message.len().to_string());
        
        Ok(SkillResult {
            success: true,
            output: message.clone(),
            metadata,
            execution_time_ms: 0,
        })
    }
}

impl Default for EchoSkill {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize built-in skills
pub fn initialize_skills(registry: &SkillRegistry) {
    info!("Initializing built-in skills");
    registry.register(Arc::new(EchoSkill::new()));
    built_in::register_all(registry);
    registry.register(Arc::new(auto_fix::AutoFixSkill::new()));
    registry.register(Arc::new(github_api::GitHubApiSkill::new()));
}
