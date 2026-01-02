// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/src/auth_activity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Authentication activity monitoring - observes login/logout/sudo events (NO enforcement)

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::Utc;
use tracing::{warn, debug, error};
use crossbeam_channel::Sender;
use crate::event::AuthEvent;

/// Monitors authentication events from system logs
/// OBSERVATION ONLY - never enforces, never blocks authentication
pub struct AuthActivityMonitor;

impl AuthActivityMonitor {
    /// Monitor authentication events from system logs
    pub async fn monitor(
        running: Arc<AtomicBool>,
        log_paths: Vec<PathBuf>,
        event_tx: Sender<AuthEvent>,
    ) {
        // Monitor auth.log, secure, and wtmp
        let mut monitors = vec![];
        
        for log_path in log_paths {
            if log_path.exists() {
                let path = log_path.clone();
                let tx = event_tx.clone();
                let run = running.clone();
                
                let handle = tokio::spawn(async move {
                    Self::monitor_log_file(run, path, tx).await;
                });
                
                monitors.push(handle);
            }
        }
        
        // Wait for all monitors
        for monitor in monitors {
            let _ = monitor.await;
        }
    }
    
    /// Monitor a single log file for auth events
    async fn monitor_log_file(
        running: Arc<AtomicBool>,
        log_path: PathBuf,
        event_tx: Sender<AuthEvent>,
    ) {
        // Try to read log file and tail it
        // For simplicity, we'll do periodic scans
        // In production, this would use inotify to tail the file
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        let mut last_position = 0u64;
        
        while running.load(Ordering::Relaxed) {
            interval.tick().await;
            
            if let Ok(file) = File::open(&log_path) {
                let metadata = match file.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                
                let current_size = metadata.len();
                
                if current_size > last_position {
                    // Read new lines
                    if let Ok(file) = File::open(&log_path) {
                        let reader = BufReader::new(file);
                        // Skip to last position (simplified - in production use seek)
                        for line in reader.lines().skip(last_position as usize) {
                            if let Ok(line) = line {
                                if let Some(event) = Self::parse_auth_line(&line, &log_path) {
                                    if event_tx.try_send(event).is_err() {
                                        warn!("Auth event queue full, dropping event");
                                    }
                                }
                            }
                        }
                    }
                    
                    last_position = current_size;
                }
            }
        }
    }
    
    /// Parse an auth log line and extract auth event
    fn parse_auth_line(line: &str, source: &PathBuf) -> Option<AuthEvent> {
        let source_str = source.to_string_lossy().to_string();
        let lower_line = line.to_lowercase();
        
        // Parse login events
        if lower_line.contains("login") || lower_line.contains("sshd") {
            let user = Self::extract_user_from_line(line);
            let success = !lower_line.contains("failed") && !lower_line.contains("invalid");
            
            return Some(AuthEvent {
                event_type: "login".to_string(),
                user,
                source: source_str,
                success,
                timestamp: Utc::now(),
            });
        }
        
        // Parse sudo events
        if lower_line.contains("sudo") {
            let user = Self::extract_user_from_line(line);
            let success = !lower_line.contains("incorrect password");
            
            return Some(AuthEvent {
                event_type: "sudo".to_string(),
                user,
                source: source_str,
                success,
                timestamp: Utc::now(),
            });
        }
        
        // Parse logout events
        if lower_line.contains("logout") || lower_line.contains("session closed") {
            let user = Self::extract_user_from_line(line);
            
            return Some(AuthEvent {
                event_type: "logout".to_string(),
                user,
                source: source_str,
                success: true,
                timestamp: Utc::now(),
            });
        }
        
        None
    }
    
    /// Extract username from log line (simplified parser)
    fn extract_user_from_line(line: &str) -> String {
        // Try to find user= or username pattern
        if let Some(user_start) = line.find("user=") {
            let after_user = &line[user_start + 5..];
            if let Some(end) = after_user.find(|c: char| c == ' ' || c == '\t' || c == ':') {
                return after_user[..end].to_string();
            }
            return after_user.trim().to_string();
        }
        
        // Try to find common patterns
        for word in line.split_whitespace() {
            if word.starts_with("for") && word.len() > 4 {
                return word[4..].to_string();
            }
        }
        
        "unknown".to_string()
    }
}
