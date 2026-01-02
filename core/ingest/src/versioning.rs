// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/versioning.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Version management - manages schema version compatibility

/*
 * Version Manager
 * 
 * Manages schema version compatibility.
 * Enforces explicit version rules.
 * No auto-upgrade.
 */

use std::collections::HashSet;
use tracing::warn;

pub struct VersionManager {
    supported_versions: HashSet<u32>,
    current_version: u32,
}

impl VersionManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut supported_versions = HashSet::new();
        supported_versions.insert(1); // v1 is supported
        
        Ok(Self {
            supported_versions,
            current_version: 1,
        })
    }
    
    pub fn is_compatible(&self, version: u32) -> bool {
        if !self.supported_versions.contains(&version) {
            warn!("Unsupported schema version: {}", version);
            return false;
        }
        
        // Version 1 is always compatible with itself
        version == self.current_version
    }
    
    pub fn get_current_version(&self) -> u32 {
        self.current_version
    }
}

