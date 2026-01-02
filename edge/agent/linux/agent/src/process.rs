// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/agent/src/process.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Process monitoring - exec, fork, mmap syscalls

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use parking_lot::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

use super::errors::AgentError;

/// Process event types
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ProcessEventType {
    Exec,
    Fork,
    Mmap,
    Exit,
}

/// Process event
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessEvent {
    pub event_type: ProcessEventType,
    pub pid: u32,
    pub ppid: Option<u32>,
    pub uid: u32,
    pub gid: u32,
    pub executable: Option<String>,
    pub command_line: Option<String>,
    pub timestamp: u64,
    pub mmap_address: Option<u64>,
    pub mmap_size: Option<u64>,
}

/// Process monitor
/// 
/// Tracks process events: exec, fork, mmap.
/// Bounded memory for process tracking.
pub struct ProcessMonitor {
    processes: Arc<RwLock<HashMap<u32, ProcessInfo>>>,
    max_processes: usize,
    events_processed: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    ppid: Option<u32>,
    uid: u32,
    gid: u32,
    executable: Option<String>,
    first_seen: u64,
    last_seen: u64,
}

impl ProcessMonitor {
    /// Create new process monitor
    pub fn new(max_processes: usize) -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            max_processes,
            events_processed: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Record exec event
    pub fn record_exec(&self, pid: u32, ppid: Option<u32>, uid: u32, gid: u32, 
                     executable: String, command_line: Option<String>) -> Result<ProcessEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::ProcessMonitoringFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        // Update process info
        {
            let mut processes = self.processes.write();
            
            // Check memory bound
            if processes.len() >= self.max_processes {
                self.evict_oldest(&mut processes);
            }
            
            processes.insert(pid, ProcessInfo {
                pid,
                ppid,
                uid,
                gid,
                executable: Some(executable.clone()),
                first_seen: timestamp,
                last_seen: timestamp,
            });
        }
        
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        
        debug!("Process exec: pid={}, executable={}", pid, executable);
        
        Ok(ProcessEvent {
            event_type: ProcessEventType::Exec,
            pid,
            ppid,
            uid,
            gid,
            executable: Some(executable),
            command_line,
            timestamp,
            mmap_address: None,
            mmap_size: None,
        })
    }
    
    /// Record fork event
    pub fn record_fork(&self, parent_pid: u32, child_pid: u32, uid: u32, gid: u32) -> Result<ProcessEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::ProcessMonitoringFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        // Update process info for child
        {
            let mut processes = self.processes.write();
            
            if processes.len() >= self.max_processes {
                self.evict_oldest(&mut processes);
            }
            
            // Get parent info if available
            let parent_info = processes.get(&parent_pid).cloned();
            
            processes.insert(child_pid, ProcessInfo {
                pid: child_pid,
                ppid: Some(parent_pid),
                uid,
                gid,
                executable: parent_info.and_then(|p| p.executable),
                first_seen: timestamp,
                last_seen: timestamp,
            });
        }
        
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        
        debug!("Process fork: parent={}, child={}", parent_pid, child_pid);
        
        Ok(ProcessEvent {
            event_type: ProcessEventType::Fork,
            pid: child_pid,
            ppid: Some(parent_pid),
            uid,
            gid,
            executable: None,
            command_line: None,
            timestamp,
            mmap_address: None,
            mmap_size: None,
        })
    }
    
    /// Record mmap event
    pub fn record_mmap(&self, pid: u32, address: u64, size: u64) -> Result<ProcessEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::ProcessMonitoringFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        // Update process info
        {
            let mut processes = self.processes.write();
            if let Some(process) = processes.get_mut(&pid) {
                process.last_seen = timestamp;
            }
        }
        
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        
        debug!("Process mmap: pid={}, address=0x{:x}, size={}", pid, address, size);
        
        Ok(ProcessEvent {
            event_type: ProcessEventType::Mmap,
            pid,
            ppid: None,
            uid: 0,
            gid: 0,
            executable: None,
            command_line: None,
            timestamp,
            mmap_address: Some(address),
            mmap_size: Some(size),
        })
    }
    
    /// Evict oldest processes (bounded memory)
    fn evict_oldest(&self, processes: &mut HashMap<u32, ProcessInfo>) {
        let target_size = (self.max_processes as f64 * 0.8) as usize;
        
        if processes.len() <= target_size {
            return;
        }
        
        // Collect processes sorted by last_seen
        let mut process_vec: Vec<(u32, u64)> = processes.iter()
            .map(|(pid, info)| (*pid, info.last_seen))
            .collect();
        
        process_vec.sort_by_key(|(_, ts)| *ts);
        
        // Evict oldest
        let to_evict = processes.len() - target_size;
        for (pid, _) in process_vec.iter().take(to_evict) {
            processes.remove(pid);
        }
        
        debug!("Evicted {} processes, current size: {}", to_evict, processes.len());
    }
    
    /// Get process count
    pub fn process_count(&self) -> usize {
        self.processes.read().len()
    }
    
    /// Get events processed
    pub fn events_processed(&self) -> u64 {
        self.events_processed.load(Ordering::Relaxed)
    }
}

