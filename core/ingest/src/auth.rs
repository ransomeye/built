// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/auth.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Producer authentication - verifies producer identity, expiration, revocation, and replay protection

/*
 * Producer Authentication
 * 
 * Enforces mutual authentication, verifies producer identity,
 * checks identity expiration, revocation status, and replay protection.
 * Fails-closed on any authentication failure.
 */

use std::sync::Arc;
use std::net::SocketAddr;
use dashmap::DashMap;
use chrono::{DateTime, Utc};
use tracing::{error, debug};

use crate::protocol::event_envelope::EventEnvelope;
use crate::config::Config;
use crate::security::identity::IdentityVerifier;
use crate::security::replay_protection::ReplayProtector;

pub struct Authenticator {
    _config: Config,
    identity_verifier: Arc<IdentityVerifier>,
    replay_protector: Arc<ReplayProtector>,
    producer_cache: Arc<DashMap<String, ProducerInfo>>,
}

struct ProducerInfo {
    _producer_id: String,
    _component_type: String,
    _authenticated_at: DateTime<Utc>,
    _expires_at: DateTime<Utc>,
}

impl Authenticator {
    pub fn new(
        config: &Config,
        identity_verifier: Arc<IdentityVerifier>,
        replay_protector: Arc<ReplayProtector>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            _config: config.clone(),
            identity_verifier,
            replay_protector,
            producer_cache: Arc::new(DashMap::new()),
        })
    }
    
    pub async fn authenticate(
        &self,
        envelope: &EventEnvelope,
        addr: &SocketAddr,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let producer_id = &envelope.producer_id;
        
        debug!("Authenticating producer: {} from {}", producer_id, addr);
        
        // Step 1: Verify identity (includes certificate chain, expiration, signature)
        let verified_identity = self.identity_verifier.verify(envelope).await
            .map_err(|e| {
                error!("Identity verification failed for producer: {}: {}", producer_id, e);
                format!("Identity verification failed: {}", e)
            })?;
        
        // Step 2: Check replay protection (nonce, timestamp, sequence number)
        self.replay_protector.check_replay(
            producer_id,
            &envelope.nonce,
            &envelope.timestamp,
            envelope.sequence_number,
        ).await
            .map_err(|e| {
                error!("Replay protection failed for producer: {}: {}", producer_id, e);
                format!("Replay protection failed: {}", e)
            })?;
        
        // Step 3: Validate component type
        let component_type = envelope.component_type.clone();
        if !self.is_valid_component_type(&component_type) {
            error!("Invalid component type: {} for producer: {}", component_type, producer_id);
            return Err("Invalid component type".into());
        }
        
        // Step 4: Cache producer info
        let producer_info = ProducerInfo {
            _producer_id: producer_id.clone(),
            _component_type: component_type,
            _authenticated_at: Utc::now(),
            _expires_at: verified_identity.valid_until,
        };
        self.producer_cache.insert(producer_id.clone(), producer_info);
        
        debug!("Producer authenticated successfully: {}", producer_id);
        Ok(producer_id.clone())
    }
    
    fn is_valid_component_type(&self, component_type: &str) -> bool {
        matches!(component_type, "dpi_probe" | "linux_agent" | "windows_agent")
    }
}
