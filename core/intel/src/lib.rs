// Path and File Name : /home/ransomeye/rebuild/core/intel/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Intel library root - exports correlation and confidence scoring modules

pub mod correlation;
pub mod confidence;
pub mod scoring;
pub mod policy_integration;
pub mod errors;

pub use correlation::IntelCorrelator;
pub use confidence::ConfidenceScorer;
pub use scoring::IntelScorer;
pub use policy_integration::PolicyIntegration;
pub use errors::IntelError;
