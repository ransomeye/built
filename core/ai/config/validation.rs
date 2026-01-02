# Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/config/validation.rs
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: ENV-only configuration validation - fails startup if required ENV variables are missing

use std::env;

/// Configuration validation for advisory module.
/// All configuration must come from environment variables.
/// Missing required ENV variables will cause startup failure.
pub struct AdvisoryConfig {
    pub model_dir: String,
    pub data_dir: String,
    pub max_tokens: u32,
    pub topk: u32,
}

impl AdvisoryConfig {
    /// Load configuration from environment variables.
    /// Returns error if any required ENV variable is missing.
    pub fn from_env() -> Result<Self, String> {
        let model_dir = env::var("MODEL_DIR")
            .map_err(|_| "MODEL_DIR environment variable is required but not set")?;
        
        let data_dir = env::var("ASSISTANT_DATA_DIR")
            .map_err(|_| "ASSISTANT_DATA_DIR environment variable is required but not set")?;
        
        let max_tokens = env::var("ASSISTANT_MAX_TOKENS")
            .unwrap_or_else(|_| "2048".to_string())
            .parse::<u32>()
            .map_err(|_| "ASSISTANT_MAX_TOKENS must be a valid integer")?;
        
        let topk = env::var("ASSISTANT_TOPK")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .map_err(|_| "ASSISTANT_TOPK must be a valid integer")?;

        Ok(AdvisoryConfig {
            model_dir,
            data_dir,
            max_tokens,
            topk,
        })
    }

    /// Validate that all required configuration is present.
    /// This is called at startup and will cause failure if validation fails.
    pub fn validate() -> Result<(), String> {
        Self::from_env()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_requires_model_dir() {
        env::remove_var("MODEL_DIR");
        env::remove_var("ASSISTANT_DATA_DIR");
        assert!(AdvisoryConfig::from_env().is_err());
    }

    #[test]
    fn test_config_requires_data_dir() {
        env::set_var("MODEL_DIR", "/tmp/models");
        env::remove_var("ASSISTANT_DATA_DIR");
        assert!(AdvisoryConfig::from_env().is_err());
    }

    #[test]
    fn test_config_succeeds_with_all_required() {
        env::set_var("MODEL_DIR", "/tmp/models");
        env::set_var("ASSISTANT_DATA_DIR", "/tmp/data");
        assert!(AdvisoryConfig::from_env().is_ok());
    }
}
