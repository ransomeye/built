// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/security/replay_protection.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Replay protection - detects and prevents replay attacks using nonces and timestamps

/*
 * Replay Protection
 * 
 * Detects and prevents replay attacks using nonces and timestamps.
 * Maintains nonce cache with expiration.
 */

use std::sync::Arc;
use dashmap::DashMap;
use chrono::{DateTime, Utc, Duration};
use tracing::{warn, debug};

pub struct ReplayProtector {
    nonce_cache: Arc<DashMap<String, DashMap<String, DateTime<Utc>>>>,
    nonce_ttl: Duration,
    timestamp_tolerance: Duration,
}

impl ReplayProtector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            nonce_cache: Arc::new(DashMap::new()),
            nonce_ttl: Duration::hours(24),
            timestamp_tolerance: Duration::minutes(5),
        })
    }
    
    pub async fn check_replay(
        &self,
        producer_id: &str,
        nonce: &str,
        timestamp: &DateTime<Utc>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let now = Utc::now();
        
        // Check timestamp tolerance
        let time_diff = (*timestamp - now).num_seconds().abs();
        if time_diff > self.timestamp_tolerance.num_seconds() {
            warn!("Timestamp out of tolerance for producer: {}", producer_id);
            return Ok(false);
        }
        
        // Get or create producer nonce cache
        let producer_nonces = self.nonce_cache
            .entry(producer_id.to_string())
            .or_insert_with(DashMap::new);
        
        // Check if nonce already seen
        if producer_nonces.contains_key(nonce) {
            warn!("Replay detected for producer: {}, nonce: {}", producer_id, nonce);
            return Ok(false);
        }
        
        // Add nonce to cache
        producer_nonces.insert(nonce.to_string(), *timestamp);
        
        // Cleanup expired nonces
        self.cleanup_expired_nonces(producer_id, now).await;
        
        debug!("Replay check passed for producer: {}", producer_id);
        Ok(true)
    }
    
    async fn cleanup_expired_nonces(&self, producer_id: &str, now: DateTime<Utc>) {
        if let Some(producer_nonces) = self.nonce_cache.get(producer_id) {
            let expired: Vec<String> = producer_nonces
                .iter()
                .filter(|entry| {
                    let age = now - *entry.value();
                    age > self.nonce_ttl
                })
                .map(|entry| entry.key().clone())
                .collect();
            
            for nonce in expired {
                producer_nonces.remove(&nonce);
            }
        }
    }
}

