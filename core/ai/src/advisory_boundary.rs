// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/advisory_boundary.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Advisory-only boundary enforcement - prevents AI from influencing enforcement

use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{error, warn, info};
use serde::{Deserialize, Serialize};

/// Advisory-only output structure - NO enforcement fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisoryOutput {
    pub confidence: f64,
    pub calibrated_confidence: f64,
    pub risk_score: f64,
    pub rationale: String,
    pub feature_contributions: Vec<FeatureContribution>,
    pub evidence_references: Vec<String>,
    pub recommended_investigation_steps: Vec<String>,
    
    // EXPLICITLY FORBIDDEN FIELDS (compile-time prevention):
    // - enforcement_directive: NOT PRESENT
    // - policy_modification: NOT PRESENT
    // - state_change: NOT PRESENT
    // - control_command: NOT PRESENT
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureContribution {
    pub feature_name: String,
    pub contribution: f64,
    pub signal_reference: String,
    pub timestamp_reference: String,
}

pub struct AdvisoryBoundaryGuard {
    violations: Arc<RwLock<Vec<BoundaryViolation>>>,
}

#[derive(Debug, Clone)]
struct BoundaryViolation {
    timestamp: chrono::DateTime<chrono::Utc>,
    violation_type: String,
    context: String,
}

impl AdvisoryBoundaryGuard {
    pub fn new() -> Self {
        Self {
            violations: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Verify output is advisory-only (no enforcement)
    pub fn verify_advisory_only(&self, output: &AdvisoryOutput) -> Result<(), String> {
        // Verify no enforcement language in rationale
        let enforcement_keywords = vec!["enforce", "block", "isolate", "quarantine", "execute"];
        let rationale_lower = output.rationale.to_lowercase();
        let enforcement_keywords_clone = enforcement_keywords.clone();
        
        for keyword in enforcement_keywords_clone {
            if rationale_lower.contains(keyword) {
                let violation = BoundaryViolation {
                    timestamp: chrono::Utc::now(),
                    violation_type: "Enforcement language in rationale".to_string(),
                    context: format!("Keyword '{}' found in rationale", keyword),
                };
                
                self.record_violation(violation);
                return Err(format!("Advisory boundary violation: enforcement language detected"));
            }
        }
        
        // Verify recommended steps are advisory only
        let enforcement_keywords_clone2 = enforcement_keywords.clone();
        for step in &output.recommended_investigation_steps {
            let step_lower = step.to_lowercase();
            for keyword in &enforcement_keywords_clone2 {
                if step_lower.contains(keyword) {
                    let violation = BoundaryViolation {
                        timestamp: chrono::Utc::now(),
                        violation_type: "Enforcement language in recommendations".to_string(),
                        context: format!("Keyword '{}' found in step: {}", keyword, step),
                    };
                    
                    self.record_violation(violation);
                    return Err(format!("Advisory boundary violation: enforcement language in recommendations"));
                }
            }
        }
        
        Ok(())
    }
    
    fn record_violation(&self, violation: BoundaryViolation) {
        error!("ADVISORY BOUNDARY VIOLATION: {:?}", violation);
        let mut violations = self.violations.write();
        violations.push(violation);
        
        // In production, would also write to audit log
    }
    
    /// Get violation count
    pub fn violation_count(&self) -> usize {
        self.violations.read().len()
    }
}

