//! Environment Variable Interpolation

use once_cell::sync::Lazy;
use regex::Regex;
use tracing::{debug, warn};

static ENV_VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")
        .unwrap_or_else(|_| panic!("Invalid environment variable regex pattern"))
});

pub fn interpolate_env_vars(s: &str) -> String {
    let mut result = s.to_string();
    
    for cap in ENV_VAR_REGEX.captures_iter(s) {
        if let Some(var_name) = cap.get(1) {
            let var_name = var_name.as_str();
            match std::env::var(var_name) {
                Ok(value) => {
                    debug!("Replacing ${} with environment variable value", var_name);
                    let placeholder = format!("${{{}}}", var_name);
                    result = result.replace(&placeholder, &value);
                }
                Err(_) => {
                    warn!("Environment variable ${} not found, leaving placeholder", var_name);
                }
            }
        }
    }
    
    result
}

pub fn interpolate_env_vars_recursive(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            *s = interpolate_env_vars(s);
        }
        serde_json::Value::Object(obj) => {
            for (_, v) in obj.iter_mut() {
                interpolate_env_vars_recursive(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                interpolate_env_vars_recursive(v);
            }
        }
        _ => {}
    }
}

pub trait EnvInterpolate {
    fn interpolate_env(self) -> Self;
}

impl EnvInterpolate for String {
    fn interpolate_env(self) -> Self {
        interpolate_env_vars(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_env_vars() {
        std::env::set_var("TEST_VAR", "test_value");
        
        let result = interpolate_env_vars("Hello ${TEST_VAR}");
        assert_eq!(result, "Hello test_value");
        
        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_interpolate_missing_env_var() {
        let result = interpolate_env_vars("Hello ${MISSING_VAR}");
        assert_eq!(result, "Hello ${MISSING_VAR}");
    }
}
