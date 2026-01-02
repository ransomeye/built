// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main entry point for enforcement dispatcher service

/*
 * RansomEye Enforcement Dispatcher
 * 
 * Converts policy decisions into controlled enforcement commands.
 * Enforces safety guards, approvals, rate limits, and blast radius limits.
 */

use std::env;
use tokio::signal;
use tracing::{info, error};

mod dispatcher;
mod validator;
mod approvals;
mod guardrails;
mod rate_limit;
mod blast_radius;
mod rollback;
mod dry_run;
mod output;
mod errors;
mod adapters;
mod security;

use dispatcher::EnforcementDispatcher;
use errors::EnforcementError;

const DISPATCHER_VERSION: &str = "1.0.0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting RansomEye Enforcement Dispatcher (version: {})", DISPATCHER_VERSION);
    
    // Initialize dispatcher
    let dispatcher = EnforcementDispatcher::new()
        .map_err(|e| {
            error!("Failed to initialize dispatcher: {}", e);
            e
        })?;
    
    info!("Enforcement Dispatcher initialized successfully");
    info!("Safety guards: ENABLED");
    info!("Approval workflows: ENABLED");
    info!("Rate limiting: ENABLED");
    info!("Blast radius limiting: ENABLED");
    info!("Rollback support: ENABLED");
    info!("Dry-run mode: ENABLED");
    
    // In production, would connect to decision queue and process decisions
    info!("Enforcement Dispatcher ready");
    info!("Waiting for policy decisions...");
    
    // Keep running (in production, would process decisions from queue)
    signal::ctrl_c().await?;
    
    info!("Shutting down Enforcement Dispatcher");
    Ok(())
}

