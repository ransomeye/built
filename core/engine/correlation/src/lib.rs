// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Core correlation engine library - Phase 5 main module

pub mod engine;
pub mod entity_state;
pub mod errors;
pub mod explainability;
pub mod graph;
pub mod invariants;
pub mod scheduler;
pub mod scoring;
pub mod temporal;

#[path = "../kill_chain/mod.rs"]
pub mod kill_chain;

#[path = "../input/mod.rs"]
pub mod input;

#[path = "../output/mod.rs"]
pub mod output;

// Config module is in separate directory - not included in lib for now
// pub mod config;

pub use crate::engine::{CorrelationEngine, EngineConfig, EngineStats};
pub use crate::errors::{CorrelationError, CorrelationResult};
pub use crate::input::validated_events::ValidatedEvent;
pub use crate::output::detection_result::DetectionResult;

