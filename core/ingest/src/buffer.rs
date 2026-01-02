// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/buffer.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Bounded event buffer - manages event buffering with explicit capacity limits

/*
 * Event Buffer
 * 
 * Manages bounded event buffering.
 * Uses explicit capacity limits.
 * Never drops events silently.
 * Rejects events when buffer is full.
 */

use std::sync::Arc;
use tokio::sync::RwLock;
use crossbeam_channel::{bounded, Receiver, Sender};
use tracing::{warn, debug};

use crate::protocol::event_envelope::EventEnvelope;
use crate::config::Config;

pub struct EventBuffer {
    config: Config,
    capacity: usize,
    sender: Sender<EventEnvelope>,
    receiver: Receiver<EventEnvelope>,
    current_size: Arc<RwLock<usize>>,
}

impl EventBuffer {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let capacity = config.buffer_capacity;
        let (sender, receiver) = bounded(capacity);
        
        Ok(Self {
            config: config.clone(),
            capacity,
            sender,
            receiver,
            current_size: Arc::new(RwLock::new(0)),
        })
    }
    
    pub async fn has_capacity(&self) -> bool {
        let size = *self.current_size.read().await;
        size < self.capacity
    }
    
    pub async fn add(&self, envelope: &EventEnvelope) -> Result<(), Box<dyn std::error::Error>> {
        // Check capacity
        if !self.has_capacity().await {
            return Err("Buffer full".into());
        }
        
        // Try to send (non-blocking)
        match self.sender.try_send(envelope.clone()) {
            Ok(_) => {
                *self.current_size.write().await += 1;
                debug!("Event added to buffer, size: {}", *self.current_size.read().await);
                Ok(())
            }
            Err(_) => {
                warn!("Buffer full, event rejected");
                Err("Buffer full".into())
            }
        }
    }
    
    pub async fn flush(&self) {
        // Drain buffer
        while self.receiver.try_recv().is_ok() {
            *self.current_size.write().await = self.current_size.read().await.saturating_sub(1);
        }
        debug!("Buffer flushed");
    }
    
    pub fn get_receiver(&self) -> Receiver<EventEnvelope> {
        self.receiver.clone()
    }
}

