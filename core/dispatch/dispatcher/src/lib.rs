// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Dispatcher library module exports

// Protocol modules (from root protocol/)
#[path = "../../protocol/directive_envelope.rs"]
pub mod directive_envelope;
#[path = "../../protocol/acknowledgment_envelope.rs"]
pub mod acknowledgment_envelope;

// Security modules (from root security/)
#[path = "../../security/signature.rs"]
pub mod signature;
#[path = "../../security/trust_chain.rs"]
pub mod trust_chain;
#[path = "../../security/nonce.rs"]
pub mod nonce;
#[path = "../../security/replay_protection.rs"]
pub mod replay_protection;

// Config module (from root config/)
#[path = "../../config/validation.rs"]
pub mod validation;

// Protocol re-exports
pub use directive_envelope::{DirectiveEnvelope, TargetScope, AuditReceipt};
pub use acknowledgment_envelope::{AcknowledgmentEnvelope, ExecutionResult};

// Security re-exports
pub use signature::SignatureVerifier;
pub use trust_chain::TrustChain;
pub use nonce::NonceTracker;
pub use replay_protection::ReplayProtector;

// Config re-exports
pub use validation::ConfigValidator;

// Dispatcher modules
pub mod errors;
pub mod verifier;
pub mod router;
pub mod delivery;
pub mod acknowledgment;
pub mod timeout;
pub mod replay;
pub mod reentrancy;
pub mod rollback;
pub mod audit;
pub mod dispatcher;
pub mod safety;

pub use dispatcher::EnforcementDispatcher;
pub use errors::DispatcherError;
pub use audit::{AuditLogger, AuditEventType};
pub use verifier::DirectiveVerifier;
pub use router::{TargetRouter, AgentInfo};
pub use replay::ReplayGuard;
pub use reentrancy::ReentrancyGuard;
pub use safety::SafetyGuards;
pub use timeout::TimeoutManager;
