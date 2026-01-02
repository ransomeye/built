// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Dispatcher main entry point

use tracing::{info, error};
use ransomeye_dispatcher::EnforcementDispatcher;
use ransomeye_dispatcher::DispatcherError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("RansomEye Phase 7 Dispatcher starting");
    
    // Initialize dispatcher
    let dispatcher = match EnforcementDispatcher::new() {
        Ok(d) => {
            info!("Dispatcher initialized successfully");
            d
        }
        Err(e) => {
            error!("Failed to initialize dispatcher: {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };
    
    info!("Dispatcher ready - waiting for directives");
    
    // In production, would connect to directive queue and process directives
    // For now, keep running
    tokio::signal::ctrl_c().await?;
    
    info!("Dispatcher shutting down");
    Ok(())
}

