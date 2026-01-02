// Path and File Name : /home/ransomeye/rebuild/core/threat_feed/src/normalization.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Intel normalization - standardizes IOCs, TTPs, campaigns to unified ontology

use crate::ingestion::{IOC, IOCType, TTP, Campaign, ProcessedIntel};
use crate::errors::ThreatFeedError;
use tracing::debug;

/// Intel normalizer - normalizes threat intel to standard format
pub struct IntelNormalizer;

impl IntelNormalizer {
    /// Normalize processed intel
    pub fn normalize(intel: ProcessedIntel) -> Result<NormalizedIntel, ThreatFeedError> {
        let normalized_iocs = intel.iocs.into_iter()
            .map(|ioc| Self::normalize_ioc(ioc))
            .collect::<Result<Vec<_>, _>>()?;
        
        let normalized_ttps = intel.ttps.into_iter()
            .map(|ttp| Self::normalize_ttp(ttp))
            .collect::<Result<Vec<_>, _>>()?;
        
        let normalized_campaigns = intel.campaigns.into_iter()
            .map(|campaign| Self::normalize_campaign(campaign))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(NormalizedIntel {
            bundle_id: intel.bundle_id,
            source: intel.source,
            source_reputation: intel.source_reputation,
            timestamp: intel.timestamp,
            iocs: normalized_iocs,
            ttps: normalized_ttps,
            campaigns: normalized_campaigns,
            processed_at: intel.processed_at,
        })
    }
    
    /// Normalize IOC
    fn normalize_ioc(ioc: IOC) -> Result<NormalizedIOC, ThreatFeedError> {
        // Normalize value (lowercase, trim)
        let normalized_value = match &ioc.ioc_type {
            IOCType::IP | IOCType::Domain | IOCType::URL | IOCType::Email => {
                ioc.value.to_lowercase().trim().to_string()
            },
            IOCType::HashMD5 | IOCType::HashSHA1 | IOCType::HashSHA256 => {
                ioc.value.to_lowercase().trim().to_string()
            },
            IOCType::FilePath => {
                ioc.value.trim().to_string()
            },
        };
        
        Ok(NormalizedIOC {
            ioc_id: ioc.ioc_id,
            ioc_type: ioc.ioc_type,
            value: normalized_value,
            first_seen: ioc.first_seen,
            last_seen: ioc.last_seen,
            confidence: ioc.confidence,
            tags: ioc.tags,
            metadata: ioc.metadata,
        })
    }
    
    /// Normalize TTP
    fn normalize_ttp(ttp: TTP) -> Result<NormalizedTTP, ThreatFeedError> {
        // Normalize MITRE ID (uppercase)
        let normalized_mitre_id = ttp.mitre_id.to_uppercase().trim().to_string();
        
        Ok(NormalizedTTP {
            ttp_id: ttp.ttp_id,
            mitre_id: normalized_mitre_id,
            name: ttp.name.trim().to_string(),
            description: ttp.description.trim().to_string(),
            confidence: ttp.confidence,
            observed_at: ttp.observed_at,
            metadata: ttp.metadata,
        })
    }
    
    /// Normalize campaign
    fn normalize_campaign(campaign: Campaign) -> Result<NormalizedCampaign, ThreatFeedError> {
        Ok(NormalizedCampaign {
            campaign_id: campaign.campaign_id,
            name: campaign.name.trim().to_string(),
            description: campaign.description.trim().to_string(),
            start_date: campaign.start_date,
            end_date: campaign.end_date,
            associated_iocs: campaign.associated_iocs,
            associated_ttps: campaign.associated_ttps,
            confidence: campaign.confidence,
            metadata: campaign.metadata,
        })
    }
}

/// Normalized threat intel
#[derive(Debug, Clone)]
pub struct NormalizedIntel {
    pub bundle_id: String,
    pub source: String,
    pub source_reputation: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub iocs: Vec<NormalizedIOC>,
    pub ttps: Vec<NormalizedTTP>,
    pub campaigns: Vec<NormalizedCampaign>,
    pub processed_at: chrono::DateTime<chrono::Utc>,
}

/// Normalized IOC
#[derive(Debug, Clone)]
pub struct NormalizedIOC {
    pub ioc_id: String,
    pub ioc_type: IOCType,
    pub value: String,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub confidence: f64,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
}

/// Normalized TTP
#[derive(Debug, Clone)]
pub struct NormalizedTTP {
    pub ttp_id: String,
    pub mitre_id: String,
    pub name: String,
    pub description: String,
    pub confidence: f64,
    pub observed_at: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

/// Normalized campaign
#[derive(Debug, Clone)]
pub struct NormalizedCampaign {
    pub campaign_id: String,
    pub name: String,
    pub description: String,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub associated_iocs: Vec<String>,
    pub associated_ttps: Vec<String>,
    pub confidence: f64,
    pub metadata: serde_json::Value,
}

