// Path and File Name : /home/ransomeye/rebuild/core/threat_feed/src/governance.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Feed governance - graceful degradation, feed health monitoring, fail-closed enforcement

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use tracing::{info, warn, error};

use crate::errors::ThreatFeedError;
use crate::ingestion::ThreatFeedIngester;

/// Feed health status
#[derive(Debug, Clone, PartialEq)]
pub enum FeedHealth {
    Healthy,
    Degraded,
    Unavailable,
}

/// Feed status
#[derive(Debug, Clone)]
pub struct FeedStatus {
    pub feed_name: String,
    pub health: FeedHealth,
    pub last_successful_fetch: Option<DateTime<Utc>>,
    pub last_fetch_attempt: Option<DateTime<Utc>>,
    pub consecutive_failures: u32,
    pub error_message: Option<String>,
}

/// Feed governor - manages feed health and graceful degradation
pub struct FeedGovernor {
    feeds: HashMap<String, FeedStatus>,
    max_consecutive_failures: u32,
    degradation_threshold_hours: i64,
}

impl FeedGovernor {
    /// Create new feed governor
    pub fn new(max_consecutive_failures: u32, degradation_threshold_hours: i64) -> Self {
        Self {
            feeds: HashMap::new(),
            max_consecutive_failures,
            degradation_threshold_hours,
        }
    }
    
    /// Register a feed
    pub fn register_feed(&mut self, feed_name: String) {
        self.feeds.insert(feed_name.clone(), FeedStatus {
            feed_name: feed_name.clone(),
            health: FeedHealth::Healthy,
            last_successful_fetch: Some(Utc::now()),
            last_fetch_attempt: Some(Utc::now()),
            consecutive_failures: 0,
            error_message: None,
        });
        
        info!("Registered feed: {}", feed_name);
    }
    
    /// Record successful feed fetch
    pub fn record_success(&mut self, feed_name: &str) {
        if let Some(status) = self.feeds.get_mut(feed_name) {
            status.health = FeedHealth::Healthy;
            status.last_successful_fetch = Some(Utc::now());
            status.last_fetch_attempt = Some(Utc::now());
            status.consecutive_failures = 0;
            status.error_message = None;
        }
    }
    
    /// Record failed feed fetch (graceful degradation)
    pub fn record_failure(&mut self, feed_name: &str, error: &str) {
        if let Some(status) = self.feeds.get_mut(feed_name) {
            status.last_fetch_attempt = Some(Utc::now());
            status.consecutive_failures += 1;
            status.error_message = Some(error.to_string());
            
            // Check if feed should be marked as degraded
            if status.consecutive_failures >= self.max_consecutive_failures {
                status.health = FeedHealth::Unavailable;
                error!("Feed {} marked as unavailable after {} consecutive failures", 
                       feed_name, status.consecutive_failures);
            } else if let Some(last_success) = status.last_successful_fetch {
                let age = Utc::now().signed_duration_since(last_success);
                if age.num_hours() > self.degradation_threshold_hours {
                    status.health = FeedHealth::Degraded;
                    warn!("Feed {} marked as degraded (last success: {} hours ago)", 
                          feed_name, age.num_hours());
                }
            } else {
                // No successful fetch ever - mark as unavailable
                status.health = FeedHealth::Unavailable;
            }
        }
    }
    
    /// Check if feed is available (graceful degradation - use cached data if unavailable)
    pub fn is_feed_available(&self, feed_name: &str) -> bool {
        self.feeds.get(feed_name)
            .map(|status| status.health != FeedHealth::Unavailable)
            .unwrap_or(false)
    }
    
    /// Get feed status
    pub fn get_feed_status(&self, feed_name: &str) -> Option<&FeedStatus> {
        self.feeds.get(feed_name)
    }
    
    /// Get all feed statuses
    pub fn get_all_statuses(&self) -> Vec<&FeedStatus> {
        self.feeds.values().collect()
    }
    
    /// Check if feed should use cached data (graceful degradation)
    pub fn should_use_cache(&self, feed_name: &str) -> bool {
        self.feeds.get(feed_name)
            .map(|status| status.health == FeedHealth::Degraded || status.health == FeedHealth::Unavailable)
            .unwrap_or(true) // Default to cache if feed not registered
    }
}

