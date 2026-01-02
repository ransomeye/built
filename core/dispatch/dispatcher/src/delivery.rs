// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/delivery.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Secure delivery of directives to agents

use tracing::{debug, error, info};
use super::directive_envelope::DirectiveEnvelope;
use super::errors::DispatcherError;
use super::router::AgentInfo;
use reqwest::Client;
use serde_json;

pub struct DeliveryService {
    http_client: Client,
}

impl DeliveryService {
    pub fn new() -> Result<Self, DispatcherError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| DispatcherError::InternalError(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            http_client: client,
        })
    }
    
    /// Deliver directive to agent
    pub async fn deliver(&self, directive: &DirectiveEnvelope, agent: &AgentInfo) -> Result<(), DispatcherError> {
        info!("Delivering directive {} to agent {}", directive.directive_id, agent.agent_id);
        
        let directive_json = serde_json::to_string(directive)
            .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
        
        // In production, would use mutual TLS or other secure transport
        let url = format!("{}/api/v1/directives", agent.api_url);
        
        debug!("Sending directive to {}", url);
        
        let response = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(directive_json)
            .send()
            .await
            .map_err(|e| DispatcherError::DeliveryFailed(format!("HTTP request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DispatcherError::DeliveryFailed(
                format!("Agent returned error: {} - {}", status, body)
            ));
        }
        
        info!("Directive {} delivered successfully to agent {}", directive.directive_id, agent.agent_id);
        Ok(())
    }
    
    /// Deliver directive in dry-run mode (simulation)
    pub async fn deliver_dry_run(&self, directive: &DirectiveEnvelope, agent: &AgentInfo) -> Result<(), DispatcherError> {
        debug!("DRY-RUN: Would deliver directive {} to agent {}", directive.directive_id, agent.agent_id);
        // In dry-run, we don't actually send
        Ok(())
    }
}
