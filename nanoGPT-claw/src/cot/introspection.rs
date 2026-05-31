//! Introspection module placeholder

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Medium
    }
}
