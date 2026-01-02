// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/security/trust_store.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Trust store - loads and manages root CA and producer certificates with immutability enforcement

/*
 * Trust Store
 * 
 * Loads and manages root CA and producer certificates.
 * Enforces immutability at runtime.
 * Rejects runtime injection.
 */

use std::sync::Arc;
use std::path::Path;
use std::fs;
use dashmap::DashMap;
use x509_parser::prelude::*;
use rustls_pemfile::{Item, read_one};
use std::io::Cursor;
use tracing::{error, info, warn};
use parking_lot::RwLock;

use crate::security::errors::IdentityError;

/// Certificate data stored in trust store
#[derive(Clone)]
pub struct CertificateData {
    pub der: Vec<u8>,
}

impl CertificateData {
    pub fn new(der: Vec<u8>) -> Result<Self, IdentityError> {
        // Validate that it parses correctly
        let (_, _) = X509Certificate::from_der(&der)
            .map_err(|e| IdentityError::CertificateParseError(
                format!("X.509 parsing failed: {}", e)
            ))?;
        Ok(Self { der })
    }
    
    pub fn parse_certificate<'a>(&'a self) -> Result<X509Certificate<'a>, IdentityError> {
        // Parse from DER - the certificate lifetime is tied to self
        let (_, cert) = X509Certificate::from_der(&self.der)
            .map_err(|e| IdentityError::CertificateParseError(
                format!("Certificate re-parsing failed: {}", e)
            ))?;
        Ok(cert)
    }
}

#[derive(Clone)]
pub struct TrustStore {
    root_ca: Arc<RwLock<Option<CertificateData>>>,
    producer_certificates: Arc<DashMap<String, CertificateData>>,
    initialized: Arc<RwLock<bool>>,
    trust_store_path: String,
}

impl TrustStore {
    pub fn new(trust_store_path: &str) -> Result<Self, IdentityError> {
        Ok(Self {
            root_ca: Arc::new(RwLock::new(None)),
            producer_certificates: Arc::new(DashMap::new()),
            initialized: Arc::new(RwLock::new(false)),
            trust_store_path: trust_store_path.to_string(),
        })
    }
    
    /// Initialize trust store - loads root CA and producer certificates
    /// This is called once at startup and cannot be called again
    pub fn initialize(&self) -> Result<(), IdentityError> {
        let mut initialized = self.initialized.write();
        if *initialized {
            return Err(IdentityError::InternalError(
                "Trust store already initialized".to_string()
            ));
        }
        
        info!("Initializing trust store from: {}", self.trust_store_path);
        
        // Load root CA
        let root_ca_path = Path::new(&self.trust_store_path).join("root_ca.pem");
        if !root_ca_path.exists() {
            return Err(IdentityError::RootCANotFound);
        }
        
        let root_ca_bytes = fs::read(&root_ca_path)
            .map_err(|e| IdentityError::TrustStoreInitFailed(
                format!("Failed to read root CA: {}", e)
            ))?;
        
        let root_ca = Self::parse_certificate(&root_ca_bytes)?;
        *self.root_ca.write() = Some(root_ca);
        info!("Root CA loaded successfully");
        
        // Load producer certificates
        let producer_certs_dir = Path::new(&self.trust_store_path).join("producers");
        if producer_certs_dir.exists() {
            self.load_producer_certificates(&producer_certs_dir)?;
        } else {
            warn!("Producer certificates directory not found: {:?}", producer_certs_dir);
        }
        
        *initialized = true;
        info!("Trust store initialized successfully");
        Ok(())
    }
    
    fn load_producer_certificates(&self, certs_dir: &Path) -> Result<(), IdentityError> {
        let entries = fs::read_dir(certs_dir)
            .map_err(|e| IdentityError::TrustStoreInitFailed(
                format!("Failed to read producer certificates directory: {}", e)
            ))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| IdentityError::TrustStoreInitFailed(
                format!("Failed to read directory entry: {}", e)
            ))?;
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("pem") {
                let producer_id = path.file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| IdentityError::TrustStoreInitFailed(
                        "Invalid certificate filename".to_string()
                    ))?
                    .to_string();
                
                let cert_bytes = fs::read(&path)
                    .map_err(|e| IdentityError::TrustStoreInitFailed(
                        format!("Failed to read certificate {}: {}", producer_id, e)
                    ))?;
                
                let cert = Self::parse_certificate(&cert_bytes)?;
                self.producer_certificates.insert(producer_id.clone(), cert);
                info!("Loaded producer certificate: {}", producer_id);
            }
        }
        
        Ok(())
    }
    
    fn parse_certificate(bytes: &[u8]) -> Result<CertificateData, IdentityError> {
        // Try parsing as PEM first
        let der_bytes = if bytes.starts_with(b"-----BEGIN") {
            // Parse PEM format using rustls-pemfile
            let mut cursor = Cursor::new(bytes);
            match read_one(&mut cursor) {
                Ok(Some(Item::X509Certificate(der))) => der,
                Ok(Some(_)) => {
                    return Err(IdentityError::CertificateParseError(
                        "PEM file contains a key or other item, not a certificate".to_string()
                    ));
                }
                Ok(None) => {
                    return Err(IdentityError::CertificateParseError(
                        "No certificate found in PEM file".to_string()
                    ));
                }
                Err(e) => {
                    return Err(IdentityError::CertificateParseError(
                        format!("PEM parsing failed: {}", e)
                    ));
                }
            }
        } else {
            // Assume DER format
            bytes.to_vec()
        };
        
        // Store DER bytes and create CertificateData
        CertificateData::new(der_bytes)
    }
    
    /// Get root CA certificate data
    pub fn get_root_ca_data(&self) -> Result<CertificateData, IdentityError> {
        let guard = self.root_ca.read();
        guard
            .as_ref()
            .ok_or(IdentityError::RootCANotFound)
            .cloned()
    }
    
    /// Get producer certificate data by ID
    pub fn get_producer_certificate_data_cloned(&self, producer_id: &str) -> Result<CertificateData, IdentityError> {
        self.producer_certificates
            .get(producer_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| IdentityError::ProducerNotInTrustStore(producer_id.to_string()))
    }
    
    /// Get producer certificate data (for signature verification)
    pub fn get_producer_certificate_data(&self, producer_id: &str) -> Result<CertificateData, IdentityError> {
        self.producer_certificates
            .get(producer_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| IdentityError::ProducerNotInTrustStore(producer_id.to_string()))
    }
    
    /// Check if producer certificate exists in trust store
    pub fn has_producer(&self, producer_id: &str) -> bool {
        self.producer_certificates.contains_key(producer_id)
    }
    
    /// Get all producer IDs in trust store
    pub fn list_producers(&self) -> Vec<String> {
        self.producer_certificates
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
    
    /// Verify that trust store is initialized
    pub fn is_initialized(&self) -> bool {
        *self.initialized.read()
    }
}

