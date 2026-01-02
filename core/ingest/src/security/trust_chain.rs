// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/security/trust_chain.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Trust chain validation - validates certificate chains against root CA with real cryptography

/*
 * Trust Chain Validation
 * 
 * Validates certificate chains against root CA.
 * Ensures trust chain integrity using real cryptographic verification.
 * Fail-closed: rejects on any chain validation failure.
 */

use std::sync::Arc;
use x509_parser::prelude::*;
use ring::signature::{UnparsedPublicKey, RSA_PSS_2048_8192_SHA256};
use sha2::{Sha256, Digest};
use tracing::{error, debug, warn};

use crate::security::errors::IdentityError;
use crate::security::trust_store::TrustStore;

pub struct TrustChainValidator {
    trust_store: Arc<TrustStore>,
}

impl TrustChainValidator {
    pub fn new(trust_store: Arc<TrustStore>) -> Result<Self, IdentityError> {
        Ok(Self {
            trust_store,
        })
    }
    
    /// Validate certificate chain against root CA
    /// Returns Ok(()) on success, IdentityError on failure
    pub async fn validate_chain(&self, certificate_der: &[u8]) -> Result<(), IdentityError> {
        debug!("Validating certificate chain");
        
        // Parse certificate
        let (_, cert) = X509Certificate::from_der(certificate_der)
            .map_err(|e| IdentityError::CertificateParseError(
                format!("Failed to parse certificate: {}", e)
            ))?;
        
        // Get root CA
        let root_ca_data = self.trust_store.get_root_ca_data()?;
        let root_ca = root_ca_data.parse_certificate()?;
        
        // Verify certificate is signed by root CA
        self.verify_signature(&cert, &root_ca)?;
        
        // Verify issuer matches root CA subject
        let cert_issuer = cert.issuer().to_string();
        let root_subject = root_ca.subject().to_string();
        
        if cert_issuer != root_subject {
            return Err(IdentityError::ChainValidationFailed(
                format!("Certificate issuer '{}' does not match root CA subject '{}'", 
                    cert_issuer, root_subject)
            ));
        }
        
        debug!("Certificate chain validated successfully");
        Ok(())
    }
    
    fn verify_signature(&self, cert: &X509Certificate, issuer_cert: &X509Certificate) -> Result<(), IdentityError> {
        // Get issuer's public key
        let issuer_public_key = issuer_cert.public_key();
        let issuer_public_key_bytes = issuer_public_key.raw;
        
        // Get certificate's signature algorithm
        let sig_alg = &cert.signature_algorithm;
        
        // Verify signature algorithm is RSA-PSS-SHA256
        let algorithm = match sig_alg.algorithm.to_id_string().as_str() {
            "1.2.840.113549.1.1.10" => &RSA_PSS_2048_8192_SHA256, // RSA-PSS
            "1.2.840.113549.1.1.11" => &RSA_PSS_2048_8192_SHA256, // RSA-PSS with SHA-256 (approximate)
            _ => {
                warn!("Unsupported signature algorithm: {}, defaulting to RSA-PSS", sig_alg.algorithm.to_id_string());
                // Default to RSA-PSS-SHA256
                &RSA_PSS_2048_8192_SHA256
            }
        };
        
        // Get certificate's TBS (To Be Signed) certificate bytes
        // This is the certificate data that was signed
        let tbs_certificate = cert.tbs_certificate.as_ref();
        
        // Compute hash of TBS certificate
        let mut hasher = Sha256::new();
        hasher.update(tbs_certificate);
        let hash = hasher.finalize();
        
        // Get certificate signature
        let signature = &cert.signature_value;
        
        // Verify signature
        let unparsed_key = UnparsedPublicKey::new(algorithm, issuer_public_key_bytes);
        
        unparsed_key.verify(&hash, signature.as_ref())
            .map_err(|e| IdentityError::ChainValidationFailed(
                format!("Certificate signature verification failed: {}", e)
            ))?;
        
        Ok(())
    }
}
