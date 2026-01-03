// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/agent/src/process.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Process monitoring - process create/terminate, command line

#![cfg(feature = "future-windows-agent")]

#[cfg(windows)]
use winapi::um::processthreadsapi::*;
#[cfg(windows)]
use winapi::um::tlhelp32::*;
#[cfg(windows)]
use winapi::um::winbase::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use parking_lot::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

use super::errors::AgentError;

/// Process event types
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessEventType {
    Create,
    Terminate,
    CommandLine,
}

/// Process event
#[derive(Debug, Clone)]
pub struct ProcessEvent {
    pub event_type: ProcessEventType,
    pub pid: u32,
    pub ppid: Option<u32>,
    pub executable: Option<String>,
    pub command_line: Option<String>,
    pub timestamp: u64,
}

/// Process monitor
/// 
/// Tracks process events: create, terminate, command line.
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
    executable: String,
    command_line: Option<String>,
    created_at: u64,
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
    
    /// Get process create event
    /// 
    /// Monitors process creation via ETW or WMI fallback.
    pub fn get_process_create(&self, pid: u32) -> Result<Option<ProcessEvent>, AgentError> {
        #[cfg(windows)]
        {
            // In real implementation, this would be called from ETW callback
            // For now, we provide the structure
            
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| AgentError::ProcessMonitoringFailed(
                    format!("Failed to get timestamp: {}", e)
                ))?
                .as_secs();
            
            // Get process info
            let process_info = self.get_process_info(pid)?;
            
            if let Some(info) = process_info {
                // Check if this is a new process
                let mut processes = self.processes.write();
                
                if !processes.contains_key(&pid) {
                    // New process
                    processes.insert(pid, info.clone());
                    
                    // Enforce memory bounds
                    if processes.len() > self.max_processes {
                        self.evict_oldest_processes(&mut processes);
                    }
                    
                    self.events_processed.fetch_add(1, Ordering::AcqRel);
                    
                    return Ok(Some(ProcessEvent {
                        event_type: ProcessEventType::Create,
                        pid,
                        ppid: info.ppid,
                        executable: Some(info.executable.clone()),
                        command_line: info.command_line.clone(),
                        timestamp,
                    }));
                }
            }
            
            Ok(None)
        }
        
        #[cfg(not(windows))]
        {
            Err(AgentError::ProcessMonitoringFailed(
                "Process monitoring is only available on Windows".to_string()
            ))
        }
    }
    
    /// Get process terminate event
    pub fn get_process_terminate(&self, pid: u32) -> Result<Option<ProcessEvent>, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::ProcessMonitoringFailed(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        let mut processes = self.processes.write();
        
        if processes.remove(&pid).is_some() {
            self.events_processed.fetch_add(1, Ordering::AcqRel);
            
            Ok(Some(ProcessEvent {
                event_type: ProcessEventType::Terminate,
                pid,
                ppid: None,
                executable: None,
                command_line: None,
                timestamp,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Get process command line
    pub fn get_process_command_line(&self, pid: u32) -> Result<Option<ProcessEvent>, AgentError> {
        #[cfg(windows)]
        {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| AgentError::ProcessMonitoringFailed(
                    format!("Failed to get timestamp: {}", e)
                ))?
                .as_secs();
            
            let process_info = self.get_process_info(pid)?;
            
            if let Some(info) = process_info {
                if let Some(cmd_line) = &info.command_line {
                    return Ok(Some(ProcessEvent {
                        event_type: ProcessEventType::CommandLine,
                        pid,
                        ppid: info.ppid,
                        executable: Some(info.executable.clone()),
                        command_line: Some(cmd_line.clone()),
                        timestamp,
                    }));
                }
            }
            
            Ok(None)
        }
        
        #[cfg(not(windows))]
        {
            Err(AgentError::ProcessMonitoringFailed(
                "Process monitoring is only available on Windows".to_string()
            ))
        }
    }
    
    /// Get process info (helper)
    #[cfg(windows)]
    fn get_process_info(&self, pid: u32) -> Result<Option<ProcessInfo>, AgentError> {
        // In real implementation, this would query process info via Windows APIs
        // For now, return None to indicate process not found
        Ok(None)
    }
    
    /// Evict oldest processes to maintain memory bounds
    fn evict_oldest_processes(&self, processes: &mut HashMap<u32, ProcessInfo>) {
        if processes.len() <= self.max_processes {
            return;
        }
        
        let mut sorted: Vec<_> = processes.iter().collect();
        sorted.sort_by_key(|(_, info)| info.created_at);
        
        let to_remove = processes.len() - self.max_processes;
        // Collect keys first to avoid holding an immutable borrow of `processes`
        // while mutating it (Rust borrow rules).
        let pids_to_remove: Vec<u32> = sorted
            .iter()
            .take(to_remove)
            .map(|(pid, _)| **pid)
            .collect();
        for pid in pids_to_remove {
            processes.remove(&pid);
        }
        
        debug!("Evicted {} oldest processes", to_remove);
    }
    
    /// Get events processed count
    pub fn events_processed(&self) -> u64 {
        self.events_processed.load(Ordering::Acquire)
    }
}

