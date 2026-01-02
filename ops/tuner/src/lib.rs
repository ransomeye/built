// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Library root - exports all public modules for RansomEye operations, installer, uninstaller, and lifecycle management

pub mod installer;
pub mod uninstaller;
pub mod lifecycle;
pub mod errors;

pub use installer::Installer;
pub use installer::preflight::PreflightChecker;
pub use installer::retention::RetentionConfigurator;
pub use installer::crypto::CryptoIdentity;
pub use installer::state::InstallState;
pub use uninstaller::Uninstaller;
pub use lifecycle::status::ServiceStatus;
pub use lifecycle::start::ServiceStarter;
pub use lifecycle::stop::ServiceStopper;
pub use lifecycle::restart::ServiceRestarter;
pub use lifecycle::status::ServiceStatusChecker;
pub use errors::OperationsError;

