// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/src/telemetry.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Telemetry collector - collects process, registry, and file activity telemetry (user-mode only)

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task;
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
    pub pid: u32,
    pub ppid: u32,
    pub process_name: String,
    pub command_line: String,
    pub user_sid: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEvent {
    pub event_type: String,
    pub key_path: String,
    pub operation: String,
    pub value_name: String,
    pub process_id: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    pub event_type: String,
    pub path: String,
    pub operation: String,
    pub process_id: u32,
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
        info!("Starting Windows Agent telemetry collector");
        
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
        
        // Start registry monitoring (placeholder - would use ETW in production)
        let registry_handle = task::spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        });
        
        // Start file monitoring (placeholder - would use ETW in production)
        let file_handle = task::spawn(async {
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
            _ = registry_handle => {
                error!("Registry monitoring task exited");
            }
            _ = file_handle => {
                error!("File monitoring task exited");
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
        
        #[cfg(windows)]
        {
            use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, TH32CS_SNAPPROCESS, PROCESSENTRY32};
            use winapi::um::processthreadsapi::OpenProcess;
            use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
            use winapi::um::winbase::QueryFullProcessImageNameA;
            use std::ffi::CString;
            use std::os::raw::c_char;
            
            while running.load(Ordering::Relaxed) {
                interval.tick().await;
                
                unsafe {
                    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
                    if snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
                        continue;
                    }
                    
                    let mut entry: PROCESSENTRY32 = std::mem::zeroed();
                    entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
                    
                    if Process32First(snapshot, &mut entry) != 0 {
                        loop {
                            let process_name = String::from_utf8_lossy(
                                std::slice::from_raw_parts(
                                    entry.szExeFile.as_ptr() as *const u8,
                                    entry.szExeFile.iter().position(|&x| x == 0).unwrap_or(entry.szExeFile.len())
                                )
                            ).to_string();
                            
                            // Get command line (simplified - would use NtQueryInformationProcess in production)
                            let command_line = format!("{}", process_name);
                            
                            let process_event = ProcessEvent {
                                event_type: "process_created".to_string(),
                                pid: entry.th32ProcessID,
                                ppid: entry.th32ParentProcessID,
                                process_name,
                                command_line,
                                user_sid: "unknown".to_string(), // Would extract from process token
                                timestamp: chrono::Utc::now(),
                            };
                            
                            let event_json = serde_json::json!({
                                "event_type": "process",
                                "data": process_event,
                            });
                            
                            if event_tx.try_send(event_json).is_err() {
                                warn!("Event queue full, dropping process event");
                            }
                            
                            if Process32Next(snapshot, &mut entry) == 0 {
                                break;
                            }
                        }
                    }
                    
                    winapi::um::handleapi::CloseHandle(snapshot);
                }
            }
        }
        
        #[cfg(not(windows))]
        {
            // Non-Windows: placeholder
            while running.load(Ordering::Relaxed) {
                interval.tick().await;
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
        info!("Windows Agent telemetry collector shutdown");
    }
}

