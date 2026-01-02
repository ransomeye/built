// Path and File Name : /home/ransomeye/rebuild/edge/sentinel/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Sentinel main entry point - monitors Agent and DPI health, enforces runtime integrity

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::{info, error, warn};
use tokio::time;

mod errors;
mod hardening;
mod monitor;
mod alert;

use errors::SentinelError;
use hardening::RuntimeHardening;
use monitor::{AgentMonitor, DpiMonitor, ComponentHealth};
use alert::AlertEmitter;

#[tokio::main]
async fn main() -> Result<(), SentinelError> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("RansomEye Sentinel starting...");
    
    // Get binary path for integrity verification
    let binary_path = std::env::current_exe()
        .map_err(|e| SentinelError::ConfigurationError(format!("Failed to get binary path: {}", e)))?
        .to_string_lossy()
        .to_string();
    
    // Initialize runtime hardening (FAIL-CLOSED on integrity failure)
    let config_path = std::env::var("SENTINEL_CONFIG_PATH").ok();
    let hardening = RuntimeHardening::new(
        binary_path.clone(),
        config_path.clone(),
        30, // 30 second watchdog interval
    ).map_err(|e| SentinelError::ConfigurationError(format!("Hardening initialization failed: {}", e)))?;
    
    // Verify binary integrity at startup (FAIL-CLOSED)
    hardening.verify_binary_integrity()
        .map_err(|e| SentinelError::ConfigurationError(format!("Binary integrity check failed: {}", e)))?;
    
    // Verify config integrity at startup (FAIL-CLOSED)
    hardening.verify_config_integrity()
        .map_err(|e| SentinelError::ConfigurationError(format!("Config integrity check failed: {}", e)))?;
    
    // Perform runtime tamper checks (FAIL-CLOSED)
    hardening.perform_runtime_checks()
        .map_err(|e| SentinelError::ConfigurationError(format!("Runtime check failed: {}", e)))?;
    
    // Start watchdog timer
    hardening.start_watchdog()
        .map_err(|e| SentinelError::ConfigurationError(format!("Watchdog start failed: {}", e)))?;
    
    info!("Sentinel hardening initialized");
    
    // Initialize component monitors
    let agent_monitor = Arc::new(AgentMonitor::new(
        std::env::var("AGENT_SERVICE_NAME").unwrap_or_else(|_| "ransomeye-linux-agent.service".to_string()),
    ));
    
    let dpi_monitor = Arc::new(DpiMonitor::new(
        std::env::var("DPI_SERVICE_NAME").unwrap_or_else(|_| "ransomeye-dpi-probe.service".to_string()),
    ));
    
    // Initialize alert emitter
    let alert_emitter = Arc::new(AlertEmitter::new(
        std::env::var("CORE_API_URL").ok(),
    ));
    
    info!("Sentinel monitors initialized");
    info!("Monitoring Agent: {}", agent_monitor.service_name());
    info!("Monitoring DPI: {}", dpi_monitor.service_name());
    
    // Main monitoring loop
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    // Handle shutdown signal
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Shutdown signal received");
        running_clone.store(false, Ordering::Release);
    });
    
    let mut interval = time::interval(Duration::from_secs(10)); // Check every 10 seconds
    let mut iteration_count = 0u64;
    
    while running.load(Ordering::Acquire) {
        interval.tick().await;
        iteration_count += 1;
        
        // Record watchdog heartbeat
        hardening.heartbeat();
        
        // Perform periodic runtime checks (every 10 iterations = 100 seconds)
        if iteration_count % 10 == 0 {
            if let Err(e) = hardening.perform_runtime_checks() {
                error!("Runtime check failed: {}, stopping", e);
                hardening.stop_watchdog();
                return Err(SentinelError::ConfigurationError(format!("Runtime hardening violation: {}", e)));
            }
            
            // Check for tamper detection
            if hardening.is_tampered() {
                error!("Tamper detected, stopping immediately");
                hardening.stop_watchdog();
                return Err(SentinelError::ConfigurationError("Tamper detected - fail-closed".to_string()));
            }
        }
        
        // Check Agent health
        match agent_monitor.check_health().await {
            Ok(ComponentHealth::Healthy) => {
                // Agent is healthy
            }
            Ok(ComponentHealth::Unhealthy) => {
                warn!("Agent health check failed");
                alert_emitter.emit_agent_unhealthy().await;
            }
            Ok(ComponentHealth::Terminated) => {
                error!("Agent terminated unexpectedly");
                alert_emitter.emit_agent_terminated().await;
            }
            Err(e) => {
                error!("Agent health check error: {}", e);
                alert_emitter.emit_agent_error(e.to_string()).await;
            }
        }
        
        // Check DPI health
        match dpi_monitor.check_health().await {
            Ok(ComponentHealth::Healthy) => {
                // DPI is healthy
            }
            Ok(ComponentHealth::Unhealthy) => {
                warn!("DPI health check failed");
                alert_emitter.emit_dpi_unhealthy().await;
            }
            Ok(ComponentHealth::Terminated) => {
                error!("DPI terminated unexpectedly");
                alert_emitter.emit_dpi_terminated().await;
            }
            Err(e) => {
                error!("DPI health check error: {}", e);
                alert_emitter.emit_dpi_error(e.to_string()).await;
            }
        }
        
        // Check for binary tampering on monitored components
        if let Err(e) = agent_monitor.check_binary_integrity().await {
            error!("Agent binary integrity check failed: {}", e);
            alert_emitter.emit_agent_tamper(e.to_string()).await;
        }
        
        if let Err(e) = dpi_monitor.check_binary_integrity().await {
            error!("DPI binary integrity check failed: {}", e);
            alert_emitter.emit_dpi_tamper(e.to_string()).await;
        }
    }
    
    hardening.stop_watchdog();
    info!("Sentinel stopped");
    Ok(())
}

