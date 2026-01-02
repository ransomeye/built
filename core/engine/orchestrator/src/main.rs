// Path and File Name : /home/ransomeye/rebuild/core/engine/orchestrator/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main entrypoint for RansomEye Core Orchestrator - fail-closed lifecycle management

use std::process;
use tracing::{info, error};

// Import orchestrator library
// Since this is a binary in the engine crate, we need to reference the module
// We'll include the orchestrator module directly
#[path = "lib.rs"]
mod orchestrator;

use orchestrator::Orchestrator;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("RansomEye Core Orchestrator starting...");

    // Create orchestrator
    let mut orchestrator = match Orchestrator::new() {
        Ok(orch) => orch,
        Err(e) => {
            error!("Failed to create orchestrator: {}", e);
            process::exit(1);
        }
    };

    // Run orchestrator (startup -> wait -> shutdown)
    match orchestrator.run().await {
        Ok(_) => {
            info!("Orchestrator exited successfully");
            process::exit(0);
        }
        Err(e) => {
            error!("Orchestrator error: {}", e);
            error!("FAIL-CLOSED: System will not start with errors");
            // Best-effort DB error recording (never masks the original failure).
            orchestrator.record_fatal_error(&format!("{e}")).await;
            process::exit(1);
        }
    }
}

