// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/security/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Security error types - defines all security-related error types for identity verification

/*
 * Security Error Types
 * 
 * Defines all security-related error types for identity verification.
 * Fail-closed behavior: all errors result in rejection.
 */

use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum IdentityError {
    #[error("Certificate not found for producer: {0}")]
    CertificateNotFound(String),
    
    #[error("Certificate parsing failed: {0}")]
    CertificateParseError(String),
    
    #[error("Certificate chain validation failed: {0}")]
    ChainValidationFailed(String),
    
    #[error("Certificate expired: {0}")]
    CertificateExpired(String),
    
    #[error("Certificate not yet valid: {0}")]
    CertificateNotYetValid(String),
    
    #[error("Certificate revoked: {0}")]
    CertificateRevoked(String),
    
    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),
    
    #[error("Invalid key usage: {0}")]
    InvalidKeyUsage(String),
    
    #[error("Unknown certificate authority: {0}")]
    UnknownCertificateAuthority(String),
    
    #[error("Replay attack detected: {0}")]
    ReplayAttack(String),
    
    #[error("Timestamp out of tolerance: {0}")]
    TimestampOutOfTolerance(String),
    
    #[error("Sequence number violation: {0}")]
    SequenceNumberViolation(String),
    
    #[error("Trust store initialization failed: {0}")]
    TrustStoreInitFailed(String),
    
    #[error("Root CA not found or invalid")]
    RootCANotFound,
    
    #[error("Producer certificate not in trust store: {0}")]
    ProducerNotInTrustStore(String),
    
    #[error("Certificate subject mismatch: expected {0}, got {1}")]
    SubjectMismatch(String, String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, Clone)]
pub struct VerifiedIdentity {
    pub producer_id: String,
    pub certificate_serial: Vec<u8>,
    pub certificate_subject: String,
    pub certificate_issuer: String,
    pub valid_from: chrono::DateTime<chrono::Utc>,
    pub valid_until: chrono::DateTime<chrono::Utc>,
    pub key_algorithm: String,
    pub verified_at: chrono::DateTime<chrono::Utc>,
}

impl VerifiedIdentity {
    pub fn new(
        producer_id: String,
        certificate_serial: Vec<u8>,
        certificate_subject: String,
        certificate_issuer: String,
        valid_from: chrono::DateTime<chrono::Utc>,
        valid_until: chrono::DateTime<chrono::Utc>,
        key_algorithm: String,
    ) -> Self {
        Self {
            producer_id,
            certificate_serial,
            certificate_subject,
            certificate_issuer,
            valid_from,
            valid_until,
            key_algorithm,
            verified_at: chrono::Utc::now(),
        }
    }
}

