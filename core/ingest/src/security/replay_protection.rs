// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/security/replay_protection.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Replay protection - detects and prevents replay attacks using nonces, timestamps, and sequence number monotonicity

/*
 * Replay Protection
 * 
 * Detects and prevents replay attacks using:
 * - Nonce cache with TTL
 * - Timestamp skew limits
 * - Sequence number monotonicity per producer
 * 
 * Replay → HARD REJECT + AUDIT LOG
 */

use std::sync::Arc;
use dashmap::DashMap;
use chrono::{DateTime, Utc, Duration};
use tracing::{warn, debug, error};
use parking_lot::RwLock;

use crate::security::errors::IdentityError;

#[derive(Clone)]
struct NonceEntry {
    timestamp: DateTime<Utc>,
    sequence_number: u64,
}

#[derive(Clone)]
struct ProducerState {
    last_sequence_number: u64,
    last_timestamp: DateTime<Utc>,
}

pub struct ReplayProtector {
    nonce_cache: Arc<DashMap<String, DashMap<String, NonceEntry>>>,
    producer_states: Arc<DashMap<String, RwLock<ProducerState>>>,
    nonce_ttl: Duration,
    timestamp_tolerance: Duration,
    max_sequence_gap: u64,
}

impl ReplayProtector {
    pub fn new() -> Result<Self, IdentityError> {
        Ok(Self {
            nonce_cache: Arc::new(DashMap::new()),
            producer_states: Arc::new(DashMap::new()),
            nonce_ttl: Duration::hours(24),
            timestamp_tolerance: Duration::minutes(5),
            max_sequence_gap: 1000, // Allow up to 1000 sequence number gap
        })
    }
    
    /// Check for replay attacks
    /// Returns Ok(()) on success, IdentityError on replay detection
    pub async fn check_replay(
        &self,
        producer_id: &str,
        nonce: &str,
        timestamp: &DateTime<Utc>,
        sequence_number: u64,
    ) -> Result<(), IdentityError> {
        let now = Utc::now();
        
        // Step 1: Check timestamp tolerance
        let time_diff = (*timestamp - now).num_seconds().abs();
        if time_diff > self.timestamp_tolerance.num_seconds() {
            error!("REPLAY ATTACK: Timestamp out of tolerance for producer: {} (diff: {}s)", 
                producer_id, time_diff);
            return Err(IdentityError::TimestampOutOfTolerance(
                format!("Timestamp difference: {} seconds", time_diff)
            ));
        }
        
        // Step 2: Check nonce uniqueness
        let producer_nonces = self.nonce_cache
            .entry(producer_id.to_string())
            .or_insert_with(DashMap::new);
        
        if producer_nonces.contains_key(nonce) {
            error!("REPLAY ATTACK: Duplicate nonce detected for producer: {}, nonce: {}", 
                producer_id, nonce);
            return Err(IdentityError::ReplayAttack(
                format!("Duplicate nonce: {}", nonce)
            ));
        }
        
        // Step 3: Check sequence number monotonicity
        let producer_state = self.producer_states
            .entry(producer_id.to_string())
            .or_insert_with(|| {
                RwLock::new(ProducerState {
                    last_sequence_number: 0,
                    last_timestamp: *timestamp,
                })
            });
        
        {
            let mut state = producer_state.write();
            
            // Check if sequence number is less than last seen (replay)
            if sequence_number < state.last_sequence_number {
                error!("REPLAY ATTACK: Sequence number regression for producer: {} (last: {}, current: {})", 
                    producer_id, state.last_sequence_number, sequence_number);
                return Err(IdentityError::SequenceNumberViolation(
                    format!("Sequence number regression: {} < {}", sequence_number, state.last_sequence_number)
                ));
            }
            
            // Check if sequence number gap is too large (potential attack or data loss)
            let sequence_gap = sequence_number - state.last_sequence_number;
            if sequence_gap > self.max_sequence_gap && state.last_sequence_number > 0 {
                warn!("Large sequence number gap for producer: {} (gap: {})", 
                    producer_id, sequence_gap);
                // Allow but log warning - this could indicate legitimate data loss
            }
            
            // Check if timestamp is before last seen (replay)
            if *timestamp < state.last_timestamp {
                error!("REPLAY ATTACK: Timestamp regression for producer: {} (last: {}, current: {})", 
                    producer_id, state.last_timestamp, timestamp);
                return Err(IdentityError::TimestampOutOfTolerance(
                    format!("Timestamp regression: {} < {}", timestamp, state.last_timestamp)
                ));
            }
            
            // Step 4: Update state
            state.last_sequence_number = sequence_number;
            state.last_timestamp = *timestamp;
        } // Lock explicitly dropped here
        
        // Step 5: Add nonce to cache
        producer_nonces.insert(nonce.to_string(), NonceEntry {
            timestamp: *timestamp,
            sequence_number,
        });
        
        // Step 6: Cleanup expired nonces (async, after releasing all locks)
        self.cleanup_expired_nonces(producer_id, now).await;
        
        debug!("Replay check passed for producer: {} (nonce: {}, seq: {})", 
            producer_id, nonce, sequence_number);
        Ok(())
    }
    
    async fn cleanup_expired_nonces(&self, producer_id: &str, now: DateTime<Utc>) {
        if let Some(producer_nonces) = self.nonce_cache.get(producer_id) {
            let expired: Vec<String> = producer_nonces
                .iter()
                .filter(|entry| {
                    let age = now - entry.value().timestamp;
                    age > self.nonce_ttl
                })
                .map(|entry| entry.key().clone())
                .collect();
            
            for nonce in expired {
                producer_nonces.remove(&nonce);
            }
        }
    }
    
    /// Get last sequence number for a producer (for debugging)
    pub fn get_last_sequence_number(&self, producer_id: &str) -> Option<u64> {
        self.producer_states
            .get(producer_id)
            .map(|state| state.read().last_sequence_number)
    }
}
