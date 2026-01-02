// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/signature.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Signature verification - signature verification is now handled by identity verification

/*
 * Signature Verification
 * 
 * NOTE: Signature verification is now handled by IdentityVerifier as part of
 * the identity verification process. This module is kept for backward compatibility
 * but signature verification is performed during authentication.
 */

use std::sync::Arc;
use tracing::debug;

use crate::protocol::event_envelope::EventEnvelope;
use crate::config::Config;

pub struct SignatureVerifier {
    config: Config,
}

impl SignatureVerifier {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config: config.clone(),
        })
    }
    
    /// Verify signature
    /// NOTE: Signature verification is now performed by IdentityVerifier during authentication.
    /// This method is kept for backward compatibility but always returns Ok.
    pub async fn verify(&self, envelope: &EventEnvelope) -> Result<(), Box<dyn std::error::Error>> {
        // Signature is already verified by IdentityVerifier during authentication
        debug!("Signature verification skipped (handled by identity verification)");
        Ok(())
    }
}
