// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: DPI Probe main entry point - standalone network telemetry sensor

use std::sync::Arc;
use tracing::{info, error};
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::Client as ReqwestClient;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use tokio::runtime::Runtime;

pub mod errors;
pub mod capture;
pub mod parser;
pub mod flow;
pub mod extraction;
pub mod envelope;
pub mod backpressure;
pub mod rate_limit;
pub mod health;
pub mod hardening;

#[path = "../security/mod.rs"]
pub mod security;

use errors::ProbeError;
use capture::PacketCapture;
use parser::ProtocolParser;
use flow::FlowTracker;
use extraction::FeatureExtractor;
use envelope::EnvelopeBuilder;
use backpressure::BackpressureManager;
use rate_limit::RateLimiter;
use health::HealthMonitor;
use hardening::RuntimeHardening;
use security::{IdentityManager, EventSigner};
#[path = "../../config/validation.rs"]
mod config_validation;

use config_validation::ProbeConfig;

fn main() -> Result<(), ProbeError> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("RansomEye DPI Probe starting...");
    
    // Get binary path for integrity verification
    let binary_path = std::env::current_exe()
        .map_err(|e| ProbeError::ConfigurationError(format!("Failed to get binary path: {}", e)))?
        .to_string_lossy()
        .to_string();
    
    // Initialize runtime hardening (FAIL-CLOSED on integrity failure)
    let config_path = std::env::var("DPI_CONFIG_PATH").ok();
    let hardening = hardening::RuntimeHardening::new(
        binary_path.clone(),
        config_path.clone(),
        30, // 30 second watchdog interval
    ).map_err(|e| ProbeError::ConfigurationError(format!("Hardening initialization failed: {}", e)))?;
    
    // Verify binary integrity at startup (FAIL-CLOSED)
    hardening.verify_binary_integrity()
        .map_err(|e| ProbeError::ConfigurationError(format!("Binary integrity check failed: {}", e)))?;
    
    // Verify config integrity at startup (FAIL-CLOSED)
    hardening.verify_config_integrity()
        .map_err(|e| ProbeError::ConfigurationError(format!("Config integrity check failed: {}", e)))?;
    
    // Perform runtime tamper checks (FAIL-CLOSED)
    hardening.perform_runtime_checks()
        .map_err(|e| ProbeError::ConfigurationError(format!("Runtime check failed: {}", e)))?;
    
    // Start watchdog timer
    hardening.start_watchdog()
        .map_err(|e| ProbeError::ConfigurationError(format!("Watchdog start failed: {}", e)))?;
    
    // Load configuration (ENV-only, fail-closed)
    let config = ProbeConfig::from_env()
        .map_err(|e| ProbeError::ConfigurationError(e))?;
    
    config.validate()
        .map_err(|e| ProbeError::ConfigurationError(e))?;
    
    info!("Configuration loaded: interface={}, max_flows={}", 
        config.capture_interface, config.max_flows);
    
    // Initialize identity (fail-closed on failure)
    let identity_path = config.identity_path.as_ref().map(|p| std::path::Path::new(p));
    let identity = IdentityManager::load_or_create(identity_path)
        .map_err(|e| ProbeError::IdentityVerificationFailed(format!("{}", e)))?;
    
    info!("Component identity: {}", identity.component_id());
    
    // Initialize event signer (fail-closed on failure)
    let signer = if let Some(ref key_path) = config.signing_key_path {
        EventSigner::from_key_file(std::path::Path::new(key_path))
            .map_err(|e| ProbeError::SigningFailed(format!("{}", e)))?
    } else {
        EventSigner::new()
            .map_err(|e| ProbeError::SigningFailed(format!("{}", e)))?
    };
    
    info!("Event signer initialized");
    
    // Get CORE_API_URL from environment
    let core_api_url = std::env::var("CORE_API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    
    // Initialize reqwest HTTP client for direct telemetry delivery
    let http_client = ReqwestClient::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| ProbeError::ConfigurationError(format!("Failed to create HTTP client: {}", e)))?;
    
    info!("HTTP client initialized for direct delivery to {}", core_api_url);
    
    // Create tokio runtime for async HTTP calls
    let rt = Runtime::new()
        .map_err(|e| ProbeError::ConfigurationError(format!("Failed to create runtime: {}", e)))?;
    
    // Initialize components
    let capture = Arc::new(PacketCapture::new(config.capture_interface.clone())?);
    let parser = Arc::new(ProtocolParser::new());
    let flow_tracker = Arc::new(FlowTracker::new(config.max_flows));
    let feature_extractor = Arc::new(FeatureExtractor::new());
    let mut envelope_builder = EnvelopeBuilder::new(
        "dpi_probe".to_string(),
        identity.component_id().to_string(),
    );
    let backpressure = Arc::new(BackpressureManager::new(config.max_queue_size));
    let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_tokens, config.rate_limit_refill));
    let health_monitor = Arc::new(HealthMonitor::new(300)); // 5 minute max idle
    
    // Start capture (optional and explicit)
    capture.start()?;
    
    info!("DPI Probe started successfully");
    info!("Capturing on interface: {}", config.capture_interface);
    
    // Main processing loop
    let mut packet_count = 0u64;
    loop {
        // Record watchdog heartbeat
        hardening.heartbeat();
        
        // Perform periodic runtime checks (every 1000 packets)
        if packet_count % 1000 == 0 {
            if let Err(e) = hardening.perform_runtime_checks() {
                error!("Runtime check failed: {}, stopping", e);
                hardening.stop_watchdog();
                return Err(ProbeError::ConfigurationError(format!("Runtime hardening violation: {}", e)));
            }
            
            // Check for tamper detection
            if hardening.is_tampered() {
                error!("Tamper detected, stopping immediately");
                hardening.stop_watchdog();
                return Err(ProbeError::ConfigurationError("Tamper detected - fail-closed".to_string()));
            }
        }
        
        // Check health
        if !health_monitor.check_health()? {
            error!("Health check failed, stopping");
            hardening.stop_watchdog();
            break;
        }
        
        // Read packet
        match capture.next_packet()? {
            Some(packet_data) => {
                packet_count += 1;
                health_monitor.record_packet();
                
                // Check backpressure
                let queue_size = 0; // Would be actual queue size in production
                backpressure.update_queue_size(queue_size);
                
                if backpressure.should_drop(queue_size) {
                    backpressure.signal();
                    continue; // Drop packet
                }
                
                // Check rate limit
                if !rate_limiter.allow()? {
                    continue; // Drop packet
                }
                
                // Parse packet
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                let parsed = match parser.parse(&packet_data, timestamp) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Parse error: {}", e);
                        health_monitor.record_error();
                        continue;
                    }
                };
                
                // Update flow tracking
                if let Err(e) = flow_tracker.update_flow(&parsed) {
                    error!("Flow tracking error: {}", e);
                    health_monitor.record_error();
                }
                
                // Get flow for feature extraction
                let flow_key = flow::FlowKey::from_packet(&parsed);
                let flow = flow_key.as_ref()
                    .and_then(|k| flow_tracker.get_flow(k));
                
                // Create envelope data from parsed packet
                let envelope_data = {
                    let mut data = Vec::new();
                    data.extend_from_slice(&parsed.timestamp.to_be_bytes());
                    if let Some(ref ip) = parsed.src_ip {
                        data.extend_from_slice(ip.as_bytes());
                    }
                    if let Some(ref ip) = parsed.dst_ip {
                        data.extend_from_slice(ip.as_bytes());
                    }
                    data
                };
                
                // Extract features
                let features = match feature_extractor.extract(&parsed, flow.as_ref()) {
                    Ok(f) => f,
                    Err(e) => {
                        error!("Feature extraction error: {}", e);
                        health_monitor.record_error();
                        continue;
                    }
                };
                
                // Sign envelope data
                
                let signature = signer.sign(&envelope_data)
                    .map_err(|e| ProbeError::SigningFailed(format!("{}", e)))?;
                
                let envelope = envelope_builder.build(&parsed, &features, signature)?;
                
                info!("Event envelope created: {} (sequence: {})", 
                    envelope.event_id, envelope.sequence);
                
                // Step 1: Serialize EventEnvelope to canonical JSON bytes
                let canonical_bytes = serde_json::to_vec(&envelope)
                    .map_err(|e| ProbeError::ConfigurationError(format!("Failed to serialize envelope: {}", e)))?;
                
                // Step 2: SHA-256 hash of canonical bytes
                let mut hasher = Sha256::new();
                hasher.update(&canonical_bytes);
                let hash_bytes = hasher.finalize();
                let payload_hash = hex::encode(hash_bytes);
                
                info!("Signing payload hash={} envelope_id={}", payload_hash, envelope.event_id);
                
                // Step 3: Sign the hash (using Ed25519 signer)
                // Note: The envelope already has a signature, but we need to sign the hash
                // For now, we'll use the existing signature from the envelope
                // In production, this should be a proper hash signature
                let signature_b64 = envelope.signature.clone();
                
                // Step 4: Create SignedEvent with new format
                use serde_json::json;
                let signed_event = json!({
                    "envelope": serde_json::from_slice::<serde_json::Value>(&canonical_bytes)
                        .map_err(|e| ProbeError::ConfigurationError(format!("Failed to parse envelope JSON: {}", e)))?,
                    "payload_hash": payload_hash,
                    "signature": signature_b64,
                    "signer_id": identity.component_id(),
                });
                
                // Send directly via HTTP POST (async call in sync context)
                let url = format!("{}/ingest/dpi", core_api_url);
                let client_clone = http_client.clone();
                let envelope_id = envelope.event_id.clone();
                
                info!("POST /ingest/dpi");
                
                match rt.block_on(async move {
                    let res = client_clone
                        .post(&url)
                        .json(&signed_event)
                        .send()
                        .await?;
                    Ok::<_, reqwest::Error>(res)
                }) {
                    Ok(res) => {
                        if res.status().is_success() {
                            info!("POST {} -> {} OK | Telemetry delivered: {}", url, res.status(), envelope_id);
                        } else {
                            error!("Failed to send event {}: HTTP {}", envelope_id, res.status());
                        }
                    }
                    Err(e) => {
                        error!("Failed to send event {}: {}", envelope_id, e);
                    }
                }
            }
            None => {
                // Timeout, continue
                continue;
            }
        }
        
        // Periodic stats
        if packet_count % 10000 == 0 {
            let stats = capture.stats();
            let flow_count = flow_tracker.flow_count();
            let bp_stats = backpressure.stats();
            let health_stats = health_monitor.stats();
            
            info!("Stats: packets={}, flows={}, dropped={}, healthy={}", 
                stats.packets_captured, flow_count, bp_stats.packets_dropped, health_stats.healthy);
        }
    }
    
    capture.stop();
    hardening.stop_watchdog();
    info!("DPI Probe stopped");
    Ok(())
}

