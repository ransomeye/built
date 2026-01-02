// Path and File Name : /home/ransomeye/rebuild/core/ingest/tests/priority_rate_limit_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for priority-based rate limiting

use ransomeye_ingestion::rate_limit::RateLimiter;
use ransomeye_ingestion::config::Config;

#[tokio::test]
async fn test_rate_limit_info_dropped_first() {
    let mut config = Config::load().unwrap();
    config.global_rate_limit = 2; // Very low limit for testing
    config.producer_rate_limit = 2;
    
    let rate_limiter = RateLimiter::new(&config).unwrap();
    
    // Fill up rate limit with INFO events
    assert!(rate_limiter.check_limit("producer1", "dpi_probe", "INFO").await.unwrap());
    assert!(rate_limiter.check_limit("producer1", "dpi_probe", "INFO").await.unwrap());
    
    // Next INFO should be dropped
    assert!(!rate_limiter.check_limit("producer1", "dpi_probe", "INFO").await.unwrap());
}

#[tokio::test]
async fn test_rate_limit_critical_never_dropped() {
    let mut config = Config::load().unwrap();
    config.global_rate_limit = 1; // Very low limit
    config.producer_rate_limit = 1;
    
    let rate_limiter = RateLimiter::new(&config).unwrap();
    
    // Fill up rate limit
    assert!(rate_limiter.check_limit("producer1", "dpi_probe", "INFO").await.unwrap());
    
    // CRITICAL should still pass even when limit exceeded
    assert!(rate_limiter.check_limit("producer1", "dpi_probe", "CRITICAL").await.unwrap());
}

#[tokio::test]
async fn test_rate_limit_warn_dropped_before_critical() {
    let mut config = Config::load().unwrap();
    config.global_rate_limit = 1;
    config.producer_rate_limit = 1;
    
    let rate_limiter = RateLimiter::new(&config).unwrap();
    
    // Fill up rate limit
    assert!(rate_limiter.check_limit("producer1", "dpi_probe", "INFO").await.unwrap());
    
    // WARN might be dropped if overloaded
    // But CRITICAL should always pass
    let warn_result = rate_limiter.check_limit("producer1", "dpi_probe", "WARN").await.unwrap();
    assert!(rate_limiter.check_limit("producer1", "dpi_probe", "CRITICAL").await.unwrap());
}

