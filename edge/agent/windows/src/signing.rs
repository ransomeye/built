// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/src/signing.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Event signing using RSA-4096-PSS-SHA256 - all telemetry must be signed

use ring::signature::{RsaKeyPair, RSA_PSS_SHA256};
use ring::rand::{SystemRandom, SecureRandom};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use base64;
use hex;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SigningError {
    #[error("Failed to sign data: {0}")]
    SigningFailed(String),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEvent {
    pub message_id: String,
    pub timestamp: DateTime<Utc>,
    pub nonce: String,
    pub component_identity: String,
    pub host_id: String,
    pub data: serde_json::Value,
    pub signature: String,
    pub data_hash: String,
}

pub struct EventSigner {
    keypair: RsaKeyPair,
    producer_id: String,
    host_id: String,
    rng: SystemRandom,
    sequence_number: std::sync::atomic::AtomicU64,
}

impl EventSigner {
    pub fn new(keypair: RsaKeyPair, producer_id: String) -> Self {
        let host_id = Self::get_host_id();
        Self {
            keypair,
            producer_id,
            host_id,
            rng: SystemRandom::new(),
            sequence_number: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    fn get_host_id() -> String {
        // Get Windows machine GUID from registry
        #[cfg(windows)]
        {
            use winapi::um::winreg::{RegOpenKeyExA, RegQueryValueExA, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ};
            use std::ffi::CString;
            use std::os::raw::c_char;
            
            let key_name = CString::new("SOFTWARE\\Microsoft\\Cryptography").unwrap();
            let value_name = CString::new("MachineGuid").unwrap();
            let mut hkey: winapi::um::winreg::HKEY = std::ptr::null_mut();
            let mut data: [u8; 256] = [0; 256];
            let mut data_len: u32 = 256;
            
            unsafe {
                if RegOpenKeyExA(
                    HKEY_LOCAL_MACHINE,
                    key_name.as_ptr(),
                    0,
                    KEY_READ,
                    &mut hkey,
                ) == 0 {
                    if RegQueryValueExA(
                        hkey,
                        value_name.as_ptr(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        data.as_mut_ptr() as *mut c_char,
                        &mut data_len,
                    ) == 0 {
                        let guid = String::from_utf8_lossy(&data[..data_len as usize - 1]);
                        return guid.to_string();
                    }
                }
            }
        }
        
        // Fallback to hostname
        #[cfg(windows)]
        {
            use std::ffi::OsString;
            use std::os::windows::ffi::OsStringExt;
            use winapi::um::winnls::GetComputerNameA;
            
            let mut buffer: [u8; 256] = [0; 256];
            let mut size = 256u32;
            
            unsafe {
                if GetComputerNameA(buffer.as_mut_ptr() as *mut i8, &mut size) != 0 {
                    return String::from_utf8_lossy(&buffer[..size as usize]).to_string();
                }
            }
        }
        
        "unknown".to_string()
    }
    
    pub fn sign_event(&self, data: serde_json::Value) -> Result<SignedEvent, SigningError> {
        // Generate unique message ID and nonce
        let message_id = Uuid::new_v4().to_string();
        let nonce = self.generate_nonce()?;
        let timestamp = Utc::now();
        
        // Compute data hash
        let data_json = serde_json::to_vec(&data)
            .map_err(|e| SigningError::SerializationFailed(format!("{}", e)))?;
        let mut hasher = Sha256::new();
        hasher.update(&data_json);
        let data_hash = hex::encode(hasher.finalize());
        
        // Serialize envelope for signing (excluding signature)
        let envelope_data = self.serialize_envelope(&message_id, &timestamp, &nonce, &data_hash)?;
        
        // Compute SHA-256 hash of envelope
        let mut hasher = Sha256::new();
        hasher.update(&envelope_data);
        let hash = hasher.finalize();
        
        // Sign hash with RSA-4096-PSS-SHA256
        let mut signature_bytes = vec![0u8; self.keypair.public_modulus_len()];
        self.keypair.sign(&RSA_PSS_SHA256, &self.rng, &hash, &mut signature_bytes)
            .map_err(|e| SigningError::SigningFailed(format!("{}", e)))?;
        
        let signature = base64::encode(&signature_bytes);
        
        Ok(SignedEvent {
            message_id,
            timestamp,
            nonce,
            component_identity: self.producer_id.clone(),
            host_id: self.host_id.clone(),
            data,
            signature,
            data_hash,
        })
    }
    
    fn serialize_envelope(
        &self,
        message_id: &str,
        timestamp: &DateTime<Utc>,
        nonce: &str,
        data_hash: &str,
    ) -> Result<Vec<u8>, SigningError> {
        let mut data = Vec::new();
        data.extend_from_slice(self.producer_id.as_bytes());
        data.extend_from_slice(b"windows_agent");
        data.extend_from_slice(message_id.as_bytes());
        data.extend_from_slice(timestamp.to_rfc3339().as_bytes());
        data.extend_from_slice(nonce.as_bytes());
        data.extend_from_slice(data_hash.as_bytes());
        Ok(data)
    }
    
    fn generate_nonce(&self) -> Result<String, SigningError> {
        let mut bytes = vec![0u8; 32];
        self.rng.fill(&mut bytes)
            .map_err(|e| SigningError::SigningFailed(format!("Failed to generate nonce: {}", e)))?;
        Ok(hex::encode(&bytes))
    }
    
    pub fn next_sequence_number(&self) -> u64 {
        self.sequence_number.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

