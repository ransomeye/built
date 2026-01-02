// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/agent/src/filesystem.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Filesystem monitoring - rename, unlink, chmod, mass writes

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

use super::errors::AgentError;

/// Filesystem event types
#[derive(Debug, Clone, PartialEq)]
pub enum FilesystemEventType {
    Rename,
    Unlink,
    Chmod,
    MassWrite,
    Create,
    Open,
}

/// Filesystem event
#[derive(Debug, Clone)]
pub struct FilesystemEvent {
    pub event_type: FilesystemEventType,
    pub pid: u32,
    pub uid: u32,
    pub gid: u32,
    pub path: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub mode: Option<u32>,
    pub write_count: Option<u64>,
    pub timestamp: u64,
}

/// Filesystem monitor
/// 
/// Tracks filesystem events: rename, unlink, chmod, mass writes.
/// Bounded memory for path tracking.
pub struct FilesystemMonitor {
    write_counts: Arc<parking_lot::RwLock<std::collections::HashMap<String, u64>>>,
    events_processed: Arc<AtomicU64>,
    mass_write_threshold: u64,
}

impl FilesystemMonitor {
    /// Create new filesystem monitor
    pub fn new(mass_write_threshold: u64) -> Self {
        Self {
            write_counts: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            events_processed: Arc::new(AtomicU64::new(0)),
            mass_write_threshold,
        }
    }
    
    /// Record rename event
    pub fn record_rename(&self, pid: u32, uid: u32, gid: u32, old_path: String, new_path: String) -> Result<FilesystemEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::FilesystemMonitoringFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        
        debug!("Filesystem rename: {} -> {}", old_path, new_path);
        
        Ok(FilesystemEvent {
            event_type: FilesystemEventType::Rename,
            pid,
            uid,
            gid,
            path: new_path.clone(),
            old_path: Some(old_path),
            new_path: Some(new_path),
            mode: None,
            write_count: None,
            timestamp,
        })
    }
    
    /// Record unlink event
    pub fn record_unlink(&self, pid: u32, uid: u32, gid: u32, path: String) -> Result<FilesystemEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::FilesystemMonitoringFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        
        debug!("Filesystem unlink: {}", path);
        
        Ok(FilesystemEvent {
            event_type: FilesystemEventType::Unlink,
            pid,
            uid,
            gid,
            path,
            old_path: None,
            new_path: None,
            mode: None,
            write_count: None,
            timestamp,
        })
    }
    
    /// Record chmod event
    pub fn record_chmod(&self, pid: u32, uid: u32, gid: u32, path: String, mode: u32) -> Result<FilesystemEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::FilesystemMonitoringFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        
        debug!("Filesystem chmod: {} mode={:o}", path, mode);
        
        Ok(FilesystemEvent {
            event_type: FilesystemEventType::Chmod,
            pid,
            uid,
            gid,
            path,
            old_path: None,
            new_path: None,
            mode: Some(mode),
            write_count: None,
            timestamp,
        })
    }
    
    /// Record write event (tracks mass writes)
    pub fn record_write(&self, pid: u32, uid: u32, gid: u32, path: String) -> Result<Option<FilesystemEvent>, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::FilesystemMonitoringFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        // Track write count
        let write_count = {
            let mut counts = self.write_counts.write();
            let count = counts.entry(path.clone()).or_insert(0);
            *count += 1;
            *count
        };
        
        // Check if mass write threshold exceeded
        if write_count >= self.mass_write_threshold {
            self.events_processed.fetch_add(1, Ordering::Relaxed);
            
            debug!("Filesystem mass write detected: {} ({} writes)", path, write_count);
            
            return Ok(Some(FilesystemEvent {
                event_type: FilesystemEventType::MassWrite,
                pid,
                uid,
                gid,
                path,
                old_path: None,
                new_path: None,
                mode: None,
                write_count: Some(write_count),
                timestamp,
            }));
        }
        
        Ok(None)
    }
    
    /// Get events processed
    pub fn events_processed(&self) -> u64 {
        self.events_processed.load(Ordering::Relaxed)
    }
}

