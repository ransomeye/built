// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/src/rate_limit.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Rate limiting for active scanner - tokens/sec, concurrency caps

use std::sync::Arc;
use parking_lot::Mutex;
use tokio::time::{Duration, Instant};
use tracing::{warn, debug};

pub struct RateLimiter {
    tokens_per_second: f64,
    max_concurrent: usize,
    tokens: Arc<Mutex<f64>>,
    last_refill: Arc<Mutex<Instant>>,
    active_scans: Arc<Mutex<usize>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(tokens_per_second: f64, max_concurrent: usize) -> Self {
        Self {
            tokens_per_second,
            max_concurrent,
            tokens: Arc::new(Mutex::new(tokens_per_second)),
            last_refill: Arc::new(Mutex::new(Instant::now())),
            active_scans: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Acquire tokens (blocking until available)
    pub async fn acquire(&self, tokens: f64) -> Result<(), crate::errors::ScannerError> {
        // Check concurrency limit
        {
            let mut active = self.active_scans.lock();
            if *active >= self.max_concurrent {
                return Err(crate::errors::ScannerError::RateLimitExceeded(
                    format!("Max concurrent scans ({}) exceeded", self.max_concurrent)
                ));
            }
            *active += 1;
        }
        
        // Refill tokens based on elapsed time
        {
            let mut tokens_guard = self.tokens.lock();
            let mut last_refill_guard = self.last_refill.lock();
            
            let now = Instant::now();
            let elapsed = now.duration_since(*last_refill_guard);
            
            if elapsed.as_secs_f64() > 0.0 {
                let refill = elapsed.as_secs_f64() * self.tokens_per_second;
                *tokens_guard = (*tokens_guard + refill).min(self.tokens_per_second);
                *last_refill_guard = now;
            }
            
            // Wait until we have enough tokens
            while *tokens_guard < tokens {
                let needed = tokens - *tokens_guard;
                let wait_time = needed / self.tokens_per_second;
                
                drop(tokens_guard);
                drop(last_refill_guard);
                
                tokio::time::sleep(Duration::from_secs_f64(wait_time)).await;
                
                tokens_guard = self.tokens.lock();
                last_refill_guard = self.last_refill.lock();
                
                let now = Instant::now();
                let elapsed = now.duration_since(*last_refill_guard);
                
                if elapsed.as_secs_f64() > 0.0 {
                    let refill = elapsed.as_secs_f64() * self.tokens_per_second;
                    *tokens_guard = (*tokens_guard + refill).min(self.tokens_per_second);
                    *last_refill_guard = now;
                }
            }
            
            // Consume tokens
            *tokens_guard -= tokens;
        }
        
        Ok(())
    }
    
    /// Release concurrency slot
    pub fn release(&self) {
        let mut active = self.active_scans.lock();
        if *active > 0 {
            *active -= 1;
        }
    }
    
    /// Get current token count
    pub fn available_tokens(&self) -> f64 {
        let mut tokens_guard = self.tokens.lock();
        let mut last_refill_guard = self.last_refill.lock();
        
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill_guard);
        
        if elapsed.as_secs_f64() > 0.0 {
            let refill = elapsed.as_secs_f64() * self.tokens_per_second;
            *tokens_guard = (*tokens_guard + refill).min(self.tokens_per_second);
            *last_refill_guard = now;
        }
        
        *tokens_guard
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(10.0, 5);
        
        // Should acquire tokens immediately
        assert!(limiter.acquire(5.0).await.is_ok());
        limiter.release();
        
        // Should respect concurrency limit
        for _ in 0..5 {
            assert!(limiter.acquire(1.0).await.is_ok());
        }
        
        // Should fail on 6th concurrent scan
        assert!(limiter.acquire(1.0).await.is_err());
        
        // Release one
        limiter.release();
        
        // Should succeed again
        assert!(limiter.acquire(1.0).await.is_ok());
        limiter.release();
    }
}

