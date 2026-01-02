// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/agent/src/features.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Bounded feature extraction from syscall events

use tracing::debug;

use super::errors::AgentError;
use super::process::ProcessEvent;
use super::filesystem::FilesystemEvent;
use super::network::NetworkEvent;

/// Extracted features (bounded)
#[derive(Debug, Clone)]
pub struct Features {
    pub event_type: String,
    pub pid: u32,
    pub uid: u32,
    pub gid: u32,
    pub syscall_number: Option<u64>,
    pub path_count: usize,
    pub network_activity: bool,
    pub process_activity: bool,
    pub filesystem_activity: bool,
}

pub struct FeatureExtractor {
    max_features: usize,
    max_paths: usize,
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {
            max_features: 100, // Bounded feature count
            max_paths: 50,      // Bounded path count
        }
    }
    
    /// Extract features from process event (bounded)
    pub fn extract_from_process(&self, event: &ProcessEvent) -> Result<Features, AgentError> {
        if self.max_features == 0 {
            return Err(AgentError::FeatureExtractionFailed(
                "Feature extraction disabled".to_string()
            ));
        }
        
        debug!("Extracting features from process event: pid={}, type={:?}", 
            event.pid, event.event_type);
        
        Ok(Features {
            event_type: format!("{:?}", event.event_type),
            pid: event.pid,
            uid: event.uid,
            gid: event.gid,
            syscall_number: None,
            path_count: 0,
            network_activity: false,
            process_activity: true,
            filesystem_activity: false,
        })
    }
    
    /// Extract features from filesystem event (bounded)
    pub fn extract_from_filesystem(&self, event: &FilesystemEvent) -> Result<Features, AgentError> {
        if self.max_features == 0 {
            return Err(AgentError::FeatureExtractionFailed(
                "Feature extraction disabled".to_string()
            ));
        }
        
        let path_count = if event.old_path.is_some() { 2 } else { 1 };
        
        if path_count > self.max_paths {
            return Err(AgentError::FeatureExtractionFailed(
                format!("Path count {} exceeds maximum {}", path_count, self.max_paths)
            ));
        }
        
        debug!("Extracting features from filesystem event: pid={}, type={:?}", 
            event.pid, event.event_type);
        
        Ok(Features {
            event_type: format!("{:?}", event.event_type),
            pid: event.pid,
            uid: event.uid,
            gid: event.gid,
            syscall_number: None,
            path_count,
            network_activity: false,
            process_activity: false,
            filesystem_activity: true,
        })
    }
    
    /// Extract features from network event (bounded)
    pub fn extract_from_network(&self, event: &NetworkEvent) -> Result<Features, AgentError> {
        if self.max_features == 0 {
            return Err(AgentError::FeatureExtractionFailed(
                "Feature extraction disabled".to_string()
            ));
        }
        
        debug!("Extracting features from network event: pid={}, type={:?}", 
            event.pid, event.event_type);
        
        Ok(Features {
            event_type: format!("{:?}", event.event_type),
            pid: event.pid,
            uid: event.uid,
            gid: event.gid,
            syscall_number: None,
            path_count: 0,
            network_activity: true,
            process_activity: false,
            filesystem_activity: false,
        })
    }
    
    /// Get maximum feature count
    pub fn max_features(&self) -> usize {
        self.max_features
    }
    
    /// Get maximum path count
    pub fn max_paths(&self) -> usize {
        self.max_paths
    }
}

