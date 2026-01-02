// Path and File Name : /home/ransomeye/rebuild/edge/agent/linux/src/deception.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Deception artifacts engine - honeyfiles, honeycredentials, fake services with fail-closed alerting

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, debug};
use parking_lot::RwLock;
use sha2::{Sha256, Digest};
use hex;
use crossbeam_channel::Sender;
use crate::event::{AgentEvent, FileEvent, AuthEvent, NetworkEvent};

/// Deception event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeceptionEventType {
    HoneyfileRead,
    HoneyfileWrite,
    HoneyfileDelete,
    HoneycredentialUse,
    FakeServiceConnection,
    LateralMovementAttempt,
}

/// Deception event with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeceptionEvent {
    pub event_type: DeceptionEventType,
    pub artifact_id: String,
    pub artifact_path: String,
    pub process_id: i32,
    pub process_name: String,
    pub user_id: u32,
    pub host_id: String,
    pub timestamp: DateTime<Utc>,
    pub severity: String, // "HIGH" or "CRITICAL"
    pub confidence_score: f64,
    pub attacker_session_id: Option<String>,
}

/// Signed allowlist for legitimate processes
#[derive(Debug, Clone)]
pub struct AllowlistEntry {
    pub process_path: String,
    pub process_hash: String, // SHA-256 hash of binary
    pub signature: String, // Cryptographic signature
}

/// Honeyfile artifact
#[derive(Debug, Clone)]
pub struct Honeyfile {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub hash: String,
}

/// Honeycredential artifact
#[derive(Debug, Clone)]
pub struct Honeycredential {
    pub id: String,
    pub location: String, // config file path, env var name, or memory location
    pub credential_type: String, // "password", "api_key", "token"
    pub fake_value: String,
    pub created_at: DateTime<Utc>,
}

/// Fake service listener
#[derive(Debug, Clone)]
pub struct FakeService {
    pub id: String,
    pub port: u16,
    pub protocol: String, // "ssh", "smb", "http", etc.
    pub created_at: DateTime<Utc>,
}

/// Deception engine - manages all deception artifacts
pub struct DeceptionEngine {
    honeyfiles: Arc<RwLock<Vec<Honeyfile>>>,
    honeycredentials: Arc<RwLock<Vec<Honeycredential>>>,
    fake_services: Arc<RwLock<Vec<FakeService>>>,
    allowlist: Arc<RwLock<HashSet<String>>>, // Process paths that bypass deception
    event_tx: Sender<AgentEvent>,
    host_id: String,
    artifact_dir: PathBuf,
}

impl DeceptionEngine {
    /// Create new deception engine
    pub fn new(
        event_tx: Sender<AgentEvent>,
        host_id: String,
        artifact_dir: impl AsRef<Path>,
    ) -> Result<Self, DeceptionError> {
        let artifact_dir = artifact_dir.as_ref();
        
        // Create artifact directory if it doesn't exist
        fs::create_dir_all(artifact_dir)
            .map_err(|e| DeceptionError::InitializationFailed(
                format!("Failed to create artifact directory: {}", e)
            ))?;
        
        // Load allowlist from environment or config
        let allowlist = Self::load_allowlist()?;
        
        Ok(Self {
            honeyfiles: Arc::new(RwLock::new(Vec::new())),
            honeycredentials: Arc::new(RwLock::new(Vec::new())),
            fake_services: Arc::new(RwLock::new(Vec::new())),
            allowlist: Arc::new(RwLock::new(allowlist)),
            event_tx,
            host_id,
            artifact_dir: artifact_dir.to_path_buf(),
        })
    }
    
    /// Load allowlist from environment or config file
    fn load_allowlist() -> Result<HashSet<String>, DeceptionError> {
        let mut allowlist = HashSet::new();
        
        // Load from environment variable (comma-separated)
        if let Ok(allowlist_env) = std::env::var("RANSOMEYE_DECEPTION_ALLOWLIST") {
            for path in allowlist_env.split(',') {
                let path = path.trim();
                if !path.is_empty() {
                    allowlist.insert(path.to_string());
                }
            }
        }
        
        // Load from config file if specified
        if let Ok(config_path) = std::env::var("RANSOMEYE_DECEPTION_ALLOWLIST_FILE") {
            if let Ok(contents) = fs::read_to_string(&config_path) {
                for line in contents.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        allowlist.insert(line.to_string());
                    }
                }
            }
        }
        
        info!("Loaded {} allowlist entries", allowlist.len());
        Ok(allowlist)
    }
    
    /// Check if process is allowlisted
    fn is_allowlisted(&self, process_path: &str) -> bool {
        let allowlist = self.allowlist.read();
        allowlist.contains(process_path)
    }
    
    /// Generate honeyfiles
    pub fn generate_honeyfiles(&self, count: usize) -> Result<Vec<Honeyfile>, DeceptionError> {
        let mut honeyfiles = Vec::new();
        let fake_names = vec![
            "backup_credentials.txt",
            "production_db_password.txt",
            "api_keys_backup.json",
            "ssh_private_key_backup",
            "admin_password.txt",
            "database_config.conf",
            "secrets.env",
            "master_key.pem",
        ];
        
        for i in 0..count {
            let name = fake_names[i % fake_names.len()];
            let honeyfile = self.create_honeyfile(name)?;
            honeyfiles.push(honeyfile.clone());
            
            let mut files = self.honeyfiles.write();
            files.push(honeyfile);
        }
        
        info!("Generated {} honeyfiles", honeyfiles.len());
        Ok(honeyfiles)
    }
    
    /// Create a single honeyfile
    fn create_honeyfile(&self, name: &str) -> Result<Honeyfile, DeceptionError> {
        let id = format!("honeyfile_{}", uuid::Uuid::new_v4().to_string());
        let path = self.artifact_dir.join(&id).join(name);
        
        // Create parent directory
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| DeceptionError::ArtifactCreationFailed(
                    format!("Failed to create honeyfile directory: {}", e)
                ))?;
        }
        
        // Write fake content (never real data)
        let fake_content = format!(
            "# This is a fake file created by RansomEye deception engine\n\
            # DO NOT USE THESE CREDENTIALS\n\
            # File ID: {}\n\
            # Created: {}\n\
            # This file is monitored - any access will trigger an alert\n",
            id,
            Utc::now().to_rfc3339()
        );
        
        fs::write(&path, fake_content)
            .map_err(|e| DeceptionError::ArtifactCreationFailed(
                format!("Failed to write honeyfile: {}", e)
            ))?;
        
        // Calculate hash
        let mut hasher = Sha256::new();
        hasher.update(&fake_content);
        let hash = hex::encode(hasher.finalize());
        
        let honeyfile = Honeyfile {
            id: id.clone(),
            path: path.clone(),
            name: name.to_string(),
            created_at: Utc::now(),
            hash,
        };
        
        debug!("Created honeyfile: {} at {}", id, path.display());
        Ok(honeyfile)
    }
    
    /// Generate honeycredentials
    pub fn generate_honeycredentials(&self, count: usize) -> Result<Vec<Honeycredential>, DeceptionError> {
        let mut honeycredentials = Vec::new();
        
        let locations = vec![
            ("/etc/config/secrets.env", "password"),
            ("/home/user/.ssh/config", "api_key"),
            ("/var/lib/app/credentials.json", "token"),
            ("/tmp/.env_backup", "password"),
        ];
        
        for i in 0..count {
            let (location, cred_type) = &locations[i % locations.len()];
            let honeycred = self.create_honeycredential(location, cred_type)?;
            honeycredentials.push(honeycred.clone());
            
            let mut creds = self.honeycredentials.write();
            creds.push(honeycred);
        }
        
        info!("Generated {} honeycredentials", honeycredentials.len());
        Ok(honeycredentials)
    }
    
    /// Create a single honeycredential
    fn create_honeycredential(&self, location: &str, cred_type: &str) -> Result<Honeycredential, DeceptionError> {
        let id = format!("honeycred_{}", uuid::Uuid::new_v4().to_string());
        
        // Generate fake credential value (never real)
        let fake_value = format!("FAKE_{}_{}", cred_type.to_uppercase(), hex::encode(&id.as_bytes()[..8]));
        
        let honeycred = Honeycredential {
            id: id.clone(),
            location: location.to_string(),
            credential_type: cred_type.to_string(),
            fake_value,
            created_at: Utc::now(),
        };
        
        debug!("Created honeycredential: {} at {}", id, location);
        Ok(honeycred)
    }
    
    /// Generate fake services
    pub fn generate_fake_services(&self, count: usize) -> Result<Vec<FakeService>, DeceptionError> {
        let mut fake_services = Vec::new();
        
        let services = vec![
            (2222, "ssh"),
            (445, "smb"),
            (3389, "rdp"),
            (5432, "postgres"),
            (3306, "mysql"),
        ];
        
        for i in 0..count {
            let (port, protocol) = &services[i % services.len()];
            let fake_service = self.create_fake_service(*port, protocol)?;
            fake_services.push(fake_service.clone());
            
            let mut services = self.fake_services.write();
            services.push(fake_service);
        }
        
        info!("Generated {} fake services", fake_services.len());
        Ok(fake_services)
    }
    
    /// Create a single fake service
    fn create_fake_service(&self, port: u16, protocol: &str) -> Result<FakeService, DeceptionError> {
        let id = format!("fakeservice_{}_{}", protocol, port);
        
        let fake_service = FakeService {
            id: id.clone(),
            port,
            protocol: protocol.to_string(),
            created_at: Utc::now(),
        };
        
        debug!("Created fake service: {} on port {}", protocol, port);
        Ok(fake_service)
    }
    
    /// Monitor file access and detect honeyfile interaction
    pub fn check_file_access(&self, path: &Path, operation: &str, process_id: i32, process_name: &str, user_id: u32) -> Result<(), DeceptionError> {
        // Check if process is allowlisted
        if self.is_allowlisted(process_name) {
            debug!("Process {} is allowlisted, skipping deception check", process_name);
            return Ok(());
        }
        
        let path_str = path.to_string_lossy().to_string();
        
        // Check if this is a honeyfile
        let honeyfiles = self.honeyfiles.read();
        for honeyfile in honeyfiles.iter() {
            if path_str.contains(&honeyfile.id) || path_str.ends_with(&honeyfile.name) {
                // Honeyfile accessed!
                let severity = match operation {
                    "read" => "HIGH",
                    "write" | "delete" => "CRITICAL",
                    _ => "HIGH",
                };
                
                let event = DeceptionEvent {
                    event_type: match operation {
                        "read" => DeceptionEventType::HoneyfileRead,
                        "write" => DeceptionEventType::HoneyfileWrite,
                        "delete" => DeceptionEventType::HoneyfileDelete,
                        _ => DeceptionEventType::HoneyfileRead,
                    },
                    artifact_id: honeyfile.id.clone(),
                    artifact_path: path_str.clone(),
                    process_id,
                    process_name: process_name.to_string(),
                    user_id,
                    host_id: self.host_id.clone(),
                    timestamp: Utc::now(),
                    severity: severity.to_string(),
                    confidence_score: 1.0, // 100% confidence for honeyfile access
                    attacker_session_id: None,
                };
                
                self.emit_deception_event(event)?;
                
                warn!("DECEPTION ALERT: Honeyfile {} accessed by {} (operation: {})", 
                      honeyfile.name, process_name, operation);
                
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    /// Monitor credential usage and detect honeycredential interaction
    pub fn check_credential_use(&self, location: &str, process_id: i32, process_name: &str, user_id: u32) -> Result<(), DeceptionError> {
        // Check if process is allowlisted
        if self.is_allowlisted(process_name) {
            return Ok(());
        }
        
        let honeycredentials = self.honeycredentials.read();
        for honeycred in honeycredentials.iter() {
            if location.contains(&honeycred.location) || location.contains(&honeycred.fake_value) {
                // Honeycredential used!
                let event = DeceptionEvent {
                    event_type: DeceptionEventType::HoneycredentialUse,
                    artifact_id: honeycred.id.clone(),
                    artifact_path: location.to_string(),
                    process_id,
                    process_name: process_name.to_string(),
                    user_id,
                    host_id: self.host_id.clone(),
                    timestamp: Utc::now(),
                    severity: "CRITICAL".to_string(),
                    confidence_score: 1.0,
                    attacker_session_id: None,
                };
                
                self.emit_deception_event(event)?;
                
                error!("DECEPTION ALERT: Honeycredential {} used by {} at {}", 
                       honeycred.id, process_name, location);
                
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    /// Monitor network connections and detect fake service access
    pub fn check_service_connection(&self, port: u16, protocol: &str, src_ip: &str, process_id: i32, process_name: &str) -> Result<(), DeceptionError> {
        // Check if process is allowlisted
        if self.is_allowlisted(process_name) {
            return Ok(());
        }
        
        let fake_services = self.fake_services.read();
        for fake_service in fake_services.iter() {
            if fake_service.port == port && fake_service.protocol == protocol {
                // Fake service accessed!
                let event = DeceptionEvent {
                    event_type: DeceptionEventType::FakeServiceConnection,
                    artifact_id: fake_service.id.clone(),
                    artifact_path: format!("{}:{}", protocol, port),
                    process_id,
                    process_name: process_name.to_string(),
                    user_id: 0, // Network events may not have user context
                    host_id: self.host_id.clone(),
                    timestamp: Utc::now(),
                    severity: "HIGH".to_string(),
                    confidence_score: 0.95,
                    attacker_session_id: None,
                };
                
                self.emit_deception_event(event)?;
                
                warn!("DECEPTION ALERT: Fake service {}:{} accessed by {} from {}", 
                      protocol, port, process_name, src_ip);
                
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    /// Emit deception event to the event pipeline
    fn emit_deception_event(&self, event: DeceptionEvent) -> Result<(), DeceptionError> {
        // Convert to AgentEvent
        let agent_event = match event.event_type {
            DeceptionEventType::HoneyfileRead | 
            DeceptionEventType::HoneyfileWrite | 
            DeceptionEventType::HoneyfileDelete => {
                AgentEvent::File(FileEvent {
                    event_type: "deception".to_string(),
                    path: event.artifact_path.clone(),
                    operation: format!("{:?}", event.event_type),
                    user_id: event.user_id,
                    process_id: event.process_id,
                    timestamp: event.timestamp,
                })
            },
            DeceptionEventType::HoneycredentialUse => {
                AgentEvent::Auth(AuthEvent {
                    event_type: "deception".to_string(),
                    user: format!("honeycred_{}", event.artifact_id),
                    source: event.artifact_path.clone(),
                    success: false, // Always false for honeycredentials
                    timestamp: event.timestamp,
                })
            },
            DeceptionEventType::FakeServiceConnection => {
                AgentEvent::Network(NetworkEvent {
                    event_type: "deception".to_string(),
                    src_ip: "0.0.0.0".to_string(),
                    dst_ip: "127.0.0.1".to_string(),
                    src_port: 0,
                    dst_port: 0,
                    protocol: event.artifact_path.clone(),
                    process_id: event.process_id,
                    timestamp: event.timestamp,
                })
            },
            DeceptionEventType::LateralMovementAttempt => {
                AgentEvent::Network(NetworkEvent {
                    event_type: "deception".to_string(),
                    src_ip: "0.0.0.0".to_string(),
                    dst_ip: "127.0.0.1".to_string(),
                    src_port: 0,
                    dst_port: 0,
                    protocol: "lateral_movement".to_string(),
                    process_id: event.process_id,
                    timestamp: event.timestamp,
                })
            },
        };
        
        // Add metadata to event (severity, confidence, etc.)
        // This will be enriched by the ingest pipeline
        
        // Send to event channel
        self.event_tx.send(agent_event)
            .map_err(|e| DeceptionError::EventEmissionFailed(
                format!("Failed to send deception event: {}", e)
            ))?;
        
        Ok(())
    }
    
    /// Get all honeyfiles
    pub fn get_honeyfiles(&self) -> Vec<Honeyfile> {
        self.honeyfiles.read().clone()
    }
    
    /// Get all honeycredentials
    pub fn get_honeycredentials(&self) -> Vec<Honeycredential> {
        self.honeycredentials.read().clone()
    }
    
    /// Get all fake services
    pub fn get_fake_services(&self) -> Vec<FakeService> {
        self.fake_services.read().clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeceptionError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Artifact creation failed: {0}")]
    ArtifactCreationFailed(String),
    #[error("Event emission failed: {0}")]
    EventEmissionFailed(String),
}

