// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Library root - exports all public modules for RansomEye reporting, forensics, and evidence preservation

// Internal modules - gated behind features
#[cfg(feature = "future-reporting")]
mod collector;
#[cfg(feature = "future-reporting")]
mod evidence_store;
#[cfg(feature = "future-reporting")]
mod hasher;
#[cfg(feature = "future-reporting")]
mod timeline;
#[cfg(feature = "future-reporting")]
mod report_builder;
#[cfg(feature = "future-reporting")]
mod exporter;
#[cfg(feature = "future-reporting")]
mod verifier;
#[cfg(feature = "future-retention")]
mod retention;
pub mod errors;
#[cfg(feature = "future-reporting")]
pub mod formats;
#[cfg(feature = "future-reporting")]
mod deception_report;
#[cfg(feature = "future-reporting")]
mod intel_report;
#[cfg(feature = "future-reporting")]
mod forensic_report;

// Public API exports - gated behind features
#[cfg(feature = "future-reporting")]
pub use collector::EvidenceCollector;
#[cfg(feature = "future-reporting")]
pub use evidence_store::EvidenceStore;
#[cfg(feature = "future-reporting")]
pub use hasher::EvidenceHasher;
#[cfg(feature = "future-reporting")]
pub use timeline::ForensicTimeline;
#[cfg(feature = "future-reporting")]
pub use report_builder::ReportBuilder;
#[cfg(feature = "future-reporting")]
pub use exporter::ReportExporter;
#[cfg(feature = "future-reporting")]
pub use verifier::EvidenceVerifier;
#[cfg(feature = "future-retention")]
pub use retention::RetentionManager;
pub use errors::ReportingError;

