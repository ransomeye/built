// Path and File Name : /home/ransomeye/rebuild/core/deception/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Phase 16 - Deception Framework library entry point

#![cfg(feature = "future-deception")]

pub mod asset;
pub mod errors;
pub mod registry;
pub mod deployer;
pub mod signals;
pub mod correlation;
pub mod playbook_integration;
pub mod visibility;
pub mod teardown;
pub mod security;

pub use asset::DeceptionAsset;
pub use errors::DeceptionError;
pub use registry::DeceptionRegistry;
pub use deployer::{DeceptionDeployer, DeploymentState};
pub use signals::DeceptionSignal;
pub use teardown::TeardownEngine;
pub use visibility::DeceptionVisibility;
pub use crate::playbook_integration::PlaybookIntegration;

