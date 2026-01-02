// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: DPI Probe library - high-throughput network telemetry sensor

/// RansomEye DPI Probe
/// 
/// High-throughput network telemetry sensor that:
/// - Captures network traffic (AF_PACKET/libpcap)
/// - Parses protocols (L3-L7)
/// - Tracks flows (bounded memory)
/// - Extracts features (bounded)
/// - Creates Phase-4 event envelopes
/// - Signs events with Ed25519
/// 
/// **CRITICAL**: This module is STAND-ALONE, NO enforcement, NO policy, NO AI authority.
/// Produces validated telemetry ONLY.

pub mod errors;
pub mod capture;
pub mod parser;
pub mod flow;
pub mod extraction;
pub mod envelope;
pub mod backpressure;
pub mod rate_limit;
pub mod health;

// Security module is in probe/security/

pub use errors::ProbeError;
pub use capture::PacketCapture;
pub use parser::ProtocolParser;
pub use flow::FlowTracker;
pub use extraction::FeatureExtractor;
pub use envelope::EventEnvelope;
pub use backpressure::BackpressureManager;
pub use rate_limit::RateLimiter;
pub use health::HealthMonitor;

