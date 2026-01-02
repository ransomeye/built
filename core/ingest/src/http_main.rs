// Path and File Name : /home/ransomeye/rebuild/core/ingest/src/http_main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: HTTP ingestion server main entry point - listens on :8080 and accepts Linux Agent + DPI Probe telemetry

use std::env;
use tokio::signal;
use tracing::{info, error};

mod http_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting RansomEye HTTP Ingestion Server");

    // Get listen address from environment (default: 127.0.0.1:8080)
    let listen_addr = env::var("RANSOMEYE_INGESTION_LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    // Create and start server
    let server = http_server::HttpIngestionServer::new(listen_addr.clone()).await?;
    
    info!("HTTP Ingestion Server initialized, starting on {}", listen_addr);

    // Start server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            error!("Server error: {}", e);
            std::process::exit(1);
        }
    });

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received");

    // Cancel server task
    server_handle.abort();
    
    info!("HTTP Ingestion Server stopped");
    Ok(())
}

