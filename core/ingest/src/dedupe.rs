// Path and File Name : /home/ransomeye/rebuild/core/ingest/src/dedupe.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Content hash deduplication - prevents duplicate payload processing using cryptographic hashing

/*
 * Content Hash Deduplication
 * 
 * Implements content-based deduplication using:
 * - Canonicalized payload
 * - Cryptographic hash (SHA-256)
 * - Time-bounded dedup window
 * - Deterministic duplicate detection
 * 
 * FAIL-CLOSED: Duplicate payloads are dropped deterministically
 */

use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use parking_lot::RwLock;
use sha2::{Sha256, Digest};
use tracing::{warn, debug};
use chrono::Utc;

use crate::protocol::event_envelope::EventEnvelope;
use crate::config::Config;

#[derive(Debug, Clone)]
struct ContentHashRecord {
    first_seen: Instant,
    count: u64,
}

pub struct ContentDeduplicator {
    config: Config,
    content_hashes: Arc<DashMap<String, ContentHashRecord>>,
    dedup_window: Duration,
    last_cleanup: Arc<RwLock<Instant>>,
}

impl ContentDeduplicator {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let dedup_window_seconds = std::env::var("RANSOMEYE_DEDUP_WINDOW_SECONDS")
            .unwrap_or_else(|_| "300".to_string()) // 5 minutes default
            .parse::<u64>()
            .map_err(|e| format!("Invalid dedup window: {}", e))?;
        
        let dedup_window = Duration::from_secs(dedup_window_seconds);
        
        Ok(Self {
            config: config.clone(),
            content_hashes: Arc::new(DashMap::new()),
            dedup_window,
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        })
    }
    
    /// Compute canonical content hash from event payload
    /// 
    /// Canonicalizes by:
    /// 1. Normalizing event_data (trim, sort JSON keys if applicable)
    /// 2. Combining producer_id, component_type, event_data
    /// 3. Computing SHA-256 hash
    fn compute_content_hash(&self, envelope: &EventEnvelope) -> String {
        // Canonicalize payload: normalize event_data
        let mut canonical_data = envelope.event_data.clone();
        // Trim whitespace
        canonical_data = canonical_data.trim().to_string();
        
        // Build canonical string: producer_id|component_type|event_data
        let canonical = format!("{}|{}|{}", 
            envelope.producer_id.trim(),
            envelope.component_type.trim().to_lowercase(),
            canonical_data
        );
        
        // Compute SHA-256 hash
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let hash = hasher.finalize();
        
        // Return hex-encoded hash
        hex::encode(hash)
    }
    
    /// Check if content should be processed (not a duplicate)
    /// 
    /// Returns:
    /// - true: Process (new or expired duplicate)
    /// - false: Drop (duplicate within window)
    pub async fn should_process(&self, envelope: &EventEnvelope) -> Result<bool, Box<dyn std::error::Error>> {
        // Cleanup old entries periodically
        self.cleanup_expired().await;
        
        // Compute content hash
        let content_hash = self.compute_content_hash(envelope);
        
        let now = Instant::now();
        
        // Check if hash exists
        if let Some(mut record) = self.content_hashes.get_mut(&content_hash) {
            // Hash exists - check if within window
            let age = now.duration_since(record.first_seen);
            
            if age < self.dedup_window {
                // Duplicate within window - drop
                record.count += 1;
                warn!("Content duplicate detected (hash: {}, age: {}s, count: {})", 
                      &content_hash[..16], age.as_secs(), record.count);
                return Ok(false);
            } else {
                // Expired - update record
                record.first_seen = now;
                record.count = 1;
                debug!("Content hash expired, resetting (hash: {})", &content_hash[..16]);
                return Ok(true);
            }
        } else {
            // New hash - add record
            self.content_hashes.insert(content_hash.clone(), ContentHashRecord {
                first_seen: now,
                count: 1,
            });
            debug!("New content hash registered (hash: {})", &content_hash[..16]);
            return Ok(true);
        }
    }
    
    /// Cleanup expired hash records
    async fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut last_cleanup = self.last_cleanup.write();
        
        // Cleanup every 60 seconds
        if now.duration_since(*last_cleanup) < Duration::from_secs(60) {
            return;
        }
        
        *last_cleanup = now;
        
        let mut expired_hashes = Vec::new();
        
        for entry in self.content_hashes.iter() {
            let age = now.duration_since(entry.value().first_seen);
            if age >= self.dedup_window {
                expired_hashes.push(entry.key().clone());
            }
        }
        
        let count = expired_hashes.len();
        for hash in expired_hashes {
            self.content_hashes.remove(&hash);
        }
        
        if count > 0 {
            debug!("Cleaned up {} expired content hashes", count);
        }
    }
}

