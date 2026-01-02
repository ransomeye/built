// Path and File Name : /home/ransomeye/rebuild/edge/sentinel/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Sentinel library - monitors Agent and DPI health, enforces runtime integrity

pub mod errors;
pub mod hardening;
pub mod monitor;
pub mod alert;
pub mod lateral_movement;

pub use errors::SentinelError;
pub use hardening::RuntimeHardening;
pub use monitor::{AgentMonitor, DpiMonitor, ComponentHealth};
pub use alert::AlertEmitter;
pub use lateral_movement::{LateralMovementDetector, LateralMovementEvent, LateralMovementType};
