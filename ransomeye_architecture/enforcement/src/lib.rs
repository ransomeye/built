// Path and File Name : /home/ransomeye/rebuild/ransomeye_architecture/enforcement/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime trust boundary enforcement engine - blocks forbidden flows and enforces fail-closed behavior

pub mod boundary_enforcer;
pub mod audit_logger;
pub mod identity_enforcer;
pub mod fail_closed;
pub mod plane_classifier;
pub mod contract_version;

pub use boundary_enforcer::BoundaryEnforcer;
pub use audit_logger::AuditLogger;
pub use identity_enforcer::IdentityEnforcer;
pub use fail_closed::FailClosedGuard;
pub use plane_classifier::{Plane, PlaneClassifier};
pub use contract_version::{ContractVersion, ContractVersionEnforcer};

