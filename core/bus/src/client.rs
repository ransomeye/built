// Path and File Name : /home/ransomeye/rebuild/core/bus/src/client.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Message bus client with mTLS and ACL enforcement

use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::{TlsConnector, TlsStream};
use rustls::ClientConfig;
use tracing::{error, warn, debug, info};
use thiserror::Error;
use chrono::Utc;
use uuid::Uuid;

use crate::mtls::{load_client_cert, MtlsError};
use crate::acl::{Acl, ComponentRole, MessageType, AclError};
use crate::integrity::MessageIntegrity;

#[derive(Debug, Error)]
pub enum BusClientError {
    #[error("mTLS configuration failed: {0}")]
    MtlsFailed(#[from] MtlsError),
    #[error("ACL violation: {0}")]
    AclViolation(#[from] AclError),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BusMessage {
    pub message_id: String,
    pub component_id: String,
    pub component_role: ComponentRole,
    pub message_type: MessageType,
    pub topic: String,
    pub data: Vec<u8>,
    pub signature: String,
    pub timestamp: chrono::DateTime<Utc>,
}

pub struct BusClient {
    tls_config: Arc<ClientConfig>,
    component_role: ComponentRole,
    component_id: String,
    server_addr: String,
    integrity: Arc<MessageIntegrity>,
}

impl BusClient {
    /// Create new bus client with fail-closed mTLS enforcement
    /// 
    /// FAIL-CLOSED: Returns error if certificates are missing or invalid
    pub fn new(
        component_role: ComponentRole,
        component_id: String,
        server_addr: String,
    ) -> Result<Self, BusClientError> {
        // Require client certificate from environment
        let client_cert_path = std::env::var("RANSOMEYE_BUS_CLIENT_CERT")
            .map_err(|_| BusClientError::MtlsFailed(MtlsError::CertNotFound(
                "RANSOMEYE_BUS_CLIENT_CERT environment variable not set".to_string()
            )))?;
        
        let client_key_path = std::env::var("RANSOMEYE_BUS_CLIENT_KEY")
            .map_err(|_| BusClientError::MtlsFailed(MtlsError::KeyNotFound(
                "RANSOMEYE_BUS_CLIENT_KEY environment variable not set".to_string()
            )))?;
        
        let root_ca_path = std::env::var("RANSOMEYE_BUS_ROOT_CA_PATH")
            .map_err(|_| BusClientError::MtlsFailed(MtlsError::RootCANotFound(
                "RANSOMEYE_BUS_ROOT_CA_PATH environment variable not set".to_string()
            )))?;
        
        // Load mTLS configuration (fail-closed)
        let tls_config = load_client_cert(&client_cert_path, &client_key_path, &root_ca_path)?;
        
        info!("Bus client created for component: {} (role: {:?})", component_id, component_role);
        
        Ok(Self {
            tls_config: Arc::new(tls_config),
            component_role,
            component_id,
            server_addr,
            integrity: Arc::new(MessageIntegrity::new()),
        })
    }
    
    /// Publish message to bus
    /// 
    /// FAIL-CLOSED: Returns error on ACL violation or connection failure
    pub async fn publish(
        &self,
        topic: &str,
        message_type: MessageType,
        data: Vec<u8>,
    ) -> Result<(), BusClientError> {
        // Step 1: Check ACL
        Acl::can_publish(&self.component_role, &message_type)?;
        
        // Step 2: Create message
        let message_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        // Step 3: Sign message (in production, use actual signing key)
        // For now, create placeholder signature
        let signature = "placeholder_signature_base64_64_bytes_long_xxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        
        let message = BusMessage {
            message_id,
            component_id: self.component_id.clone(),
            component_role: self.component_role.clone(),
            message_type,
            topic: topic.to_string(),
            data,
            signature: signature.to_string(),
            timestamp,
        };
        
        // Step 4: Send message over mTLS
        self.send_message(&message).await?;
        
        Ok(())
    }
    
    async fn send_message(&self, message: &BusMessage) -> Result<(), BusClientError> {
        // Serialize message
        let message_json = serde_json::to_vec(message)
            .map_err(|e| BusClientError::SendFailed(format!("Serialization failed: {}", e)))?;
        
        // Connect to server
        let tcp_stream = TcpStream::connect(&self.server_addr).await
            .map_err(|e| BusClientError::ConnectionFailed(format!("TCP connect failed: {}", e)))?;
        
        // Establish mTLS connection
        let connector = TlsConnector::from(self.tls_config.clone());
        let host = self.server_addr.split(':').next().unwrap_or("localhost");
        let server_name = rustls::ServerName::try_from(host)
            .map_err(|_| BusClientError::ConnectionFailed(format!("Invalid server name: {}", host)))?;
        let mut tls_stream = connector.connect(server_name, tcp_stream).await
            .map_err(|e| BusClientError::ConnectionFailed(format!("TLS handshake failed: {}", e)))?;
        
        // Send message
        tls_stream.write_all(&message_json).await
            .map_err(|e| BusClientError::SendFailed(format!("Write failed: {}", e)))?;
        
        // Read response
        let mut response = vec![0u8; 1024];
        let n = tls_stream.read(&mut response).await
            .map_err(|e| BusClientError::ReceiveFailed(format!("Read failed: {}", e)))?;
        
        let response_str = String::from_utf8_lossy(&response[..n]);
        if response_str.contains("ACL_VIOLATION") {
            return Err(BusClientError::AclViolation(AclError::AccessDenied(
                "Server rejected message due to ACL violation".to_string()
            )));
        }
        
        if response_str.contains("REJECTED") {
            return Err(BusClientError::SendFailed("Server rejected message".to_string()));
        }
        
        debug!("Message published successfully: {}", message.message_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_fails_without_cert() {
        std::env::remove_var("RANSOMEYE_BUS_CLIENT_CERT");
        std::env::remove_var("RANSOMEYE_BUS_CLIENT_KEY");
        std::env::remove_var("RANSOMEYE_BUS_ROOT_CA_PATH");
        
        let result = BusClient::new(
            ComponentRole::Agent,
            "test-agent".to_string(),
            "localhost:8443".to_string(),
        );
        
        assert!(result.is_err());
    }
}

