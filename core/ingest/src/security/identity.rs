// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/security/identity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Identity verification - verifies producer identities using X.509 certificates with real cryptography

/*
 * Identity Verification
 * 
 * Verifies producer identities using X.509 certificates.
 * Enforces certificate chain validation, expiration, revocation, and signature verification.
 * Fail-closed: rejects on any verification ambiguity.
 */

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use ring::signature::{UnparsedPublicKey, RSA_PSS_2048_8192_SHA256};
use sha2::{Sha256, Digest};
use x509_parser::prelude::*;
use x509_parser::public_key::PublicKey;
use chrono::{DateTime, Utc};
use tracing::{error, warn, debug};
use base64;

use crate::security::errors::{IdentityError, VerifiedIdentity};
use crate::security::trust_store::TrustStore;
use crate::security::trust_chain::TrustChainValidator;
use crate::security::revocation::RevocationChecker;
use crate::protocol::event_envelope::EventEnvelope;

pub struct IdentityVerifier {
    trust_store: Arc<TrustStore>,
    trust_chain_validator: Arc<TrustChainValidator>,
    revocation_checker: Arc<RevocationChecker>,
}

impl IdentityVerifier {
    pub fn new(
        trust_store: Arc<TrustStore>,
        trust_chain_validator: Arc<TrustChainValidator>,
        revocation_checker: Arc<RevocationChecker>,
    ) -> Result<Self, IdentityError> {
        Ok(Self {
            trust_store,
            trust_chain_validator,
            revocation_checker,
        })
    }
    
    /// Verify producer identity from event envelope
    /// Returns VerifiedIdentity on success, IdentityError on failure
    /// Fail-closed: rejects on any verification ambiguity
    pub async fn verify(
        &self,
        envelope: &EventEnvelope,
    ) -> Result<VerifiedIdentity, IdentityError> {
        let producer_id = &envelope.producer_id;
        
        debug!("Verifying identity for producer: {}", producer_id);
        
        // Step 1: Get producer certificate from trust store
        let cert_data = self.trust_store.get_producer_certificate_data(producer_id)?;
        let cert = cert_data.parse_certificate()?;
        
        // Step 2: Verify certificate chain against root CA
        self.trust_chain_validator.validate_chain(&cert_data.der).await?;
        
        // Step 3: Verify certificate validity period
        self.verify_certificate_validity(&cert)?;
        
        // Step 4: Verify certificate key usage
        self.verify_key_usage(&cert)?;
        
        // Step 5: Verify certificate subject matches producer_id
        self.verify_certificate_subject(&cert, producer_id)?;
        
        // Step 6: Check revocation status
        if self.revocation_checker.is_revoked(producer_id).await? {
            return Err(IdentityError::CertificateRevoked(producer_id.clone()));
        }
        
        // Step 7: Verify signature
        self.verify_signature(&cert, envelope)?;
        
        // Step 8: Extract certificate metadata
        let verified_identity = self.extract_identity(&cert, producer_id)?;
        
        debug!("Identity verified successfully for producer: {}", producer_id);
        Ok(verified_identity)
    }
    
    fn verify_certificate_validity(&self, cert: &X509Certificate) -> Result<(), IdentityError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| IdentityError::InternalError(
                format!("System time error: {}", e)
            ))?
            .as_secs();
        
        // Check not_before
        let validity = cert.validity();
        let not_before = validity.not_before.timestamp();
        if now < not_before as u64 {
            return Err(IdentityError::CertificateNotYetValid(
                format!("Certificate not valid until {}", validity.not_before)
            ));
        }
        
        // Check not_after
        let not_after = validity.not_after.timestamp();
        if now > not_after as u64 {
            return Err(IdentityError::CertificateExpired(
                format!("Certificate expired on {}", validity.not_after)
            ));
        }
        
        Ok(())
    }
    
    fn verify_key_usage(&self, cert: &X509Certificate) -> Result<(), IdentityError> {
        // Verify that certificate has digital signature key usage
        let extensions = cert.extensions();
        for ext in extensions {
            if let ParsedExtension::KeyUsage(key_usage) = ext.parsed_extension() {
                if !key_usage.digital_signature() {
                    return Err(IdentityError::InvalidKeyUsage(
                        "Certificate does not have digital signature key usage".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    fn verify_certificate_subject(&self, cert: &X509Certificate, producer_id: &str) -> Result<(), IdentityError> {
        let subject = cert.subject();
        let subject_str = subject.to_string();
        
        // Check if producer_id appears in subject (CN, OU, or O fields)
        // In production, this should match a specific field based on policy
        if !subject_str.contains(producer_id) {
            // Also check common name
            let cn = subject.iter_common_name()
                .next()
                .and_then(|attr| attr.as_str().ok())
                .unwrap_or("");
            
            if cn != producer_id {
                return Err(IdentityError::SubjectMismatch(
                    producer_id.to_string(),
                    subject_str
                ));
            }
        }
        
        Ok(())
    }
    
    fn verify_signature(&self, cert: &X509Certificate, envelope: &EventEnvelope) -> Result<(), IdentityError> {
        // Extract public key from certificate
        // x509-parser provides the public key in SubjectPublicKeyInfo format
        // For ring, we need the raw public key bytes
        let public_key_info = cert.public_key();
        
        // Verify it's RSA
        let public_key = public_key_info.parsed()
            .map_err(|e| IdentityError::CertificateParseError(
                format!("Failed to parse certificate public key: {}", e)
            ))?;
        
        match public_key {
            PublicKey::RSA(_) => {
                // RSA key - use the raw public key bytes
                // ring expects the public key in SubjectPublicKeyInfo format for UnparsedPublicKey
                let public_key_bytes = public_key_info.raw;
                
                // Serialize envelope for signing (same as signature.rs)
                let message = self.serialize_envelope(envelope)?;
                
                // Compute SHA-256 hash
                let mut hasher = Sha256::new();
                hasher.update(&message);
                let hash = hasher.finalize();
                
                // Decode signature
                let signature_bytes = base64::decode(&envelope.signature)
                    .map_err(|e| IdentityError::SignatureVerificationFailed(
                        format!("Failed to decode signature: {}", e)
                    ))?;
                
                // Verify signature using ring
                let unparsed_key = UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA256, public_key_bytes);
                
                unparsed_key.verify(&hash, &signature_bytes)
                    .map_err(|e| IdentityError::SignatureVerificationFailed(
                        format!("Signature verification failed: {}", e)
                    ))?;
            }
            _ => {
                return Err(IdentityError::SignatureVerificationFailed(
                    "Only RSA keys are supported".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    fn serialize_envelope(&self, envelope: &EventEnvelope) -> Result<Vec<u8>, IdentityError> {
        // Serialize envelope fields (excluding signature) for signing
        let mut data = Vec::new();
        data.extend_from_slice(envelope.producer_id.as_bytes());
        data.extend_from_slice(envelope.component_type.as_bytes());
        data.extend_from_slice(&envelope.schema_version.to_le_bytes());
        data.extend_from_slice(envelope.timestamp.to_rfc3339().as_bytes());
        data.extend_from_slice(&envelope.sequence_number.to_le_bytes());
        data.extend_from_slice(&envelope.integrity_hash.as_bytes());
        data.extend_from_slice(envelope.nonce.as_bytes());
        Ok(data)
    }
    
    fn extract_identity(&self, cert: &X509Certificate, producer_id: &str) -> Result<VerifiedIdentity, IdentityError> {
        let validity = cert.validity();
        
        let serial = cert.tbs_certificate.serial.to_bytes_be();
        let subject = cert.subject().to_string();
        let issuer = cert.issuer().to_string();
        
        let valid_from = DateTime::from_timestamp(validity.not_before.timestamp(), 0)
            .ok_or_else(|| IdentityError::CertificateParseError(
                "Invalid not_before timestamp".to_string()
            ))?;
        
        let valid_until = DateTime::from_timestamp(validity.not_after.timestamp(), 0)
            .ok_or_else(|| IdentityError::CertificateParseError(
                "Invalid not_after timestamp".to_string()
            ))?;
        
        let key_algorithm = match cert.public_key().parsed() {
            Ok(PublicKey::RSA(_)) => "RSA".to_string(),
            Ok(PublicKey::EC(_)) => "EC".to_string(),
            Ok(PublicKey::DSA(_)) => "DSA".to_string(),
            _ => "Unknown".to_string(),
        };
        
        Ok(VerifiedIdentity::new(
            producer_id.to_string(),
            serial,
            subject,
            issuer,
            valid_from,
            valid_until,
            key_algorithm,
        ))
    }
}
