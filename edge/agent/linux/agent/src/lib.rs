// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/agent/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Linux Agent library - host-based telemetry sensor

/// RansomEye Linux Agent
/// 
/// Host-based telemetry sensor that:
/// - Monitors syscalls (eBPF/auditd)
/// - Tracks process, filesystem, network events
/// - Emits signed telemetry only
/// 
/// **CRITICAL**: This module is STAND-ALONE, NO enforcement, NO policy, NO kill-switch authority.
/// Produces validated telemetry ONLY.

pub mod errors;
pub mod process;
pub mod filesystem;
pub mod network;
pub mod syscalls;
pub mod features;
pub mod envelope;
pub mod backpressure;
pub mod rate_limit;
pub mod health;
pub mod hardening;

// Security module is in agent/security/

pub use errors::AgentError;
pub use process::ProcessMonitor;
pub use filesystem::FilesystemMonitor;
pub use network::NetworkMonitor;
pub use syscalls::SyscallMonitor;
pub use features::FeatureExtractor;
pub use envelope::EventEnvelope;
pub use backpressure::BackpressureManager;
pub use rate_limit::RateLimiter;
pub use health::HealthMonitor;
pub use hardening::RuntimeHardening;

