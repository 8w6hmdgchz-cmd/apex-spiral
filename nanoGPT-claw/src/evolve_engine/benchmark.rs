use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTask;

pub struct Benchmarker;

impl Benchmarker {
    pub fn new() -> Self {
        Self
    }
}
