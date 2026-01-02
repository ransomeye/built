// Path and File Name : /home/ransomeye/rebuild/core/response_playbooks/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main entry point for Phase 6 playbook engine service

use std::sync::Arc;
use tracing::{info, error};
use ransomeye_response_playbooks::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting RansomEye Phase 6: Incident Response Playbooks Engine");
    
    // Get configuration from environment
    let playbook_dir = std::env::var("RANSOMEYE_PLAYBOOK_DIR")
        .unwrap_or_else(|_| "/etc/ransomeye/playbooks".to_string());
    
    let binding_file = std::env::var("RANSOMEYE_POLICY_BINDING_FILE")
        .unwrap_or_else(|_| "/etc/ransomeye/config/policy_bindings.yaml".to_string());
    
    // Initialize registry
    let registry = Arc::new(PlaybookRegistry::new(&playbook_dir)?);
    info!("Playbook registry initialized with {} playbooks", registry.count());
    
    // Initialize persistence
    let persistence = Arc::new(PlaybookPersistence::new().await?);
    info!("Playbook persistence initialized");
    
    // Initialize executor
    let executor = Arc::new(PlaybookExecutor::new(
        Arc::clone(&registry),
        Arc::clone(&persistence),
    ));
    info!("Playbook executor initialized");
    
    // Initialize rollback engine
    let rollback_engine = Arc::new(RollbackEngine::new(Arc::clone(&persistence)));
    info!("Rollback engine initialized");
    
    // Initialize policy binding manager
    let binding_manager = PolicyPlaybookBindingManager::new(
        Arc::clone(&registry),
        &binding_file,
    )?;
    info!("Policy-playbook binding manager initialized");
    
    // Initialize visibility interface
    let visibility = PlaybookVisibility::new(
        Arc::clone(&registry),
        Arc::clone(&executor),
        Arc::clone(&rollback_engine),
    );
    info!("Playbook visibility interface initialized");
    
    info!("Phase 6 playbook engine ready");
    
    // Service would run here (API server, message bus listener, etc.)
    // For now, just keep running
    tokio::signal::ctrl_c().await?;
    
    info!("Shutting down Phase 6 playbook engine");
    Ok(())
}

