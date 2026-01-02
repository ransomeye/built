// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/tests/rate_limit_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for active scanner rate limiting

use ransomeye_network_scanner::rate_limit::RateLimiter;

#[tokio::test]
async fn test_rate_limit_respected() {
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

#[tokio::test]
async fn test_rate_limit_token_refill() {
    let limiter = RateLimiter::new(10.0, 10);
    
    // Consume all tokens
    assert!(limiter.acquire(10.0).await.is_ok());
    limiter.release();
    
    // Wait for refill
    tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;
    
    // Should have tokens available
    assert!(limiter.available_tokens() > 0.0);
}

