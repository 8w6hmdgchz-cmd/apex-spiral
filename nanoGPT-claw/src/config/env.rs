//! Environment interpolation module
use anyhow::Result;
use std::collections::HashMap;

pub trait EnvInterpolate {
    fn interpolate_env_vars(&self) -> Result<String>;
}

impl EnvInterpolate for str {
    fn interpolate_env_vars(&self) -> Result<String> {
        let mut result = self.to_string();
        let re = regex::Regex::new(r"\$\{([^}]+)\}")?;
        for cap in re.captures_iter(self) {
            if let Ok(env_val) = std::env::var(&cap[1]) {
                result = result.replace(&cap[0], &env_val);
            }
        }
        Ok(result)
    }
}

pub fn interpolate_env_vars(_s: &str) -> Result<String> {
    Ok(_s.to_string())
}
