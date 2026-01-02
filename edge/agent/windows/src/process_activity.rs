// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/src/process_activity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Process activity monitoring on Windows using WMI/Toolhelp32 - observes process creation/termination (NO enforcement, NO blocking)

#[cfg(windows)]
use std::sync::Arc;
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use chrono::Utc;
#[cfg(windows)]
use tracing::{warn, debug, error};
#[cfg(windows)]
use crossbeam_channel::Sender;
#[cfg(windows)]
use crate::event::ProcessEvent;

/// Monitors process creation and termination events on Windows
/// OBSERVATION ONLY - never blocks, never enforces, never modifies
#[cfg(windows)]
pub struct ProcessActivityMonitor;

#[cfg(windows)]
impl ProcessActivityMonitor {
    /// Monitor processes and send events to channel
    /// Uses Windows Toolhelp32 API for process enumeration
    pub async fn monitor(
        running: Arc<AtomicBool>,
        event_tx: Sender<ProcessEvent>,
        scan_interval_secs: u64,
    ) {
        let mut last_pids = std::collections::HashSet::new();
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(scan_interval_secs));
        
        while running.load(Ordering::Relaxed) {
            interval.tick().await;
            
            let current_pids = Self::scan_processes();
            
            // Detect new processes
            for pid in &current_pids {
                if !last_pids.contains(pid) {
                    if let Some(event) = Self::read_process_info(*pid) {
                        if event_tx.try_send(event).is_err() {
                            warn!("Process event queue full, dropping event");
                        }
                    }
                }
            }
            
            // Detect terminated processes
            for pid in &last_pids {
                if !current_pids.contains(pid) {
                    let event = ProcessEvent {
                        event_type: "process_terminated".to_string(),
                        pid: *pid,
                        ppid: 0,
                        process_name: "unknown".to_string(),
                        command_line: String::new(),
                        user_sid: String::new(),
                        timestamp: Utc::now(),
                    };
                    
                    if event_tx.try_send(event).is_err() {
                        warn!("Process event queue full, dropping termination event");
                    }
                }
            }
            
            last_pids = current_pids;
        }
    }
    
    /// Scan processes using Windows Toolhelp32 API
    fn scan_processes() -> std::collections::HashSet<u32> {
        use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS};
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        
        let mut pids = std::collections::HashSet::new();
        
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return pids;
            }
            
            let mut entry: PROCESSENTRY32 = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
            
            if Process32First(snapshot, &mut entry) != 0 {
                loop {
                    pids.insert(entry.th32ProcessID);
                    
                    if Process32Next(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }
            
            winapi::um::handleapi::CloseHandle(snapshot);
        }
        
        pids
    }
    
    /// Read process information using Windows APIs
    /// Returns None if process doesn't exist or info cannot be read
    fn read_process_info(pid: u32) -> Option<ProcessEvent> {
        use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS};
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;
        
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return None;
            }
            
            let mut entry: PROCESSENTRY32 = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
            
            if Process32First(snapshot, &mut entry) != 0 {
                loop {
                    if entry.th32ProcessID == pid {
                        // Found the process
                        let process_name = OsStr::from_bytes(
                            &entry.szExeFile[..entry.szExeFile.iter().position(|&x| x == 0).unwrap_or(entry.szExeFile.len())]
                        ).to_string_lossy().to_string();
                        
                        // Get command line (simplified - would use NtQueryInformationProcess in production)
                        let command_line = process_name.clone();
                        
                        // Get user SID (simplified - would extract from process token)
                        let user_sid = "unknown".to_string();
                        
                        winapi::um::handleapi::CloseHandle(snapshot);
                        
                        return Some(ProcessEvent {
                            event_type: "process_created".to_string(),
                            pid,
                            ppid: entry.th32ParentProcessID,
                            process_name,
                            command_line,
                            user_sid,
                            timestamp: Utc::now(),
                        });
                    }
                    
                    if Process32Next(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }
            
            winapi::um::handleapi::CloseHandle(snapshot);
        }
        
        None
    }
}

#[cfg(not(windows))]
pub struct ProcessActivityMonitor;

#[cfg(not(windows))]
impl ProcessActivityMonitor {
    pub async fn monitor(
        _running: std::sync::Arc<std::sync::atomic::AtomicBool>,
        _event_tx: crossbeam_channel::Sender<crate::event::ProcessEvent>,
        _scan_interval_secs: u64,
    ) {
        // Placeholder for non-Windows builds
    }
}
