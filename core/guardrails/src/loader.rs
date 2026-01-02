// Path and File Name : /home/ransomeye/rebuild/core/guardrails/src/loader.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Loads and parses guardrails.yaml specification

use std::path::{Path, PathBuf};
use crate::errors::{GuardrailError, GuardrailResult};
use crate::spec::GuardrailSpec;

pub struct GuardrailLoader {
    spec_path: PathBuf,
}

impl GuardrailLoader {
    pub fn new<P: AsRef<Path>>(spec_path: P) -> Self {
        Self {
            spec_path: spec_path.as_ref().to_path_buf(),
        }
    }

    pub fn default() -> Self {
        Self::new("/home/ransomeye/rebuild/core/guardrails/guardrails.yaml")
    }

    /// Load guardrails.yaml and parse it
    /// Does NOT verify signature - that's done by GuardrailVerifier
    pub fn load(&self) -> GuardrailResult<GuardrailSpec> {
        if !self.spec_path.exists() {
            return Err(GuardrailError::SpecNotFound(
                self.spec_path.display().to_string(),
            ));
        }

        let content = std::fs::read_to_string(&self.spec_path)?;
        let spec: GuardrailSpec = serde_yaml::from_str(&content)
            .map_err(|e| GuardrailError::SchemaValidationFailed(format!("YAML parse error: {}", e)))?;

        Ok(spec)
    }

    /// Get the path to the specification file
    pub fn spec_path(&self) -> &Path {
        &self.spec_path
    }
}

