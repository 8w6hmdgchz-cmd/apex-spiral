//! Settings module placeholder
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Settings {
    pub debug: bool,
}

impl Settings {
    pub fn new() -> Self {
        Self { debug: false }
    }
}
