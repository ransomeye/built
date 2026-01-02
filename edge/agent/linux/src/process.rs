// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/src/process.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Process activity monitoring - observes process creation/termination (NO enforcement, NO blocking)

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::Utc;
use tracing::{warn, debug, error};
use crossbeam_channel::Sender;
use crate::event::ProcessEvent;

/// Monitors process creation and termination events
/// OBSERVATION ONLY - never blocks, never enforces, never modifies
pub struct ProcessMonitor;

impl ProcessMonitor {
    /// Monitor processes and send events to channel
    /// This is a user-space observer - reads /proc filesystem
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
                        ppid: 0, // Unknown for terminated processes
                        process_name: "unknown".to_string(),
                        command_line: String::new(),
                        user_id: 0,
                        group_id: 0,
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
    
    /// Scan /proc for current process IDs
    fn scan_processes() -> std::collections::HashSet<i32> {
        let mut pids = std::collections::HashSet::new();
        
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(pid_str) = entry.file_name().into_string() {
                    if let Ok(pid) = pid_str.parse::<i32>() {
                        pids.insert(pid);
                    }
                }
            }
        }
        
        pids
    }
    
    /// Read process information from /proc filesystem
    /// Returns None if process doesn't exist or info cannot be read
    fn read_process_info(pid: i32) -> Option<ProcessEvent> {
        let pid_dir = format!("/proc/{}", pid);
        let pid_path = Path::new(&pid_dir);
        
        if !pid_path.exists() {
            return None;
        }
        
        // Read process name from comm
        let process_name = fs::read_to_string(pid_path.join("comm"))
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();
        
        // Read command line from cmdline
        let command_line = fs::read_to_string(pid_path.join("cmdline"))
            .unwrap_or_else(|_| String::new())
            .replace('\0', " ")
            .trim()
            .to_string();
        
        // Read stat for ppid, uid, gid
        let mut ppid = 0;
        let mut uid = 0;
        let mut gid = 0;
        
        if let Ok(stat_content) = fs::read_to_string(pid_path.join("stat")) {
            let fields: Vec<&str> = stat_content.split_whitespace().collect();
            if fields.len() > 3 {
                ppid = fields[3].parse().unwrap_or(0);
            }
        }
        
        // Read status for uid/gid
        if let Ok(status_content) = fs::read_to_string(pid_path.join("status")) {
            for line in status_content.lines() {
                if line.starts_with("Uid:") {
                    if let Some(uid_str) = line.split_whitespace().nth(1) {
                        uid = uid_str.parse().unwrap_or(0);
                    }
                }
                if line.starts_with("Gid:") {
                    if let Some(gid_str) = line.split_whitespace().nth(1) {
                        gid = gid_str.parse().unwrap_or(0);
                    }
                }
            }
        }
        
        Some(ProcessEvent {
            event_type: "process_created".to_string(),
            pid,
            ppid,
            process_name,
            command_line,
            user_id: uid,
            group_id: gid,
            timestamp: Utc::now(),
        })
    }
}
