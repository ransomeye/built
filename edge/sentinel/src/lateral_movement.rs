// Path and File Name : /home/ransomeye/rebuild/edge/sentinel/src/lateral_movement.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Lateral movement detection and correlation across hosts - detects credential reuse, token replay, SMB/NTLM abuse, SSH brute-force

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use parking_lot::RwLock;
use tracing::warn;

/// Lateral movement event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LateralMovementEvent {
    pub event_id: String,
    pub event_type: LateralMovementType,
    pub source_host: String,
    pub target_host: String,
    pub credential_hash: Option<String>,
    pub token_id: Option<String>,
    pub protocol: String,
    pub timestamp: DateTime<Utc>,
    pub attacker_session_id: String,
    pub confidence_score: f64,
}

/// Types of lateral movement attempts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LateralMovementType {
    CredentialReuse,
    TokenReplay,
    SMBProbe,
    NTLMAbuse,
    SSHBruteForce,
    PrivilegeEscalation,
}

/// Attacker session tracking
#[derive(Debug, Clone)]
struct AttackerSession {
    session_id: String,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
    source_hosts: Vec<String>,
    target_hosts: Vec<String>,
    credential_hashes: Vec<String>,
    event_count: u64,
    patterns: Vec<String>,
}

/// Lateral movement detector - correlates events across hosts
pub struct LateralMovementDetector {
    sessions: Arc<RwLock<HashMap<String, AttackerSession>>>,
    credential_history: Arc<RwLock<HashMap<String, Vec<CredentialUse>>>>,
    event_window: Duration,
    correlation_threshold: f64,
}

#[derive(Debug, Clone)]
struct CredentialUse {
    host: String,
    timestamp: DateTime<Utc>,
    success: bool,
}

impl LateralMovementDetector {
    /// Create new lateral movement detector
    pub fn new(event_window_secs: u64, correlation_threshold: f64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            credential_history: Arc::new(RwLock::new(HashMap::new())),
            event_window: Duration::from_secs(event_window_secs),
            correlation_threshold,
        }
    }
    
    /// Detect credential reuse across hosts
    pub fn detect_credential_reuse(
        &self,
        credential_hash: &str,
        source_host: &str,
        target_host: &str,
        timestamp: DateTime<Utc>,
    ) -> Option<LateralMovementEvent> {
        let mut history = self.credential_history.write();
        
        // Check if credential was used on different host recently
        if let Some(uses) = history.get(credential_hash) {
            for cred_use in uses.iter() {
                if cred_use.host != source_host && cred_use.host != target_host {
                    // Credential used on different host - potential reuse
                    let time_diff = timestamp.signed_duration_since(cred_use.timestamp);
                    if time_diff.num_seconds() < self.event_window.as_secs() as i64 {
                        // Within time window - high confidence
                        let session_id = self.get_or_create_session(source_host, target_host, timestamp);
                        
                        let event = LateralMovementEvent {
                            event_id: format!("lm_{}", uuid::Uuid::new_v4().to_string()),
                            event_type: LateralMovementType::CredentialReuse,
                            source_host: source_host.to_string(),
                            target_host: target_host.to_string(),
                            credential_hash: Some(credential_hash.to_string()),
                            token_id: None,
                            protocol: "unknown".to_string(),
                            timestamp,
                            attacker_session_id: session_id.clone(),
                            confidence_score: 0.95,
                        };
                        
                        self.update_session(&session_id, source_host, target_host, timestamp, "credential_reuse");
                        
                        warn!("LATERAL MOVEMENT DETECTED: Credential reuse from {} to {} (credential used on {} previously)", 
                              source_host, target_host, cred_use.host);
                        
                        return Some(event);
                    }
                }
            }
        }
        
        // Record this credential use
        history.entry(credential_hash.to_string())
            .or_insert_with(Vec::new)
            .push(CredentialUse {
                host: target_host.to_string(),
                timestamp,
                success: true,
            });
        
        // Clean old entries
        self.cleanup_old_entries();
        
        None
    }
    
    /// Detect token replay
    pub fn detect_token_replay(
        &self,
        token_id: &str,
        source_host: &str,
        target_host: &str,
        timestamp: DateTime<Utc>,
    ) -> Option<LateralMovementEvent> {
        // Token replay detection - same token used on multiple hosts
        let session_id = self.get_or_create_session(source_host, target_host, timestamp);
        
        // Check if token was seen on different host
        // In production, would query distributed token store
        
        let event = LateralMovementEvent {
            event_id: format!("lm_{}", uuid::Uuid::new_v4().to_string()),
            event_type: LateralMovementType::TokenReplay,
            source_host: source_host.to_string(),
            target_host: target_host.to_string(),
            credential_hash: None,
            token_id: Some(token_id.to_string()),
            protocol: "token".to_string(),
            timestamp,
            attacker_session_id: session_id.clone(),
            confidence_score: 0.90,
        };
        
        self.update_session(&session_id, source_host, target_host, timestamp, "token_replay");
        
        warn!("LATERAL MOVEMENT DETECTED: Token replay from {} to {}", source_host, target_host);
        
        Some(event)
    }
    
    /// Detect SMB/NTLM abuse
    pub fn detect_smb_abuse(
        &self,
        source_host: &str,
        target_host: &str,
        credential_hash: Option<&str>,
        timestamp: DateTime<Utc>,
    ) -> Option<LateralMovementEvent> {
        let session_id = self.get_or_create_session(source_host, target_host, timestamp);
        
        let event = LateralMovementEvent {
            event_id: format!("lm_{}", uuid::Uuid::new_v4().to_string()),
            event_type: LateralMovementType::SMBProbe,
            source_host: source_host.to_string(),
            target_host: target_host.to_string(),
            credential_hash: credential_hash.map(|s| s.to_string()),
            token_id: None,
            protocol: "smb".to_string(),
            timestamp,
            attacker_session_id: session_id.clone(),
            confidence_score: 0.85,
        };
        
        self.update_session(&session_id, source_host, target_host, timestamp, "smb_abuse");
        
        warn!("LATERAL MOVEMENT DETECTED: SMB abuse from {} to {}", source_host, target_host);
        
        Some(event)
    }
    
    /// Detect SSH brute force
    pub fn detect_ssh_brute_force(
        &self,
        source_host: &str,
        target_host: &str,
        attempt_count: u32,
        timestamp: DateTime<Utc>,
    ) -> Option<LateralMovementEvent> {
        if attempt_count < 3 {
            return None; // Not enough attempts
        }
        
        let session_id = self.get_or_create_session(source_host, target_host, timestamp);
        
        let confidence = if attempt_count >= 10 {
            0.95
        } else if attempt_count >= 5 {
            0.85
        } else {
            0.70
        };
        
        let event = LateralMovementEvent {
            event_id: format!("lm_{}", uuid::Uuid::new_v4().to_string()),
            event_type: LateralMovementType::SSHBruteForce,
            source_host: source_host.to_string(),
            target_host: target_host.to_string(),
            credential_hash: None,
            token_id: None,
            protocol: "ssh".to_string(),
            timestamp,
            attacker_session_id: session_id.clone(),
            confidence_score: confidence,
        };
        
        self.update_session(&session_id, source_host, target_host, timestamp, "ssh_brute_force");
        
        warn!("LATERAL MOVEMENT DETECTED: SSH brute force from {} to {} ({} attempts)", 
              source_host, target_host, attempt_count);
        
        Some(event)
    }
    
    /// Detect privilege escalation attempt
    pub fn detect_privilege_escalation(
        &self,
        source_host: &str,
        target_host: &str,
        original_user: &str,
        target_user: &str,
        timestamp: DateTime<Utc>,
    ) -> Option<LateralMovementEvent> {
        if original_user == target_user {
            return None; // No escalation
        }
        
        let session_id = self.get_or_create_session(source_host, target_host, timestamp);
        
        let event = LateralMovementEvent {
            event_id: format!("lm_{}", uuid::Uuid::new_v4().to_string()),
            event_type: LateralMovementType::PrivilegeEscalation,
            source_host: source_host.to_string(),
            target_host: target_host.to_string(),
            credential_hash: None,
            token_id: None,
            protocol: "privilege_escalation".to_string(),
            timestamp,
            attacker_session_id: session_id.clone(),
            confidence_score: 0.90,
        };
        
        self.update_session(&session_id, source_host, target_host, timestamp, "privilege_escalation");
        
        warn!("LATERAL MOVEMENT DETECTED: Privilege escalation from {} to {} ({} -> {})", 
              source_host, target_host, original_user, target_user);
        
        Some(event)
    }
    
    /// Get or create attacker session
    fn get_or_create_session(&self, source_host: &str, target_host: &str, timestamp: DateTime<Utc>) -> String {
        let mut sessions = self.sessions.write();
        
        // Try to find existing session
        for (session_id, session) in sessions.iter() {
            if session.source_hosts.contains(&source_host.to_string()) ||
               session.target_hosts.contains(&target_host.to_string()) {
                // Found existing session
                return session_id.clone();
            }
        }
        
        // Create new session
        let session_id = format!("session_{}", uuid::Uuid::new_v4().to_string());
        let session = AttackerSession {
            session_id: session_id.clone(),
            first_seen: timestamp,
            last_seen: timestamp,
            source_hosts: vec![source_host.to_string()],
            target_hosts: vec![target_host.to_string()],
            credential_hashes: Vec::new(),
            event_count: 1,
            patterns: Vec::new(),
        };
        
        sessions.insert(session_id.clone(), session);
        session_id
    }
    
    /// Update attacker session
    fn update_session(&self, session_id: &str, source_host: &str, target_host: &str, timestamp: DateTime<Utc>, pattern: &str) {
        let mut sessions = self.sessions.write();
        
        if let Some(session) = sessions.get_mut(session_id) {
            if !session.source_hosts.contains(&source_host.to_string()) {
                session.source_hosts.push(source_host.to_string());
            }
            if !session.target_hosts.contains(&target_host.to_string()) {
                session.target_hosts.push(target_host.to_string());
            }
            session.last_seen = timestamp;
            session.event_count += 1;
            if !session.patterns.contains(&pattern.to_string()) {
                session.patterns.push(pattern.to_string());
            }
        }
    }
    
    /// Cleanup old entries
    fn cleanup_old_entries(&self) {
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(self.event_window.as_secs() as i64 * 2);
        
        // Cleanup sessions
        let mut sessions = self.sessions.write();
        sessions.retain(|_, session| session.last_seen > cutoff);
        
        // Cleanup credential history
        let mut history = self.credential_history.write();
        for uses in history.values_mut() {
            uses.retain(|use_| use_.timestamp > cutoff);
        }
        history.retain(|_, uses| !uses.is_empty());
    }
    
    /// Get all active sessions
    pub fn get_active_sessions(&self) -> Vec<AttackerSession> {
        self.sessions.read().values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_credential_reuse_detection() {
        let detector = LateralMovementDetector::new(3600, 0.8);
        let now = Utc::now();
        
        // First use on host1
        detector.detect_credential_reuse("hash123", "host1", "host1", now);
        
        // Reuse on host2 - should trigger
        let event = detector.detect_credential_reuse("hash123", "host1", "host2", now);
        assert!(event.is_some());
        assert_eq!(event.unwrap().event_type, LateralMovementType::CredentialReuse);
    }
}

