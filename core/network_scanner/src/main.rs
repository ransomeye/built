// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main entry point for Phase 9 Network Scanner service

use std::sync::Arc;
use tracing::{info, error};
use ransomeye_network_scanner::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting RansomEye Phase 9: Network Scanner");
    
    // Check configuration
    let active_mode = std::env::var("ACTIVE_MODE_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    let passive_mode = std::env::var("PASSIVE_MODE_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    // Initialize persistence
    let persistence = Arc::new(ScanPersistence::new().await?);
    info!("Scan persistence initialized");
    
    // Initialize correlation integration
    let correlation = Arc::new(CorrelationIntegration::new(Arc::clone(&persistence)));
    info!("Correlation integration initialized");
    
    // Initialize playbook integration
    let playbook_integration = Arc::new(PlaybookIntegration::new(Arc::clone(&persistence))?);
    info!("Playbook integration initialized");
    
    // Initialize visibility
    let visibility = Arc::new(ScannerVisibility::new(Arc::clone(&persistence)));
    info!("Scanner visibility interface initialized");
    
    // Initialize active scanner if enabled
    let active_scanner = if active_mode {
        match ActiveScanner::new() {
            Ok(scanner) => {
                info!("Active scanner initialized");
                Some(Arc::new(scanner))
            }
            Err(e) => {
                error!("Failed to initialize active scanner: {}", e);
                None
            }
        }
    } else {
        info!("Active scanner disabled");
        None
    };
    
    // Initialize passive scanner if enabled
    let passive_scanner = if passive_mode {
        match PassiveScanner::new() {
            Ok(scanner) => {
                info!("Passive scanner initialized");
                Some(Arc::new(scanner))
            }
            Err(e) => {
                error!("Failed to initialize passive scanner: {}", e);
                None
            }
        }
    } else {
        info!("Passive scanner disabled");
        None
    };
    
    info!("Phase 9 network scanner ready");
    
    // Service would run here (periodic scans, flow metadata processing, etc.)
    // For now, just keep running
    tokio::signal::ctrl_c().await?;
    
    info!("Shutting down Phase 9 network scanner");
    Ok(())
}

