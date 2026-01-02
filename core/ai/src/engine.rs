// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/engine.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Advisory engine - main orchestrator with advisory-only guarantees

use std::sync::Arc;
use tracing::{error, warn, info, debug};
use crate::errors::AdvisoryError;
use crate::outputs::AdvisoryOutput;
use crate::controller::AIController;
use crate::scorer::RiskScorer;
use crate::explainer::SHAPExplainer;
use crate::context::ContextEnricher;
use crate::registry::ModelRegistry;

pub struct AdvisoryEngine {
    controller: Arc<AIController>,
    scorer: Arc<RiskScorer>,
    explainer: Arc<SHAPExplainer>,
    enricher: Arc<ContextEnricher>,
    model_registry: Arc<ModelRegistry>,
}

impl AdvisoryEngine {
    pub fn new() -> Result<Self, AdvisoryError> {
        // Initialize controller
        let controller = Arc::new(AIController::new());
        
        // Initialize model registry
        let model_registry = Arc::new(ModelRegistry::new()?);
        
        // Verify baseline models are present and signed
        if !model_registry.has_baseline_models()? {
            controller.disable("Missing baseline models")?;
            return Err(AdvisoryError::MissingBaseline("No baseline models found".to_string()));
        }
        
        // Initialize SHAP explainer
        let explainer = Arc::new(SHAPExplainer::new());
        
        // Initialize scorer
        let scorer = Arc::new(RiskScorer::new(model_registry.clone(), explainer.clone()));
        
        // Initialize context enricher
        let enricher = Arc::new(ContextEnricher::new());
        
        info!("Advisory Engine initialized");
        
        Ok(Self {
            controller,
            scorer,
            explainer,
            enricher,
            model_registry,
        })
    }
    
    /// Generate advisory output (advisory only, read-only)
    pub async fn generate_advisory(&self, alert_id: &str, features: &[f64]) -> Result<AdvisoryOutput, AdvisoryError> {
        // Require AI to be enabled
        self.controller.require_enabled()?;
        
        debug!("Generating advisory for alert {}", alert_id);
        
        // Score risk (advisory only)
        let mut output = self.scorer.score_risk(features, alert_id).await?;
        
        // Verify SHAP is present (MANDATORY)
        if !output.has_shap() {
            self.controller.disable("Missing SHAP in output")?;
            return Err(AdvisoryError::MissingSHAP("Output missing SHAP explanation".to_string()));
        }
        
        // Validate SHAP
        self.explainer.validate(&output.shap_explanation)
            .map_err(|e| {
                self.controller.disable(&format!("SHAP validation failed: {}", e))?;
                e
            })?;
        
        // Enrich context (read-only)
        let context = self.enricher.enrich(alert_id).await?;
        output.context_enrichment = Some(context);
        
        // Add evidence references
        output.evidence_references.push(alert_id.to_string());
        
        debug!("Advisory output generated for alert {}", alert_id);
        Ok(output)
    }
    
    /// Check if AI is enabled
    pub fn is_enabled(&self) -> Result<bool, AdvisoryError> {
        self.controller.is_enabled()
    }
    
    /// Get AI state
    pub fn get_state(&self) -> Result<crate::controller::AIState, AdvisoryError> {
        self.controller.get_state()
    }
    
    /// Handle runtime error (fail-closed)
    pub fn handle_error(&self, error: &AdvisoryError) {
        match error {
            AdvisoryError::MissingBaseline(_) => {
                let _ = self.controller.disable("Missing baseline model");
            }
            AdvisoryError::UnsignedModel(_) => {
                let _ = self.controller.disable("Unsigned model detected");
            }
            AdvisoryError::MissingSHAP(_) => {
                let _ = self.controller.disable("Missing SHAP explanation");
            }
            AdvisoryError::RuntimeError(_) => {
                let _ = self.controller.disable("Runtime error");
            }
            _ => {
                warn!("Advisory error (non-critical): {}", error);
            }
        }
    }
}

