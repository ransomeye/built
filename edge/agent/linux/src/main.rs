// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main entry point for Linux Agent - host telemetry collection

/*
 * RansomEye Linux Agent
 * 
 * Stand-alone sensor for host telemetry collection.
 * UNTRUSTED component - never enforces policy, never runs AI, never stores long-term state.
 * All telemetry is signed and sent to Control Plane via mTLS.
 */

use std::sync::Arc;
use tokio::signal;
use tracing::{info, error, warn};
use crossbeam_channel::bounded;

mod process;
mod file_activity;
mod auth_activity;
mod network_activity;
mod event;
mod signing;
mod transport;
mod backpressure;
mod buffer;
mod health;
mod config;
mod identity;
mod deception;
mod system_metrics;

use config::Config;
use std::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting RansomEye Linux Agent");
    
    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded");
    
    // Initialize identity
    let identity = Arc::new(identity::Identity::load_or_create(&config)?);
    info!("Identity initialized: {}", identity.producer_id());
    
    // Create event channels
    let (event_tx, event_rx) = bounded::<event::AgentEvent>(10000);
    
    // Initialize components
    let backpressure = Arc::new(backpressure::BackpressureHandler::new(
        config.max_buffer_size_mb * 1024 * 1024,
        config.backpressure_threshold,
    ));
    let signer = Arc::new(signing::EventSigner::new(
        identity.keypair(),
        identity.producer_id().to_string(),
    ));
    let transport = Arc::new(transport::TransportClient::new(config.clone(), backpressure.clone())?);
    let disk_buffer = Arc::new(buffer::DiskBuffer::new(&config.buffer_dir, config.max_buffer_size_mb)?);
    
    // Initialize system metrics collector and shared state
    let mut metrics_collector = system_metrics::SystemMetricsCollector::new();
    let current_system_metrics = Arc::new(Mutex::new(None::<system_metrics::SystemMetrics>));
    
    // Start monitoring tasks
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    
    // Start system metrics collection task (collect every 20 seconds)
    let metrics_handle = {
        let run = running.clone();
        let metrics_state = current_system_metrics.clone();
        let mut collector = metrics_collector;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(20));
            while run.load(std::sync::atomic::Ordering::Relaxed) {
                interval.tick().await;
                let metrics = collector.collect();
                if metrics.has_real_metrics() {
                    if let Ok(mut state) = metrics_state.lock() {
                        *state = Some(metrics);
                    }
                } else {
                    warn!("[SYS_METRICS][WARN] collected metrics but all values are null — discarding");
                }
            }
        })
    };
    
    let process_handle = {
        let run = running.clone();
        let tx = event_tx.clone();
        tokio::spawn(async move {
            process::ProcessMonitor::monitor(run, tx, 5).await;
        })
    };
    
    let file_handle = {
        let run = running.clone();
        let tx = event_tx.clone();
        let paths = config.monitor_paths.clone();
        tokio::spawn(async move {
            file_activity::FileActivityMonitor::monitor(run, paths, tx).await;
        })
    };
    
    let auth_handle = {
        let run = running.clone();
        let tx = event_tx.clone();
        let log_paths = config.auth_log_paths.clone();
        tokio::spawn(async move {
            auth_activity::AuthActivityMonitor::monitor(run, log_paths, tx).await;
        })
    };
    
    let network_handle = {
        let run = running.clone();
        let tx = event_tx.clone();
        tokio::spawn(async move {
            network_activity::NetworkActivityMonitor::monitor(run, tx, 10).await;
        })
    };
    
    // Start event processing loop
    let process_loop_handle = {
        let run = running.clone();
        let signer = signer.clone();
        let transport = transport.clone();
        let disk_buffer = disk_buffer.clone();
        let metrics_state = current_system_metrics.clone();
        tokio::spawn(async move {
            process_loop(run, event_rx, signer, transport, disk_buffer, metrics_state).await;
        })
    };
    
    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received");
    
    // Graceful shutdown
    running.store(false, std::sync::atomic::Ordering::Relaxed);
    
    // Wait for tasks (with timeout)
    tokio::select! {
        _ = process_handle => {}
        _ = file_handle => {}
        _ = auth_handle => {}
        _ = network_handle => {}
        _ = metrics_handle => {}
        _ = process_loop_handle => {}
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
            info!("Shutdown timeout reached");
        }
    }
    
    info!("RansomEye Linux Agent stopped");
    Ok(())
}

async fn process_loop(
    running: Arc<std::sync::atomic::AtomicBool>,
    event_rx: crossbeam_channel::Receiver<event::AgentEvent>,
    signer: Arc<signing::EventSigner>,
    transport: Arc<transport::TransportClient>,
    disk_buffer: Arc<buffer::DiskBuffer>,
    metrics_state: Arc<Mutex<Option<system_metrics::SystemMetrics>>>,
) {
    while running.load(std::sync::atomic::Ordering::Relaxed) {
        // Receive event
        let event = match event_rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        // Convert to JSON
        let mut event_json = event.to_json_value();
        
        // Include system metrics in payload if available and valid
        if let Ok(metrics_guard) = metrics_state.lock() {
            if let Some(ref metrics) = *metrics_guard {
                if metrics.has_real_metrics() {
                    if let Some(event_obj) = event_json.as_object_mut() {
                        let system_json = serde_json::to_value(metrics)
                            .unwrap_or_else(|_| serde_json::json!({}));
                        event_obj.insert("system".to_string(), system_json);
                    }
                } else {
                    warn!("[ENVELOPE][WARN] system metrics present but invalid (all null), injecting empty object");
                    if let Some(event_obj) = event_json.as_object_mut() {
                        event_obj.insert("system".to_string(), serde_json::json!({}));
                    }
                }
            }
        }
        
        // Sign event
        let signed_event = match signer.sign_event(event_json) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to sign event: {}", e);
                continue;
            }
        };
        
        // Send to Core (with retry on backpressure or buffer to disk if Core unavailable)
        let mut retry_count = 0;
        let max_retries = 3;
        loop {
            match transport.send_event(&signed_event).await {
                Ok(_) => {
                    // Successfully sent - try to flush any buffered events
                    try_flush_buffer(&disk_buffer, &transport).await;
                    break;
                }
                Err(transport::TransportError::Backpressure) => {
                    // Backpressure - buffer to disk and retry
                    if let Err(e) = disk_buffer.write_event(&signed_event) {
                        error!("Failed to buffer event to disk: {}", e);
                    }
                    retry_count += 1;
                    if retry_count >= max_retries {
                        break;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(100 * retry_count)).await;
                }
                Err(e) => {
                    // Core unavailable - buffer to disk
                    error!("Core unavailable, buffering event to disk: {}", e);
                    if let Err(e) = disk_buffer.write_event(&signed_event) {
                        error!("Failed to buffer event to disk: {}", e);
                    }
                    break;
                }
            }
        }
    }
}

async fn try_flush_buffer(
    disk_buffer: &buffer::DiskBuffer,
    transport: &transport::TransportClient,
) {
    // Try to flush buffered events when Core is available
    let mut flushed = 0;
    while let Ok(Some(event)) = disk_buffer.read_oldest_event() {
        match transport.send_event(&event).await {
            Ok(_) => {
                if let Err(e) = disk_buffer.remove_event(&event) {
                    error!("Failed to remove buffered event: {}", e);
                }
                flushed += 1;
                if flushed >= 10 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    flushed = 0;
                }
            }
            Err(_) => {
                // Core unavailable again, stop flushing
                break;
            }
        }
    }
}