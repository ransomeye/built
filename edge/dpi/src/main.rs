// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main entry point for DPI Probe - high-throughput packet capture and telemetry generation

/*
 * RansomEye DPI Probe
 * 
 * Stand-alone sensor for high-throughput network packet inspection.
 * UNTRUSTED component - never enforces policy, never runs AI, never stores long-term state.
 * All telemetry is signed and sent to Control Plane via mTLS.
 */

use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

mod capture;
mod flow;
mod feature;
mod signing;
mod transport;
mod backpressure;
mod health;
mod config;
mod identity;

use capture::CaptureEngine;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting RansomEye DPI Probe");
    
    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded");
    
    // Initialize identity
    let identity = Arc::new(identity::Identity::load_or_create(&config)?);
    info!("Identity initialized: {}", identity.producer_id());
    
    // Create capture engine
    let capture_engine = Arc::new(CaptureEngine::new(config.clone(), identity.clone()).await?);
    
    // Start capture engine
    let capture_handle = {
        let engine = capture_engine.clone();
        tokio::spawn(async move {
            if let Err(e) = engine.start().await {
                error!("Capture engine error: {}", e);
                std::process::exit(1);
            }
        })
    };
    
    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received");
    
    // Graceful shutdown
    capture_engine.shutdown().await;
    capture_handle.await?;
    
    info!("RansomEye DPI Probe stopped");
    Ok(())
}

