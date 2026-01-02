// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/inference/src/calibration.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Confidence calibration for advisory outputs

use ndarray::Array1;
use tracing::debug;

use super::errors::InferenceError;

#[derive(Debug, Clone)]
pub struct CalibratedOutput {
    pub confidence: f64,
    pub calibrated_confidence: f64,
    pub risk_score: f64,
}

pub struct ConfidenceCalibrator {
    calibration_model: Option<CalibrationModel>,
}

#[derive(Debug, Clone)]
struct CalibrationModel {
    // Simplified calibration model
    // In production, would use Platt scaling, isotonic regression, or temperature scaling
    temperature: f64,
    bias: f64,
}

impl ConfidenceCalibrator {
    pub fn new() -> Self {
        Self {
            calibration_model: Some(CalibrationModel {
                temperature: 1.0,
                bias: 0.0,
            }),
        }
    }
    
    /// Calibrate confidence score
    pub fn calibrate(&self, raw_confidence: f64, _features: &Array1<f64>) -> Result<CalibratedOutput, InferenceError> {
        let calibration = self.calibration_model.as_ref()
            .ok_or_else(|| InferenceError::CalibrationFailed("Calibration model not loaded".to_string()))?;
        
        // Apply temperature scaling
        let calibrated = (raw_confidence / calibration.temperature + calibration.bias)
            .max(0.0)
            .min(1.0);
        
        // Compute risk score (advisory only)
        let risk_score = calibrated * 100.0;
        
        debug!("Calibration: raw={:.3}, calibrated={:.3}, risk_score={:.1}", 
            raw_confidence, calibrated, risk_score);
        
        Ok(CalibratedOutput {
            confidence: raw_confidence,
            calibrated_confidence: calibrated,
            risk_score,
        })
    }
}

