// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/rate_limit.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Deterministic rate limiting - per-producer, per-component, and global limits

/*
 * Rate Limiting
 * 
 * Enforces deterministic rate limits:
 * - Per-producer rate limits
 * - Per-component quotas
 * - Global ingestion caps
 * 
 * Uses fixed windows and deterministic counters.
 * No adaptive heuristics.
 */

use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use parking_lot::RwLock;
use tracing::{warn, debug};

use crate::config::Config;

pub struct RateLimiter {
    config: Config,
    producer_limits: Arc<DashMap<String, ProducerLimit>>,
    component_quotas: Arc<DashMap<String, ComponentQuota>>,
    global_cap: Arc<RwLock<GlobalCap>>,
}

struct ProducerLimit {
    count: u64,
    window_start: Instant,
    limit: u64,
    window_duration: Duration,
}

struct ComponentQuota {
    count: u64,
    window_start: Instant,
    quota: u64,
    window_duration: Duration,
}

struct GlobalCap {
    count: u64,
    window_start: Instant,
    cap: u64,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config: config.clone(),
            producer_limits: Arc::new(DashMap::new()),
            component_quotas: Arc::new(DashMap::new()),
            global_cap: Arc::new(RwLock::new(GlobalCap {
                count: 0,
                window_start: Instant::now(),
                cap: config.global_rate_limit,
                window_duration: Duration::from_secs(config.rate_limit_window_seconds),
            })),
        })
    }
    
    pub async fn check_limit(&self, producer_id: &str, component_type: &str, priority: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Validate priority (fail-closed on invalid priority)
        let priority_upper = priority.to_uppercase();
        if priority_upper != "INFO" && priority_upper != "WARN" && priority_upper != "CRITICAL" {
            warn!("Invalid priority: {}, defaulting to INFO", priority);
            // Fail-closed: treat invalid priority as INFO (lowest)
        }
        
        // Check global cap (priority-aware)
        let global_result = self.check_global_cap_priority(&priority_upper).await;
        if !global_result {
            // Rate limit exceeded - check if we can drop this priority
            if priority_upper == "CRITICAL" {
                // CRITICAL never dropped - force through
                warn!("Global rate limit exceeded but CRITICAL priority - forcing through");
                return Ok(true);
            } else if priority_upper == "WARN" {
                // Check if we should drop WARN
                if self.should_drop_warn().await {
                    warn!("Rate limit exceeded - dropping WARN priority event");
                    return Ok(false);
                } else {
                    // Force through WARN if we haven't dropped too many
                    return Ok(true);
                }
            } else {
                // INFO - drop first
                warn!("Rate limit exceeded - dropping INFO priority event");
                return Ok(false);
            }
        }
        
        // Check producer limit (priority-aware)
        if !self.check_producer_limit_priority(producer_id, &priority_upper).await {
            if priority_upper == "CRITICAL" {
                warn!("Producer rate limit exceeded but CRITICAL priority - forcing through");
                return Ok(true);
            } else if priority_upper == "WARN" {
                if self.should_drop_warn().await {
                    warn!("Producer rate limit exceeded - dropping WARN priority event");
                    return Ok(false);
                } else {
                    return Ok(true);
                }
            } else {
                warn!("Producer rate limit exceeded - dropping INFO priority event");
                return Ok(false);
            }
        }
        
        // Check component quota (priority-aware)
        if !self.check_component_quota_priority(component_type, &priority_upper).await {
            if priority_upper == "CRITICAL" {
                warn!("Component quota exceeded but CRITICAL priority - forcing through");
                return Ok(true);
            } else if priority_upper == "WARN" {
                if self.should_drop_warn().await {
                    warn!("Component quota exceeded - dropping WARN priority event");
                    return Ok(false);
                } else {
                    return Ok(true);
                }
            } else {
                warn!("Component quota exceeded - dropping INFO priority event");
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    async fn check_global_cap_priority(&self, priority: &str) -> bool {
        let mut cap = self.global_cap.write();
        let now = Instant::now();
        
        // Reset window if expired
        if now.duration_since(cap.window_start) >= cap.window_duration {
            cap.count = 0;
            cap.window_start = now;
        }
        
        // CRITICAL always passes
        if priority == "CRITICAL" {
            return true;
        }
        
        // Check limit
        if cap.count >= cap.cap {
            return false;
        }
        
        cap.count += 1;
        true
    }
    
    async fn should_drop_warn(&self) -> bool {
        // Drop WARN only if we're really overloaded
        // For now, use a simple heuristic: drop WARN if global cap is > 90%
        let cap = self.global_cap.read();
        let utilization = (cap.count as f64) / (cap.cap as f64);
        utilization > 0.9
    }
    
    async fn check_producer_limit_priority(&self, producer_id: &str, priority: &str) -> bool {
        // CRITICAL always passes
        if priority == "CRITICAL" {
            return true;
        }
        
        self.check_producer_limit(producer_id).await
    }
    
    async fn check_component_quota_priority(&self, component_type: &str, priority: &str) -> bool {
        // CRITICAL always passes
        if priority == "CRITICAL" {
            return true;
        }
        
        self.check_component_quota(component_type).await
    }
    
    async fn check_global_cap(&self) -> bool {
        let mut cap = self.global_cap.write();
        let now = Instant::now();
        
        // Reset window if expired
        if now.duration_since(cap.window_start) >= cap.window_duration {
            cap.count = 0;
            cap.window_start = now;
        }
        
        // Check limit
        if cap.count >= cap.cap {
            return false;
        }
        
        cap.count += 1;
        true
    }
    
    async fn check_producer_limit(&self, producer_id: &str) -> bool {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.rate_limit_window_seconds);
        let limit = self.config.producer_rate_limit;
        
        // Get or create producer limit
        let mut producer_limit = self.producer_limits
            .entry(producer_id.to_string())
            .or_insert_with(|| ProducerLimit {
                count: 0,
                window_start: now,
                limit,
                window_duration,
            });
        
        // Reset window if expired
        if now.duration_since(producer_limit.window_start) >= producer_limit.window_duration {
            producer_limit.count = 0;
            producer_limit.window_start = now;
        }
        
        // Check limit
        if producer_limit.count >= producer_limit.limit {
            return false;
        }
        
        producer_limit.count += 1;
        true
    }
    
    async fn check_component_quota(&self, component_type: &str) -> bool {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.rate_limit_window_seconds);
        // Default component quota: same as producer limit (can be configured per component)
        let quota = self.config.producer_rate_limit;
        
        // Get or create component quota
        let mut component_quota = self.component_quotas
            .entry(component_type.to_string())
            .or_insert_with(|| ComponentQuota {
                count: 0,
                window_start: now,
                quota,
                window_duration,
            });
        
        // Reset window if expired
        if now.duration_since(component_quota.window_start) >= component_quota.window_duration {
            component_quota.count = 0;
            component_quota.window_start = now;
        }
        
        // Check quota
        if component_quota.count >= component_quota.quota {
            return false;
        }
        
        component_quota.count += 1;
        true
    }
}

