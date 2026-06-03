use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionResult {
    pub quality_score: f32,
    pub consistency_score: f32,
    pub bias_identified: Vec<String>,
    pub improvements: Vec<String>,
    pub lessons: Vec<String>,
}
