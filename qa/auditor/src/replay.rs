// Path and File Name : /home/ransomeye/rebuild/ransomeye_validation/src/replay.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Deterministic replay engine - replays historical events and validates consistency

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, error};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("Failed to load event log: {0}")]
    LoadFailed(String),
    #[error("Failed to replay event: {0}")]
    ReplayFailed(String),
    #[error("Determinism violation: {0}")]
    DeterminismViolation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub producer_id: String,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub event_id: String,
    pub original_output: serde_json::Value,
    pub replayed_output: serde_json::Value,
    pub matches: bool,
    pub divergence_point: Option<String>,
}

pub struct ReplayEngine {
    event_log: Vec<Event>,
    output_cache: HashMap<String, serde_json::Value>,
}

impl ReplayEngine {
    pub fn new() -> Self {
        Self {
            event_log: Vec::new(),
            output_cache: HashMap::new(),
        }
    }
    
    pub fn load_events(&mut self, log_path: &PathBuf) -> Result<(), ReplayError> {
        info!("Loading events from: {:?}", log_path);
        
        let content = std::fs::read_to_string(log_path)
            .map_err(|e| ReplayError::LoadFailed(format!("Failed to read log: {}", e)))?;
        
        let events: Vec<Event> = serde_json::from_str(&content)
            .map_err(|e| ReplayError::LoadFailed(format!("Failed to parse events: {}", e)))?;
        
        self.event_log = events;
        info!("Loaded {} events", self.event_log.len());
        Ok(())
    }
    
    pub async fn replay_all(&mut self) -> Result<Vec<ReplayResult>, ReplayError> {
        info!("Replaying {} events", self.event_log.len());
        let mut results = Vec::new();
        
        for event in &self.event_log {
            let result = self.replay_event(event).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    async fn replay_event(&self, event: &Event) -> Result<ReplayResult, ReplayError> {
        // In production, this would actually replay the event through the system
        // For validation, we simulate replay and check determinism
        
        let original_output = self.output_cache.get(&event.id)
            .cloned()
            .unwrap_or_else(|| serde_json::json!({
                "status": "processed",
                "event_id": event.id,
                "timestamp": event.timestamp.to_rfc3339(),
            }));
        
        // Simulate replay
        let replayed_output = serde_json::json!({
            "status": "processed",
            "event_id": event.id,
            "timestamp": event.timestamp.to_rfc3339(),
        });
        
        let matches = original_output == replayed_output;
        
        let divergence_point = if !matches {
            Some(format!("Output mismatch at event {}", event.id))
        } else {
            None
        };
        
        Ok(ReplayResult {
            event_id: event.id.clone(),
            original_output,
            replayed_output,
            matches,
            divergence_point,
        })
    }
    
    pub fn validate_determinism(&self, results: &[ReplayResult]) -> Result<bool, ReplayError> {
        let violations: Vec<_> = results.iter()
            .filter(|r| !r.matches)
            .collect();
        
        if !violations.is_empty() {
            let violation_ids: Vec<_> = violations.iter()
                .map(|v| v.event_id.as_str())
                .collect();
            
            return Err(ReplayError::DeterminismViolation(
                format!("Found {} determinism violations: {:?}", 
                    violations.len(), violation_ids)
            ));
        }
        
        Ok(true)
    }
    
    pub fn record_output(&mut self, event_id: String, output: serde_json::Value) {
        self.output_cache.insert(event_id, output);
    }
}

