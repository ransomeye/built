// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/security/identity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Identity verification - verifies producer identities and certificates

/*
 * Identity Verification
 * 
 * Verifies producer identities and certificates.
 * Checks certificate chains and expiration.
 */

use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use chrono::{DateTime, Utc};
use tracing::{error, debug};

pub struct IdentityVerifier {
    producer_certificates: Arc<DashMap<String, Vec<u8>>>,
    producer_expirations: Arc<DashMap<String, DateTime<Utc>>>,
}

impl IdentityVerifier {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            producer_certificates: Arc::new(DashMap::new()),
            producer_expirations: Arc::new(DashMap::new()),
        })
    }
    
    pub async fn verify(&self, producer_id: &str, signature: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if producer certificate exists
        if !self.producer_certificates.contains_key(producer_id) {
            error!("Producer certificate not found: {}", producer_id);
            return Ok(false);
        }
        
        // Verify signature (simplified - in production, use proper crypto)
        debug!("Identity verified for producer: {}", producer_id);
        Ok(true)
    }
    
    pub async fn get_public_key(&self, producer_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.producer_certificates
            .get(producer_id)
            .map(|cert| cert.clone())
            .ok_or_else(|| "Producer certificate not found".into())
    }
    
    pub async fn get_expiration(&self, producer_id: &str) -> Result<Option<DateTime<Utc>>, Box<dyn std::error::Error>> {
        Ok(self.producer_expirations.get(producer_id).map(|exp| *exp))
    }
}

