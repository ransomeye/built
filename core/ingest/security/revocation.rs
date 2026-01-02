// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/security/revocation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Revocation checking - checks if producer identities are revoked

/*
 * Revocation Checker
 * 
 * Checks if producer identities are revoked.
 * Maintains revocation list.
 */

use std::sync::Arc;
use dashmap::DashMap;
use tracing::{warn, debug};

pub struct RevocationChecker {
    revocation_list: Arc<DashMap<String, bool>>,
}

impl RevocationChecker {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            revocation_list: Arc::new(DashMap::new()),
        })
    }
    
    pub async fn is_revoked(&self, producer_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let revoked = self.revocation_list.contains_key(producer_id);
        if revoked {
            warn!("Producer identity revoked: {}", producer_id);
        }
        Ok(revoked)
    }
}

