// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/timeline.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Forensic timeline builder - creates deterministic, chronologically ordered timelines with source attribution and kill-chain annotations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use tracing::debug;

use crate::errors::ReportingError;
#[cfg(feature = "future-reporting")]
use crate::collector::CollectedEvidence;

/// Timeline event - represents a single event in the forensic timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub source_type: String,
    pub event_type: String,
    pub kill_chain_stage: Option<String>,
    pub description: String,
    pub evidence_id: String,
    pub metadata: serde_json::Value,
}

/// Forensic timeline - deterministic chronological ordering of events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "future-reporting")]
pub struct ForensicTimeline {
    events: Vec<TimelineEvent>,
}

#[cfg(feature = "future-reporting")]
impl ForensicTimeline {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }
    
    /// Add evidence to timeline
    /// Events are automatically sorted by timestamp
    pub fn add_evidence(&mut self, evidence: &CollectedEvidence) -> Result<(), ReportingError> {
        let event = TimelineEvent {
            timestamp: evidence.timestamp,
            source: evidence.source.clone(),
            source_type: evidence.source_type.clone(),
            event_type: "evidence".to_string(),
            kill_chain_stage: evidence.kill_chain_stage.clone(),
            description: format!("Evidence collected from {}", evidence.source),
            evidence_id: evidence.evidence_id.clone(),
            metadata: serde_json::to_value(&evidence.metadata)
                .map_err(|e| ReportingError::SerializationError(e))?,
        };
        
        self.events.push(event);
        
        // Sort by timestamp (deterministic ordering)
        self.events.sort_by(|a, b| {
            match a.timestamp.cmp(&b.timestamp) {
                Ordering::Equal => a.evidence_id.cmp(&b.evidence_id),
                other => other,
            }
        });
        
        debug!("Added evidence to timeline (total events: {})", self.events.len());
        Ok(())
    }
    
    /// Add custom timeline event
    pub fn add_event(
        &mut self,
        timestamp: DateTime<Utc>,
        source: &str,
        source_type: &str,
        event_type: &str,
        description: &str,
        kill_chain_stage: Option<String>,
        metadata: serde_json::Value,
    ) -> Result<(), ReportingError> {
        // Validate timestamp is not in the future
        let now = Utc::now();
        if timestamp > now {
            return Err(ReportingError::InvalidTimestamp(
                format!("Timestamp {} is in the future", timestamp)
            ));
        }
        
        let event = TimelineEvent {
            timestamp,
            source: source.to_string(),
            source_type: source_type.to_string(),
            event_type: event_type.to_string(),
            kill_chain_stage,
            description: description.to_string(),
            evidence_id: uuid::Uuid::new_v4().to_string(),
            metadata,
        };
        
        self.events.push(event);
        
        // Sort by timestamp
        self.events.sort_by(|a, b| {
            match a.timestamp.cmp(&b.timestamp) {
                Ordering::Equal => a.evidence_id.cmp(&b.evidence_id),
                other => other,
            }
        });
        
        Ok(())
    }
    
    /// Get all events (chronologically ordered)
    pub fn get_events(&self) -> &[TimelineEvent] {
        &self.events
    }
    
    /// Get events in time range
    pub fn get_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&TimelineEvent> {
        self.events.iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }
    
    /// Get events by kill-chain stage
    pub fn get_events_by_stage(&self, stage: &str) -> Vec<&TimelineEvent> {
        self.events.iter()
            .filter(|e| e.kill_chain_stage.as_ref().map(|s| s == stage).unwrap_or(false))
            .collect()
    }
    
    /// Get events by source
    pub fn get_events_by_source(&self, source: &str) -> Vec<&TimelineEvent> {
        self.events.iter()
            .filter(|e| e.source == source)
            .collect()
    }
    
    /// Build timeline from evidence bundles
    pub fn from_evidence_bundles(
        bundles: &[crate::evidence_store::EvidenceBundle],
    ) -> Result<Self, ReportingError> {
        let mut timeline = Self::new();
        
        for bundle in bundles {
            for evidence in &bundle.evidence_items {
                timeline.add_evidence(evidence)?;
            }
        }
        
        Ok(timeline)
    }
    
    /// Export timeline as JSON
    pub fn to_json(&self) -> Result<String, ReportingError> {
        serde_json::to_string_pretty(&self.events)
            .map_err(|e| ReportingError::SerializationError(e))
    }
    
    /// Get timeline summary statistics
    pub fn get_summary(&self) -> TimelineSummary {
        let mut sources = std::collections::HashSet::new();
        let mut stages = std::collections::HashSet::new();
        
        for event in &self.events {
            sources.insert(event.source.clone());
            if let Some(stage) = &event.kill_chain_stage {
                stages.insert(stage.clone());
            }
        }
        
        TimelineSummary {
            total_events: self.events.len(),
            time_span_start: self.events.first().map(|e| e.timestamp),
            time_span_end: self.events.last().map(|e| e.timestamp),
            unique_sources: sources.len(),
            kill_chain_stages: stages.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "future-reporting")]
pub struct TimelineSummary {
    pub total_events: usize,
    pub time_span_start: Option<DateTime<Utc>>,
    pub time_span_end: Option<DateTime<Utc>>,
    pub unique_sources: usize,
    pub kill_chain_stages: Vec<String>,
}

impl Default for ForensicTimeline {
    fn default() -> Self {
        Self::new()
    }
}

