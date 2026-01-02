// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/src/telemetry.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Telemetry collector - collects process, file, and auth activity telemetry

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::Path;
use std::fs;
use tokio::task;
use inotify::{Inotify, WatchMask};
use nix::sys::ptrace;
use nix::unistd::Pid;
use tracing::{error, warn, debug, info};
use crossbeam_channel::{bounded, Receiver, Sender};
use serde::{Serialize, Deserialize};
use crate::config::Config;
use crate::identity::Identity;
use crate::signing::{EventSigner, SignedEvent};
use crate::transport::TransportClient;
use crate::backpressure::BackpressureHandler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub event_type: String,
    pub pid: i32,
    pub ppid: i32,
    pub process_name: String,
    pub command_line: String,
    pub user_id: u32,
    pub group_id: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    pub event_type: String,
    pub path: String,
    pub operation: String,
    pub user_id: u32,
    pub process_id: i32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEvent {
    pub event_type: String,
    pub user: String,
    pub source: String,
    pub success: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct TelemetryCollector {
    config: Config,
    identity: Arc<Identity>,
    running: Arc<AtomicBool>,
    event_tx: Option<Sender<serde_json::Value>>,
    event_rx: Option<Receiver<serde_json::Value>>,
}

impl TelemetryCollector {
    pub async fn new(config: Config, identity: Arc<Identity>) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let (tx, rx) = bounded::<serde_json::Value>(10000);
        
        Ok(Arc::new(Self {
            config,
            identity,
            running: Arc::new(AtomicBool::new(false)),
            event_tx: Some(tx),
            event_rx: Some(rx),
        }))
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.running.store(true, Ordering::Relaxed);
        info!("Starting Linux Agent telemetry collector");
        
        // Initialize components
        let backpressure = Arc::new(BackpressureHandler::new(
            self.config.max_buffer_size_mb * 1024 * 1024,
            self.config.backpressure_threshold,
        ));
        let signer = Arc::new(EventSigner::new(
            self.identity.keypair(),
            self.identity.producer_id().to_string(),
        ));
        let transport = Arc::new(TransportClient::new(self.config.clone(), backpressure.clone())?);
        
        // Start process monitoring
        let process_handle = {
            let running = self.running.clone();
            let event_tx = self.event_tx.take().unwrap();
            
            task::spawn(async move {
                Self::monitor_processes(running, event_tx).await;
            })
        };
        
        // Note: File and auth monitoring would need separate channels or shared sender
        // For now, we'll focus on process monitoring
        let file_handle = task::spawn(async {
            // Placeholder - file monitoring would be implemented here
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        });
        
        let auth_handle = task::spawn(async {
            // Placeholder - auth monitoring would be implemented here
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        });
        
        // Start processing loop
        let process_handle2 = {
            let running = self.running.clone();
            let event_rx = self.event_rx.take().unwrap();
            let signer = signer.clone();
            let transport = transport.clone();
            let backpressure = backpressure.clone();
            
            task::spawn(async move {
                Self::process_loop(
                    running,
                    event_rx,
                    signer,
                    transport,
                    backpressure,
                ).await;
            })
        };
        
        // Wait for tasks
        tokio::select! {
            _ = process_handle => {
                error!("Process monitoring task exited");
            }
            _ = file_handle => {
                error!("File monitoring task exited");
            }
            _ = auth_handle => {
                error!("Auth monitoring task exited");
            }
            _ = process_handle2 => {
                error!("Processing task exited");
            }
        }
        
        Ok(())
    }
    
    async fn monitor_processes(
        running: Arc<AtomicBool>,
        event_tx: Sender<serde_json::Value>,
    ) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        
        while running.load(Ordering::Relaxed) {
            interval.tick().await;
            
            // Scan /proc for process events
            if let Ok(entries) = fs::read_dir("/proc") {
                for entry in entries.flatten() {
                    if let Ok(pid_str) = entry.file_name().into_string() {
                        if let Ok(pid) = pid_str.parse::<i32>() {
                            // Read process info
                            if let Some(event) = Self::read_process_info(pid) {
                                let event_json = serde_json::json!({
                                    "event_type": "process",
                                    "data": event,
                                });
                                
                                if event_tx.try_send(event_json).is_err() {
                                    warn!("Event queue full, dropping process event");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn read_process_info(pid: i32) -> Option<ProcessEvent> {
        // Read /proc/{pid}/stat
        let stat_path = format!("/proc/{}/stat", pid);
        let stat_content = fs::read_to_string(&stat_path).ok()?;
        let parts: Vec<&str> = stat_content.split_whitespace().collect();
        
        if parts.len() < 4 {
            return None;
        }
        
        let process_name = parts[1].trim_matches('(').trim_matches(')').to_string();
        let ppid: i32 = parts[3].parse().ok()?;
        
        // Read /proc/{pid}/status for UID/GID
        let status_path = format!("/proc/{}/status", pid);
        let status_content = fs::read_to_string(&status_path).ok()?;
        let mut uid = 0;
        let mut gid = 0;
        
        for line in status_content.lines() {
            if line.starts_with("Uid:") {
                uid = line.split_whitespace().nth(1)?.parse().ok()?;
            }
            if line.starts_with("Gid:") {
                gid = line.split_whitespace().nth(1)?.parse().ok()?;
            }
        }
        
        // Read command line
        let cmdline_path = format!("/proc/{}/cmdline", pid);
        let command_line = fs::read_to_string(&cmdline_path)
            .unwrap_or_default()
            .replace('\0', " ");
        
        Some(ProcessEvent {
            event_type: "process_created".to_string(),
            pid,
            ppid,
            process_name,
            command_line,
            user_id: uid,
            group_id: gid,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn monitor_files(
        running: Arc<AtomicBool>,
        config: Config,
        event_tx: Sender<serde_json::Value>,
    ) {
        let mut inotify = Inotify::init().expect("Failed to initialize inotify");
        
        // Watch configured paths
        for path_str in &config.monitor_paths {
            if let Ok(path) = Path::new(path_str).canonicalize() {
                if path.exists() {
                    inotify.add_watch(&path, WatchMask::CREATE | WatchMask::MODIFY | WatchMask::DELETE)
                        .unwrap_or_else(|e| {
                            warn!("Failed to watch {}: {}", path_str, e);
                            continue;
                        });
                }
            }
        }
        
        let mut buffer = [0u8; 4096];
        while running.load(Ordering::Relaxed) {
            let events = inotify.read_events_blocking(&mut buffer)
                .unwrap_or_default();
            
            for event in events {
                let path = event.name.map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                
                let file_event = FileEvent {
                    event_type: "file_event".to_string(),
                    path,
                    operation: format!("{:?}", event.mask),
                    user_id: unsafe { libc::getuid() },
                    process_id: unsafe { libc::getpid() },
                    timestamp: chrono::Utc::now(),
                };
                
                let event_json = serde_json::json!({
                    "event_type": "file",
                    "data": file_event,
                });
                
                if event_tx.try_send(event_json).is_err() {
                    warn!("Event queue full, dropping file event");
                }
            }
        }
    }
    
    async fn monitor_auth(
        running: Arc<AtomicBool>,
        event_tx: Sender<serde_json::Value>,
    ) {
        // Monitor /var/log/auth.log or /var/log/secure
        let auth_logs = vec!["/var/log/auth.log", "/var/log/secure"];
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        
        while running.load(Ordering::Relaxed) {
            interval.tick().await;
            
            for log_path in &auth_logs {
                if let Ok(content) = fs::read_to_string(log_path) {
                    for line in content.lines().rev().take(10) {
                        // Parse auth log entries (simplified)
                        if line.contains("sshd") || line.contains("login") || line.contains("su") {
                            let auth_event = AuthEvent {
                                event_type: "auth_event".to_string(),
                                user: "unknown".to_string(),
                                source: log_path.to_string(),
                                success: !line.contains("Failed"),
                                timestamp: chrono::Utc::now(),
                            };
                            
                            let event_json = serde_json::json!({
                                "event_type": "auth",
                                "data": auth_event,
                            });
                            
                            if event_tx.try_send(event_json).is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    async fn process_loop(
        running: Arc<AtomicBool>,
        event_rx: Receiver<serde_json::Value>,
        signer: Arc<EventSigner>,
        transport: Arc<TransportClient>,
        backpressure: Arc<BackpressureHandler>,
    ) {
        while running.load(Ordering::Relaxed) {
            // Receive event
            let event_data = match event_rx.recv() {
                Ok(data) => data,
                Err(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    continue;
                }
            };
            
            // Sign event
            let signed_event = match signer.sign_event(event_data) {
                Ok(e) => e,
                Err(e) => {
                    error!("Failed to sign event: {}", e);
                    continue;
                }
            };
            
            // Send to Core (with retry on backpressure)
            loop {
                match transport.send_event(&signed_event).await {
                    Ok(_) => break,
                    Err(crate::transport::TransportError::Backpressure) => {
                        // Wait and retry
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        continue;
                    }
                    Err(e) => {
                        error!("Failed to send event: {}", e);
                        break;
                    }
                }
            }
        }
    }
    
    pub async fn shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
        info!("Linux Agent telemetry collector shutdown");
    }
}

