// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/config/validation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Configuration validation for correlation engine

use crate::engine::EngineConfig;
use crate::errors::CorrelationError;

/// Validate engine configuration
pub fn validate_config(config: &EngineConfig) -> Result<(), CorrelationError> {
    if config.max_entities == 0 {
        return Err(CorrelationError::ConfigurationError(
            "max_entities must be > 0".to_string(),
        ));
    }

    if config.max_signals_per_entity == 0 {
        return Err(CorrelationError::ConfigurationError(
            "max_signals_per_entity must be > 0".to_string(),
        ));
    }

    if config.temporal_window_seconds == 0 {
        return Err(CorrelationError::ConfigurationError(
            "temporal_window_seconds must be > 0".to_string(),
        ));
    }

    if config.min_confidence_threshold < 0.0 || config.min_confidence_threshold > 1.0 {
        return Err(CorrelationError::ConfigurationError(
            "min_confidence_threshold must be between 0.0 and 1.0".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let mut config = EngineConfig::default();
        assert!(validate_config(&config).is_ok());

        config.max_entities = 0;
        assert!(validate_config(&config).is_err());
    }
}

