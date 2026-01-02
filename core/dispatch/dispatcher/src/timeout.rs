// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/timeout.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Timeout handling for acknowledgment waiting

use tokio::time::{sleep, Duration, Instant};
use tracing::{debug, warn, error};
use super::acknowledgment_envelope::AcknowledgmentEnvelope;
use super::errors::DispatcherError;

pub struct TimeoutManager {
    default_timeout_seconds: u64,
}

impl TimeoutManager {
    pub fn new(default_timeout_seconds: u64) -> Self {
        Self {
            default_timeout_seconds,
        }
    }
    
    /// Wait for acknowledgment with timeout
    /// Returns Ok(ack) if received, Err(Timeout) if timeout exceeded
    pub async fn wait_for_acknowledgment<F>(
        &self,
        directive_id: &str,
        timeout_seconds: Option<u64>,
        check_fn: F,
    ) -> Result<AcknowledgmentEnvelope, DispatcherError>
    where
        F: Fn() -> Option<AcknowledgmentEnvelope>,
    {
        let timeout = timeout_seconds.unwrap_or(self.default_timeout_seconds);
        let start = Instant::now();
        let deadline = start + Duration::from_secs(timeout);
        
        debug!("Waiting for acknowledgment for directive {} (timeout: {}s)", directive_id, timeout);
        
        while Instant::now() < deadline {
            if let Some(ack) = check_fn() {
                if ack.directive_id == directive_id {
                    debug!("Acknowledgment received for directive {}", directive_id);
                    return Ok(ack);
                }
            }
            
            // Check every 100ms
            sleep(Duration::from_millis(100)).await;
        }
        
        error!("Acknowledgment timeout for directive {}", directive_id);
        Err(DispatcherError::AcknowledgmentTimeout(
            format!("No acknowledgment received within {} seconds", timeout)
        ))
    }
    
    /// Check if timeout has been exceeded
    pub fn is_timeout_exceeded(&self, start: Instant, timeout_seconds: u64) -> bool {
        Instant::now() - start > Duration::from_secs(timeout_seconds)
    }
}
