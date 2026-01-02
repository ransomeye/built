// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/src/rate_limit.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Rate limiting for event emission

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tracing::debug;

use super::errors::ProbeError;

/// Rate limiter
/// 
/// Token bucket algorithm for rate limiting.
/// Lock-free implementation.
pub struct RateLimiter {
    max_tokens: u64,
    tokens: AtomicU64,
    refill_rate: u64, // tokens per second
    last_refill: AtomicU64,
}

impl RateLimiter {
    /// Create new rate limiter
    pub fn new(max_tokens: u64, refill_rate: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            max_tokens,
            tokens: AtomicU64::new(max_tokens),
            refill_rate,
            last_refill: AtomicU64::new(now),
        }
    }
    
    /// Check if event can be emitted (non-blocking)
    pub fn allow(&self) -> Result<bool, ProbeError> {
        // Refill tokens
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ProbeError::RateLimitExceeded(format!("Time error: {}", e)))?
            .as_secs();
        
        let last = self.last_refill.load(Ordering::Acquire);
        let elapsed = now.saturating_sub(last);
        
        if elapsed > 0 {
            let to_add = elapsed * self.refill_rate;
            let current = self.tokens.load(Ordering::Relaxed);
            let new_tokens = (current + to_add).min(self.max_tokens);
            self.tokens.store(new_tokens, Ordering::Relaxed);
            self.last_refill.store(now, Ordering::Release);
        }
        
        // Try to consume token
        let mut current = self.tokens.load(Ordering::Relaxed);
        loop {
            if current == 0 {
                debug!("Rate limit exceeded: no tokens available");
                return Ok(false);
            }
            
            match self.tokens.compare_exchange_weak(
                current,
                current - 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Ok(true);
                }
                Err(actual) => {
                    current = actual;
                }
            }
        }
    }
    
    /// Get current token count
    pub fn tokens(&self) -> u64 {
        self.tokens.load(Ordering::Relaxed)
    }
}

