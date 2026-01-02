// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/ordering.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Event ordering - ensures per-producer ordering and detects replays

/*
 * Ordering Manager
 * 
 * Ensures per-producer event ordering.
 * Detects replay attacks.
 * Handles out-of-order events.
 * Explicit ordering rules.
 */

use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::RwLock;
use tracing::{warn, debug};

use crate::protocol::event_envelope::EventEnvelope;
use crate::config::Config;
use crate::security::replay_protection::ReplayProtector;

pub struct OrderingManager {
    config: Config,
    replay_protector: Arc<ReplayProtector>,
    producer_sequences: Arc<DashMap<String, u64>>,
    expected_sequences: Arc<DashMap<String, u64>>,
}

impl OrderingManager {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config: config.clone(),
            replay_protector: Arc::new(ReplayProtector::new()?),
            producer_sequences: Arc::new(DashMap::new()),
            expected_sequences: Arc::new(DashMap::new()),
        })
    }
    
    pub async fn check_ordering(
        &self,
        producer_id: &str,
        envelope: &EventEnvelope,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Check replay protection
        self.replay_protector.check_replay(producer_id, &envelope.nonce, &envelope.timestamp, envelope.sequence_number).await?;
        
        // Check sequence ordering
        let sequence = envelope.sequence_number;
        {
            let mut expected = self.expected_sequences
                .entry(producer_id.to_string())
                .or_insert(0);
            
            // Check if sequence is in order
            if sequence < *expected {
                // Out of order or replay
                let exp_copy = *expected;
                warn!("Out of order sequence for producer {}: expected {}, got {}", 
                      producer_id, exp_copy, sequence);
                return Ok(false);
            }
            
            // Update expected sequence
            *expected = sequence + 1;
        }
        
        // Record sequence
        self.producer_sequences.insert(producer_id.to_string(), sequence);
        
        debug!("Ordering check passed for producer: {}, sequence: {}", producer_id, sequence);
        Ok(true)
    }
}

