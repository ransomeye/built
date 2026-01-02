// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main entry point for RansomEye ingestion server - initializes and runs ingestion service

/*
 * RansomEye Event Ingestion Server
 * 
 * The ONLY ingress point into the RansomEye Control Plane.
 * Handles untrusted, potentially malicious data with strict validation.
 */

use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

mod server;
mod listener;
mod auth;
mod signature;
mod schema;
mod versioning;
mod normalization;
mod rate_limit;
mod backpressure;
mod buffer;
mod ordering;
mod dispatcher;
mod config;
mod security;
mod protocol;
mod dedupe;

use server::IngestionServer;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting RansomEye Event Ingestion Server");
    
    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded");
    
    // Create ingestion server
    let server = Arc::new(IngestionServer::new(config).await?);
    
    // Start server
    let server_handle = {
        let server = server.clone();
        tokio::spawn(async move {
            if let Err(e) = server.start().await {
                error!("Server error: {}", e);
                std::process::exit(1);
            }
        })
    };
    
    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received");
    
    // Graceful shutdown
    server.shutdown().await;
    server_handle.await?;
    
    info!("RansomEye Event Ingestion Server stopped");
    Ok(())
}

