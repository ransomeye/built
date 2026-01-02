// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/src/transport.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: mTLS transport client - sends signed events to Core with backpressure handling

use std::sync::Arc;
use std::fs;
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tracing::{error, warn, debug, info};
use thiserror::Error;
use crate::signing::SignedEvent;
use crate::config::Config;
use crate::backpressure::BackpressureHandler;
use crate::config::Config;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("TLS configuration failed: {0}")]
    TlsConfigFailed(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Backpressure signal received")]
    Backpressure,
}

pub struct TransportClient {
    config: Config,
    tls_config: Arc<ClientConfig>,
    backpressure: Arc<BackpressureHandler>,
}

impl TransportClient {
    pub fn new(config: Config, backpressure: Arc<BackpressureHandler>) -> Result<Self, TransportError> {
        // Load CA certificate
        let ca_cert_data = fs::read(&config.ca_cert_path)
            .map_err(|e| TransportError::TlsConfigFailed(format!("Failed to read CA cert: {}", e)))?;
        
        let mut root_store = RootCertStore::empty();
        let mut ca_reader = std::io::BufReader::new(ca_cert_data.as_slice());
        let ca_certs = certs(&mut ca_reader)
            .map_err(|e| TransportError::TlsConfigFailed(format!("Failed to parse CA cert: {}", e)))?;
        
        for cert in ca_certs {
            root_store.add(cert)
                .map_err(|e| TransportError::TlsConfigFailed(format!("Failed to add CA cert: {}", e)))?;
        }
        
        // Load client certificate and key
        let cert_data = fs::read(&config.cert_path)
            .map_err(|e| TransportError::TlsConfigFailed(format!("Failed to read cert: {}", e)))?;
        let key_data = fs::read(&config.key_path)
            .map_err(|e| TransportError::TlsConfigFailed(format!("Failed to read key: {}", e)))?;
        
        let mut cert_reader = std::io::BufReader::new(cert_data.as_slice());
        let certs = certs(&mut cert_reader)
            .map_err(|e| TransportError::TlsConfigFailed(format!("Failed to parse cert: {}", e)))?;
        
        let mut key_reader = std::io::BufReader::new(key_data.as_slice());
        let keys = pkcs8_private_keys(&mut key_reader)
            .map_err(|e| TransportError::TlsConfigFailed(format!("Failed to parse key: {}", e)))?;
        
        let key = keys.into_iter().next()
            .ok_or_else(|| TransportError::TlsConfigFailed("No private key found".to_string()))?
            .map_err(|e| TransportError::TlsConfigFailed(format!("Failed to parse key: {}", e)))?;
        
        // Build TLS config with client authentication
        let tls_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_client_auth_cert(certs, &key)
            .map_err(|e| TransportError::TlsConfigFailed(format!("Failed to build TLS config: {}", e)))?;
        
        Ok(Self {
            config,
            tls_config: Arc::new(tls_config),
            backpressure,
        })
    }
    
    pub async fn send_event(&self, event: &SignedEvent) -> Result<(), TransportError> {
        // Check backpressure
        if self.backpressure.is_backpressure_active() {
            return Err(TransportError::Backpressure);
        }
        
        // Check buffer capacity
        if !self.backpressure.can_accept() {
            return Err(TransportError::Backpressure);
        }
        
        // Serialize event
        let event_json = serde_json::to_vec(event)
            .map_err(|e| TransportError::SendFailed(format!("Serialization failed: {}", e)))?;
        
        // Increment buffer
        if !self.backpressure.increment_buffer(event_json.len()) {
            return Err(TransportError::Backpressure);
        }
        
        // Attempt to send
        match self.try_send(&event_json).await {
            Ok(()) => {
                self.backpressure.decrement_buffer(event_json.len());
                Ok(())
            }
            Err(TransportError::Backpressure) => {
                self.backpressure.set_backpressure(true);
                Err(TransportError::Backpressure)
            }
            Err(e) => {
                self.backpressure.decrement_buffer(event_json.len());
                Err(e)
            }
        }
    }
    
    async fn try_send(&self, data: &[u8]) -> Result<(), TransportError> {
        // Parse URL
        let url = url::Url::parse(&self.config.core_api_url)
            .map_err(|e| TransportError::ConnectionFailed(format!("Invalid URL: {}", e)))?;
        
        let host = url.host_str()
            .ok_or_else(|| TransportError::ConnectionFailed("No host in URL".to_string()))?;
        let port = url.port().unwrap_or(8443);
        
        // Connect TCP
        let addr = format!("{}:{}", host, port);
        let tcp_stream = TcpStream::connect(&addr).await
            .map_err(|e| TransportError::ConnectionFailed(format!("TCP connect failed: {}", e)))?;
        
        // Establish TLS
        let connector = TlsConnector::from(self.tls_config.clone());
        let mut tls_stream = connector.connect(host, tcp_stream).await
            .map_err(|e| TransportError::ConnectionFailed(format!("TLS handshake failed: {}", e)))?;
        
        // Send data
        tls_stream.write_all(data).await
            .map_err(|e| TransportError::SendFailed(format!("Write failed: {}", e)))?;
        
        // Read response (check for backpressure signal)
        let mut response = vec![0u8; 1024];
        match tls_stream.read(&mut response).await {
            Ok(0) => {
                // Connection closed
                Ok(())
            }
            Ok(n) => {
                let response_str = String::from_utf8_lossy(&response[..n]);
                if response_str.contains("RATE_LIMIT") || response_str.contains("BACKPRESSURE") {
                    warn!("Backpressure signal received from Core");
                    Err(TransportError::Backpressure)
                } else {
                    Ok(())
                }
            }
            Err(e) => {
                // Non-blocking read error - assume success if data was sent
                debug!("Response read error (non-fatal): {}", e);
                Ok(())
            }
        }
    }
    
    pub async fn health_check(&self) -> bool {
        // Simple health check - try to connect
        match self.try_send(b"HEALTH_CHECK").await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

