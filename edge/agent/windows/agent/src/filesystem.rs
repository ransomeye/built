// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/agent/src/filesystem.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Filesystem monitoring - rename, delete, permission changes, mass writes

#![cfg(feature = "future-windows-agent")]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

use super::errors::AgentError;

/// Filesystem event types
#[derive(Debug, Clone, PartialEq)]
pub enum FilesystemEventType {
    Rename,
    Delete,
    PermissionChange,
    MassWrite,
}

/// Filesystem event
#[derive(Debug, Clone)]
pub struct FilesystemEvent {
    pub event_type: FilesystemEventType,
    pub path: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub pid: u32,
    pub timestamp: u64,
    pub write_count: Option<u64>,
}

/// Filesystem monitor
/// 
/// Tracks filesystem events: rename, delete, permission changes, mass writes.
/// Bounded memory for path tracking.
pub struct FilesystemMonitor {
    write_counts: Arc<parking_lot::RwLock<std::collections::HashMap<String, u64>>>,
    max_tracked_paths: usize,
    events_processed: Arc<AtomicU64>,
    mass_write_threshold: u64,
}

impl FilesystemMonitor {
    /// Create new filesystem monitor
    pub fn new(max_tracked_paths: usize, mass_write_threshold: u64) -> Self {
        Self {
            write_counts: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            max_tracked_paths,
            events_processed: Arc::new(AtomicU64::new(0)),
            mass_write_threshold,
        }
    }
    
    /// Get filesystem rename event
    pub fn get_rename_event(&self, old_path: String, new_path: String, pid: u32) -> Result<FilesystemEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::FilesystemMonitoringFailed(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        self.events_processed.fetch_add(1, Ordering::AcqRel);
        
        Ok(FilesystemEvent {
            event_type: FilesystemEventType::Rename,
            path: new_path.clone(),
            old_path: Some(old_path),
            new_path: Some(new_path),
            pid,
            timestamp,
            write_count: None,
        })
    }
    
    /// Get filesystem delete event
    pub fn get_delete_event(&self, path: String, pid: u32) -> Result<FilesystemEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::FilesystemMonitoringFailed(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        // Remove from write tracking
        let mut write_counts = self.write_counts.write();
        write_counts.remove(&path);
        
        self.events_processed.fetch_add(1, Ordering::AcqRel);
        
        Ok(FilesystemEvent {
            event_type: FilesystemEventType::Delete,
            path,
            old_path: None,
            new_path: None,
            pid,
            timestamp,
            write_count: None,
        })
    }
    
    /// Get filesystem permission change event
    pub fn get_permission_change_event(&self, path: String, pid: u32) -> Result<FilesystemEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::FilesystemMonitoringFailed(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        self.events_processed.fetch_add(1, Ordering::AcqRel);
        
        Ok(FilesystemEvent {
            event_type: FilesystemEventType::PermissionChange,
            path,
            old_path: None,
            new_path: None,
            pid,
            timestamp,
            write_count: None,
        })
    }
    
    /// Track write operation and detect mass writes
    pub fn track_write(&self, path: String, pid: u32) -> Result<Option<FilesystemEvent>, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::FilesystemMonitoringFailed(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        let mut write_counts = self.write_counts.write();
        
        // Increment write count. We intentionally keep the `entry()` mutable borrow
        // scoped to this block so subsequent reads/mutations of the map remain legal.
        let current_count: u64 = {
            let count = write_counts.entry(path.clone()).or_insert(0);
            *count += 1;
            *count
        };
        
        // Enforce memory bounds
        if write_counts.len() > self.max_tracked_paths {
            self.evict_oldest_paths(&mut write_counts);
        }
        
        // Check for mass write
        if current_count >= self.mass_write_threshold {
            self.events_processed.fetch_add(1, Ordering::AcqRel);
            
            return Ok(Some(FilesystemEvent {
                event_type: FilesystemEventType::MassWrite,
                path,
                old_path: None,
                new_path: None,
                pid,
                timestamp,
                write_count: Some(current_count),
            }));
        }
        
        Ok(None)
    }
    
    /// Evict oldest paths to maintain memory bounds
    fn evict_oldest_paths(&self, write_counts: &mut std::collections::HashMap<String, u64>) {
        if write_counts.len() <= self.max_tracked_paths {
            return;
        }
        
        let to_remove = write_counts.len() - self.max_tracked_paths;
        let keys: Vec<String> = write_counts.keys().take(to_remove).cloned().collect();
        for key in keys {
            write_counts.remove(&key);
        }
        
        debug!("Evicted {} oldest paths", to_remove);
    }
    
    /// Get events processed count
    pub fn events_processed(&self) -> u64 {
        self.events_processed.load(Ordering::Acquire)
    }
}

