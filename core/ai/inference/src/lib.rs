// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/inference/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Advisory-only inference module - provides AI recommendations without enforcement

/// Advisory-only inference module.
/// This module provides AI-powered recommendations and analysis.
/// It does NOT perform enforcement, policy execution, or dispatcher actions.

pub mod errors;
pub mod loader;
pub mod inference;
pub mod calibration;
pub mod thresholds;
pub mod features;

pub use errors::InferenceError;
pub use loader::{ModelLoader, LoadedModel, ModelManifest};
pub use inference::{AdvisoryInference, AdvisoryInferenceResult};
pub use calibration::{ConfidenceCalibrator, CalibratedOutput};
pub use thresholds::ThresholdManager;
pub use features::FeatureExtractor;
