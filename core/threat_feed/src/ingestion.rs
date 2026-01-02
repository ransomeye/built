// Path and File Name : /home/ransomeye/rebuild/core/threat_feed/src/ingestion.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Threat intel ingestion - IOCs, TTPs, campaigns with signature verification and fail-closed validation

use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::path::Path;
use std::time::SystemTime;
use tracing::{info, warn, error, debug};
use sha2::{Sha256, Digest};
use hex;
use ring::signature::{UnparsedPublicKey, VerificationAlgorithm};
use ring::signature::RSA_PSS_2048_8192_SHA256;

use crate::errors::ThreatFeedError;
use crate::validation::FeedValidator;

/// Threat intel bundle with signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelBundle {
    pub bundle_id: String,
    pub source: String,
    pub source_reputation: f64, // 0.0 to 1.0
    pub timestamp: DateTime<Utc>,
    pub iocs: Vec<IOC>,
    pub ttps: Vec<TTP>,
    pub campaigns: Vec<Campaign>,
    pub signature: String,
    pub signature_algorithm: String,
    pub public_key_id: String,
}

/// Indicator of Compromise
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IOC {
    pub ioc_id: String,
    pub ioc_type: IOCType,
    pub value: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub confidence: f64,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
}

/// IOC types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IOCType {
    IP,
    Domain,
    HashMD5,
    HashSHA1,
    HashSHA256,
    URL,
    Email,
    FilePath,
}

/// Tactics, Techniques, and Procedures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTP {
    pub ttp_id: String,
    pub mitre_id: String, // e.g., "T1055"
    pub name: String,
    pub description: String,
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Campaign indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub campaign_id: String,
    pub name: String,
    pub description: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub associated_iocs: Vec<String>,
    pub associated_ttps: Vec<String>,
    pub confidence: f64,
    pub metadata: serde_json::Value,
}

/// Threat feed ingester with fail-closed validation
pub struct ThreatFeedIngester {
    validator: FeedValidator,
    seen_bundles: HashSet<String>, // For replay detection
    max_freshness_hours: i64,
    trust_store_path: String,
}

impl ThreatFeedIngester {
    /// Create new threat feed ingester
    pub fn new(trust_store_path: impl AsRef<Path>, max_freshness_hours: i64) -> Result<Self, ThreatFeedError> {
        let path = trust_store_path.as_ref();
        let validator = FeedValidator::new(path)?;
        
        Ok(Self {
            validator,
            seen_bundles: HashSet::new(),
            max_freshness_hours,
            trust_store_path: path.to_string_lossy().to_string(),
        })
    }
    
    /// Ingest threat intel bundle with fail-closed validation
    /// 
    /// FAIL-CLOSED: Returns error if:
    /// - Signature invalid
    /// - Schema invalid
    /// - Intel stale
    /// - Replay detected
    /// - Source attribution missing
    pub fn ingest_bundle(&mut self, bundle: ThreatIntelBundle) -> Result<ProcessedIntel, ThreatFeedError> {
        // Step 1: Verify signature (FAIL-CLOSED)
        self.validator.verify_signature(&bundle)
            .map_err(|e| ThreatFeedError::InvalidSignature(format!("Signature verification failed: {}", e)))?;
        
        // Step 2: Validate schema (FAIL-CLOSED)
        self.validator.validate_schema(&bundle)
            .map_err(|e| ThreatFeedError::SchemaValidationFailed(format!("Schema validation failed: {}", e)))?;
        
        // Step 3: Check freshness (FAIL-CLOSED)
        let now = Utc::now();
        let age = now.signed_duration_since(bundle.timestamp);
        if age.num_hours() > self.max_freshness_hours {
            return Err(ThreatFeedError::StaleIntel(
                format!("Intel is {} hours old (max: {} hours)", age.num_hours(), self.max_freshness_hours)
            ));
        }
        
        // Step 4: Check for replay (FAIL-CLOSED)
        if self.seen_bundles.contains(&bundle.bundle_id) {
            return Err(ThreatFeedError::ReplayDetected(
                format!("Bundle {} already processed", bundle.bundle_id)
            ));
        }
        
        // Step 5: Validate source attribution (FAIL-CLOSED)
        if bundle.source.is_empty() {
            return Err(ThreatFeedError::SourceAttributionMissing(
                "Source field is empty".to_string()
            ));
        }
        
        // Step 6: Record bundle as seen
        self.seen_bundles.insert(bundle.bundle_id.clone());
        
        // Step 7: Normalize and process
        let processed = ProcessedIntel {
            bundle_id: bundle.bundle_id.clone(),
            source: bundle.source.clone(),
            source_reputation: bundle.source_reputation,
            timestamp: bundle.timestamp,
            iocs: bundle.iocs,
            ttps: bundle.ttps,
            campaigns: bundle.campaigns,
            processed_at: Utc::now(),
        };
        
        info!("Ingested threat intel bundle: {} from source: {}", bundle.bundle_id, bundle.source);
        
        Ok(processed)
    }
    
    /// Check if feed is available (graceful degradation)
    pub fn check_feed_availability(&self, _feed_name: &str) -> bool {
        // In production, would check feed endpoint
        // For now, always return true (graceful degradation means we use cached data)
        true
    }
    
    /// Get seen bundle count (for monitoring)
    pub fn seen_bundle_count(&self) -> usize {
        self.seen_bundles.len()
    }
}

/// Processed threat intel (after validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedIntel {
    pub bundle_id: String,
    pub source: String,
    pub source_reputation: f64,
    pub timestamp: DateTime<Utc>,
    pub iocs: Vec<IOC>,
    pub ttps: Vec<TTP>,
    pub campaigns: Vec<Campaign>,
    pub processed_at: DateTime<Utc>,
}

