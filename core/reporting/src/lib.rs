// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Library root - exports all public modules for RansomEye reporting, forensics, and evidence preservation

// Internal modules - always compiled (needed for internal dependencies)
mod collector;
mod evidence_store;
mod hasher;
mod timeline;
mod report_builder;
mod exporter;
mod verifier;
mod retention;
pub mod errors;
pub mod formats;
mod deception_report;
mod intel_report;
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

