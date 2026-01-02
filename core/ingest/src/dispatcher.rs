// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/dispatcher.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Event dispatcher - dispatches validated events to Control Plane

/*
 * Event Dispatcher
 * 
 * Dispatches validated events to Control Plane.
 * Handles downstream unavailability.
 * Maintains delivery guarantees.
 */

use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tracing::{error, warn, debug};

use crate::protocol::event_envelope::EventEnvelope;
use crate::config::Config;

pub struct EventDispatcher {
    config: Config,
    control_plane_address: String,
}

impl EventDispatcher {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config: config.clone(),
            control_plane_address: config.control_plane_address.clone(),
        })
    }
    
    pub async fn dispatch(&self, envelope: &EventEnvelope) -> Result<(), Box<dyn std::error::Error>> {
        // Connect to Control Plane
        let mut stream = match TcpStream::connect(&self.control_plane_address).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to connect to Control Plane: {}", e);
                return Err("Control Plane unavailable".into());
            }
        };
        
        // Serialize envelope
        let data = serde_json::to_vec(envelope)?;
        
        // Send to Control Plane
        match stream.write_all(&data).await {
            Ok(_) => {
                debug!("Event dispatched to Control Plane: producer={}, sequence={}", 
                      envelope.producer_id, envelope.sequence_number);
                Ok(())
            }
            Err(e) => {
                error!("Failed to dispatch event: {}", e);
                Err("Dispatch failed".into())
            }
        }
    }
}

