// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Library root - exports all public modules for RansomEye reporting, forensics, and evidence preservation

pub mod collector;
pub mod evidence_store;
pub mod hasher;
pub mod timeline;
pub mod report_builder;
pub mod exporter;
pub mod verifier;
pub mod retention;
pub mod errors;
pub mod formats;
pub mod deception_report;
pub mod intel_report;
pub mod forensic_report;

pub use collector::EvidenceCollector;
pub use evidence_store::EvidenceStore;
pub use hasher::EvidenceHasher;
pub use timeline::ForensicTimeline;
pub use report_builder::ReportBuilder;
pub use exporter::ReportExporter;
pub use verifier::EvidenceVerifier;
pub use retention::RetentionManager;
pub use errors::ReportingError;

