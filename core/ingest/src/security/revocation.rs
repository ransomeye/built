// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/security/revocation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Revocation checking - checks if producer identities are revoked using CRL or signed revocation list

/*
 * Revocation Checker
 * 
 * Checks if producer identities are revoked.
 * Supports CRL (Certificate Revocation List) and signed revocation lists.
 * Revoked identity → TERMINATE CONNECTION
 */

use std::sync::Arc;
use std::path::Path;
use std::fs;
use std::collections::HashSet;
use dashmap::DashMap;
use x509_parser::prelude::*;
use x509_parser::revocation_list::*;
use chrono::{DateTime, Utc};
use tracing::{warn, debug, error, info};
use parking_lot::RwLock;
use sha2::{Sha256, Digest};
use ring::signature::{UnparsedPublicKey, RSA_PSS_2048_8192_SHA256};

use crate::security::errors::IdentityError;
use crate::security::trust_store::TrustStore;

#[derive(Clone)]
struct RevocationEntry {
    producer_id: String,
    certificate_serial: Vec<u8>,
    revoked_at: DateTime<Utc>,
    reason: String,
}

pub struct RevocationChecker {
    trust_store: Arc<TrustStore>,
    revocation_list: Arc<DashMap<String, RevocationEntry>>,
    crl_path: Option<String>,
    last_crl_update: Arc<RwLock<Option<DateTime<Utc>>>>,
    crl_update_interval: chrono::Duration,
}

impl RevocationChecker {
    pub fn new(
        trust_store: Arc<TrustStore>,
        crl_path: Option<String>,
    ) -> Result<Self, IdentityError> {
        let checker = Self {
            trust_store,
            revocation_list: Arc::new(DashMap::new()),
            crl_path,
            last_crl_update: Arc::new(RwLock::new(None)),
            crl_update_interval: chrono::Duration::hours(1),
        };
        
        // Load initial revocation list
        checker.load_revocation_list()?;
        
        Ok(checker)
    }
    
    /// Check if producer identity is revoked
    /// Returns true if revoked, false otherwise
    pub async fn is_revoked(&self, producer_id: &str) -> Result<bool, IdentityError> {
        // Check if CRL needs to be reloaded
        self.check_and_reload_crl().await?;
        
        // Check in-memory revocation list
        if self.revocation_list.contains_key(producer_id) {
            let entry = self.revocation_list.get(producer_id).unwrap();
            error!("Producer identity revoked: {} (revoked at: {}, reason: {})", 
                producer_id, entry.revoked_at, entry.reason);
            return Ok(true);
        }
        
        // Also check by certificate serial number if we have the certificate
        if let Ok(cert_data) = self.trust_store.get_producer_certificate_data(producer_id) {
            if let Ok(cert) = cert_data.parse_certificate() {
                let serial = cert.tbs_certificate.serial.to_bytes_be();
            
            // Check if any revocation entry matches this serial
            for entry in self.revocation_list.iter() {
                if entry.value().certificate_serial == serial {
                    error!("Producer certificate revoked by serial: {} (serial: {:?})", 
                        producer_id, serial);
                    return Ok(true);
                }
                }
            }
        }
        
        Ok(false)
    }
    
    fn load_revocation_list(&self) -> Result<(), IdentityError> {
        // Load from CRL file if available
        if let Some(ref crl_path) = self.crl_path {
            if Path::new(crl_path).exists() {
                info!("Loading CRL from: {}", crl_path);
                self.load_crl(crl_path)?;
            } else {
                warn!("CRL file not found: {}", crl_path);
            }
        }
        
        // Load from signed revocation list if available
        // This would be a JSON file signed by the root CA
        // For now, we'll implement basic file-based revocation list
        
        Ok(())
    }
    
    fn load_crl(&self, crl_path: &str) -> Result<(), IdentityError> {
        let crl_bytes = fs::read(crl_path)
            .map_err(|e| IdentityError::InternalError(
                format!("Failed to read CRL: {}", e)
            ))?;
        
        // Parse CRL (Certificate Revocation List)
        // x509-parser supports CRL parsing
        match CertificateRevocationList::from_der(&crl_bytes) {
            Ok((_, crl)) => {
                let revoked_certs = crl.iter_revoked_certificates().collect::<Vec<_>>();
                info!("Loaded CRL with {} entries", revoked_certs.len());
                
                // Verify CRL signature against root CA
                self.verify_crl_signature(&crl)?;
                
                // Add revoked certificates to revocation list
                for revoked_cert in revoked_certs.iter() {
                    let serial = revoked_cert.user_certificate.to_bytes_be();
                    
                    // Find producer_id by serial number
                    // This requires iterating through all producers
                    for producer_id in self.trust_store.list_producers() {
                        if let Ok(cert_data) = self.trust_store.get_producer_certificate_data(&producer_id) {
                            if let Ok(cert) = cert_data.parse_certificate() {
                                if cert.tbs_certificate.serial.to_bytes_be() == serial {
                                let entry = RevocationEntry {
                                    producer_id: producer_id.clone(),
                                    certificate_serial: serial.clone(),
                                    revoked_at: DateTime::from_timestamp(revoked_cert.revocation_date.timestamp(), 0)
                                        .unwrap_or_else(|| Utc::now()),
                                    reason: format!("CRL revocation"),
                                };
                                    info!("Revoked producer: {} (serial: {:?})", producer_id, serial);
                                    self.revocation_list.insert(producer_id, entry);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // Try loading as simple revocation list (JSON format)
                warn!("Failed to parse CRL as X.509 CRL: {}, trying JSON format", e);
                self.load_json_revocation_list(&crl_bytes)?;
            }
        }
        
        *self.last_crl_update.write() = Some(Utc::now());
        Ok(())
    }
    
    fn load_json_revocation_list(&self, bytes: &[u8]) -> Result<(), IdentityError> {
        // Parse JSON revocation list
        // Format: { "revoked": [ { "producer_id": "...", "serial": "...", "revoked_at": "...", "reason": "..." } ] }
        let json: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|e| IdentityError::InternalError(
                format!("Failed to parse JSON revocation list: {}", e)
            ))?;
        
        if let Some(revoked) = json.get("revoked").and_then(|v| v.as_array()) {
            for entry in revoked {
                if let (Some(producer_id), Some(serial_str)) = (
                    entry.get("producer_id").and_then(|v| v.as_str()),
                    entry.get("serial").and_then(|v| v.as_str()),
                ) {
                    let serial = hex::decode(serial_str)
                        .map_err(|e| IdentityError::InternalError(
                            format!("Invalid serial number: {}", e)
                        ))?;
                    
                    let revoked_at = entry.get("revoked_at")
                        .and_then(|v| v.as_str())
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now);
                    
                    let reason = entry.get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Manual revocation")
                        .to_string();
                    
                    let rev_entry = RevocationEntry {
                        producer_id: producer_id.to_string(),
                        certificate_serial: serial,
                        revoked_at,
                        reason,
                    };
                    
                    self.revocation_list.insert(producer_id.to_string(), rev_entry);
                    info!("Revoked producer from JSON list: {}", producer_id);
                }
            }
        }
        
        Ok(())
    }
    
    fn verify_crl_signature(&self, crl: &CertificateRevocationList) -> Result<(), IdentityError> {
        // Verify CRL signature against root CA
        // This ensures the CRL is authentic
        let root_ca_data = self.trust_store.get_root_ca_data()?;
        let root_ca = root_ca_data.parse_certificate()?;
        let root_public_key = root_ca.public_key();
        let root_public_key_bytes = root_public_key.raw;
        
        // Get CRL signature
        let signature = &crl.signature_value;
        
        // Get CRL TBS (To Be Signed) bytes
        let tbs_crl = crl.tbs_cert_list.as_ref();
        
        // Compute hash
        let mut hasher = Sha256::new();
        hasher.update(tbs_crl);
        let hash = hasher.finalize();
        
        // Verify signature
        let unparsed_key = UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA256, root_public_key_bytes);
        
        unparsed_key.verify(&hash, signature.as_ref())
            .map_err(|e| IdentityError::InternalError(
                format!("CRL signature verification failed: {}", e)
            ))?;
        
        debug!("CRL signature verified successfully");
        Ok(())
    }
    
    async fn check_and_reload_crl(&self) -> Result<(), IdentityError> {
        let last_update = *self.last_crl_update.read();
        let now = Utc::now();
        
        if let Some(last) = last_update {
            if now - last < self.crl_update_interval {
                return Ok(());
            }
        }
        
        // Reload CRL
        if let Some(ref crl_path) = self.crl_path {
            if Path::new(crl_path).exists() {
                debug!("Reloading CRL from: {}", crl_path);
                self.load_crl(crl_path)?;
            }
        }
        
        Ok(())
    }
}
