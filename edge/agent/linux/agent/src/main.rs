// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/agent/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Linux Agent main entry point - standalone host telemetry sensor

use std::sync::Arc;
use tracing::{info, error};
use tokio::runtime::Runtime;
use libsystemd::daemon::{notify, NotifyState};

mod errors;
mod process;
mod filesystem;
mod network;
mod syscalls;
mod features;
mod envelope;
mod backpressure;
mod rate_limit;
mod health;
mod hardening;

#[path = "../security/mod.rs"]
mod security;

#[path = "../../config/validation.rs"]
mod config_validation;

// Import signing from parent src/
#[path = "../../src/signing.rs"]
mod signing;

use errors::AgentError;
use process::ProcessMonitor;
use filesystem::FilesystemMonitor;
use network::NetworkMonitor;
use syscalls::SyscallMonitor;
use features::FeatureExtractor;
use envelope::EnvelopeBuilder;
use backpressure::BackpressureManager;
use rate_limit::RateLimiter;
use health::HealthMonitor;
use security::{IdentityManager, EventSigner as SecurityEventSigner};
use config_validation::AgentConfig;
use reqwest::Client as ReqwestClient;

fn main() -> Result<(), AgentError> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("[INIT-1] RansomEye Linux Agent starting...");
    let _ = notify(false, &[NotifyState::Status("Initializing...".to_string())]);
    
    // Get binary path for integrity verification
    info!("[INIT-2] Getting binary path for integrity verification");
    let binary_path = std::env::current_exe()
        .map_err(|e| AgentError::ConfigurationError(format!("Failed to get binary path: {}", e)))?
        .to_string_lossy()
        .to_string();
    
    // Initialize runtime hardening (FAIL-CLOSED on integrity failure)
    info!("[INIT-3] Initializing runtime hardening");
    let _ = notify(false, &[NotifyState::Status("Initializing runtime hardening...".to_string())]);
    let config_path = std::env::var("AGENT_CONFIG_PATH").ok();
    let hardening = hardening::RuntimeHardening::new(
        binary_path.clone(),
        config_path.clone(),
        30, // 30 second watchdog interval
    ).map_err(|e| AgentError::ConfigurationError(format!("Hardening initialization failed: {}", e)))?;
    
    // Verify binary integrity at startup (FAIL-CLOSED)
    info!("[INIT-4] Verifying binary integrity");
    let _ = notify(false, &[NotifyState::Status("Verifying binary integrity...".to_string())]);
    hardening.verify_binary_integrity()
        .map_err(|e| AgentError::ConfigurationError(format!("Binary integrity check failed: {}", e)))?;
    info!("[INIT-4] Binary integrity verified");
    
    // Verify config integrity at startup (FAIL-CLOSED)
    info!("[INIT-5] Verifying config integrity");
    let _ = notify(false, &[NotifyState::Status("Verifying config integrity...".to_string())]);
    hardening.verify_config_integrity()
        .map_err(|e| AgentError::ConfigurationError(format!("Config integrity check failed: {}", e)))?;
    info!("[INIT-5] Config integrity verified");
    
    // Perform runtime tamper checks (FAIL-CLOSED)
    info!("[INIT-6] Performing runtime tamper checks");
    let _ = notify(false, &[NotifyState::Status("Performing runtime checks...".to_string())]);
    hardening.perform_runtime_checks()
        .map_err(|e| AgentError::ConfigurationError(format!("Runtime check failed: {}", e)))?;
    info!("[INIT-6] Runtime checks passed");
    
    // Start watchdog timer
    info!("[INIT-7] Starting watchdog timer");
    let _ = notify(false, &[NotifyState::Status("Starting watchdog...".to_string())]);
    hardening.start_watchdog()
        .map_err(|e| AgentError::ConfigurationError(format!("Watchdog start failed: {}", e)))?;
    info!("[INIT-7] Watchdog started");
    
    // Load configuration (ENV-only, fail-closed)
    info!("[INIT-8] Loading configuration from environment");
    let _ = notify(false, &[NotifyState::Status("Loading configuration...".to_string())]);
    let config = AgentConfig::from_env()
        .map_err(|e| AgentError::ConfigurationError(e))?;
    
    config.validate()
        .map_err(|e| AgentError::ConfigurationError(e))?;
    
    info!("[INIT-8] Configuration loaded: max_processes={}, max_connections={}", 
        config.max_processes, config.max_connections);
    
    // Initialize identity (fail-closed on failure)
    info!("[INIT-9] Loading component identity");
    let _ = notify(false, &[NotifyState::Status("Loading identity...".to_string())]);
    let identity_path = config.identity_path.as_ref().map(|p| std::path::Path::new(p));
    let identity = IdentityManager::load_or_create(identity_path)
        .map_err(|e| AgentError::IdentityVerificationFailed(format!("{}", e)))?;
    
    info!("[INIT-9] Component identity: {}", identity.component_id());
    
    // Initialize event signer (fail-closed on failure) - Ed25519
    info!("[INIT-10] Loading signing key");
    let _ = notify(false, &[NotifyState::Status("Loading signing key...".to_string())]);
    let component_id = identity.component_id().to_string();
    let security_signer = if let Some(ref key_path) = config.signing_key_path {
        info!("Loading signing key from: {}", key_path);
        SecurityEventSigner::from_key_file(std::path::Path::new(key_path))
            .map_err(|e| {
                error!("Failed to load Ed25519 key from {}: {}", key_path, e);
                AgentError::SigningFailed(format!("Failed to load Ed25519 key: {}", e))
            })?
    } else {
        return Err(AgentError::SigningFailed("AGENT_SIGNING_KEY_PATH must be set".to_string()));
    };
    
    // Test signer BEFORE wrapping in Arc to catch any issues
    info!("[INIT-10] Testing signer");
    let test_data = b"test";
    match security_signer.sign(test_data) {
        Ok(sig) => {
            info!("Signer test successful: signature length={}", sig.len());
        }
        Err(e) => {
            error!("Signer test failed: {}", e);
            return Err(AgentError::SigningFailed(format!("Signer test failed: {}", e)));
        }
    }
    
    let security_signer = Arc::new(security_signer);
    info!("[INIT-10] Event signer created with Ed25519 key");
    
    // Initialize reqwest HTTP client for direct telemetry delivery
    info!("[INIT-11] Initializing HTTP client");
    let _ = notify(false, &[NotifyState::Status("Initializing HTTP client...".to_string())]);
    let http_client = ReqwestClient::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AgentError::ConfigurationError(format!("Failed to create HTTP client: {}", e)))?;
    
    let core_api_url = config.core_api_url.clone();
    info!("[INIT-11] HTTP client initialized for direct delivery to {}", core_api_url);
    info!("Core API URL: {}", core_api_url);
    
    // CRITICAL: TLS/identity initialization MUST only occur for HTTPS URLs
    // If TransportClient or TLS initialization is added in the future, it must be gated:
    // if core_api_url.starts_with("https://") {
    //     // Initialize TLS transport and identity certificates
    // } else {
    //     // HTTP mode - no TLS initialization
    // }
    
    info!("[INIT-12] Initializing monitoring components");
    let _ = notify(false, &[NotifyState::Status("Initializing components...".to_string())]);
    
    // Initialize components
    let process_monitor = Arc::new(ProcessMonitor::new(config.max_processes));
    let _fs_monitor = Arc::new(FilesystemMonitor::new(config.mass_write_threshold));
    let network_monitor = Arc::new(NetworkMonitor::new(config.max_connections));
    let syscall_monitor = Arc::new(SyscallMonitor::new());
    let feature_extractor = Arc::new(FeatureExtractor::new());
    let mut envelope_builder = EnvelopeBuilder::new(
        "linux_agent".to_string(),
        identity.component_id().to_string(),
    );
    let backpressure = Arc::new(BackpressureManager::new(config.max_queue_size));
    let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_tokens, config.rate_limit_refill));
    let health_monitor = Arc::new(HealthMonitor::new(300)); // 5 minute max idle
    
    // Initialize syscall monitoring
    info!("[INIT-13] Initializing syscall monitoring");
    let _ = notify(false, &[NotifyState::Status("Initializing syscall monitoring...".to_string())]);
    if config.enable_ebpf {
        if let Err(e) = syscall_monitor.init_ebpf() {
            error!("eBPF initialization failed: {}", e);
            if config.enable_auditd {
                info!("Falling back to auditd");
                syscall_monitor.init_auditd()?;
            } else {
                return Err(e);
            }
        } else {
            info!("eBPF syscall monitoring initialized");
        }
    } else if config.enable_auditd {
        syscall_monitor.init_auditd()?;
        info!("auditd syscall monitoring initialized");
    }
    
    // Start monitoring
    info!("[INIT-14] Starting syscall monitoring");
    let _ = notify(false, &[NotifyState::Status("Starting monitoring...".to_string())]);
    syscall_monitor.start()?;
    info!("[INIT-14] Syscall monitoring started");
    
    // Create tokio runtime for async transport calls
    info!("[INIT-15] Creating tokio runtime");
    let rt = Runtime::new()
        .map_err(|e| AgentError::ConfigurationError(format!("Failed to create runtime: {}", e)))?;
    
    // All initialization complete - notify systemd we're ready
    info!("[INIT-16] Linux Agent initialization complete - notifying systemd");
    let _ = notify(true, &[NotifyState::Ready, NotifyState::Status("Running".to_string())]);
    info!("Linux Agent started successfully and ready");
    
    // Create tokio runtime for async transport calls
    let rt = Runtime::new()
        .map_err(|e| AgentError::ConfigurationError(format!("Failed to create runtime: {}", e)))?;
    
    // Main processing loop
    let mut event_count = 0u64;
    loop {
        // Record watchdog heartbeat
        hardening.heartbeat();
        
        // Perform periodic runtime checks (every 1000 events)
        if event_count % 1000 == 0 {
            if let Err(e) = hardening.perform_runtime_checks() {
                error!("Runtime check failed: {}, stopping", e);
                hardening.stop_watchdog();
                return Err(AgentError::ConfigurationError(format!("Runtime hardening violation: {}", e)));
            }
            
            // Check for tamper detection
            if hardening.is_tampered() {
                error!("Tamper detected, stopping immediately");
                hardening.stop_watchdog();
                return Err(AgentError::ConfigurationError("Tamper detected - fail-closed".to_string()));
            }
        }
        
        // Check health
        if !health_monitor.check_health()? {
            error!("Health check failed, stopping");
            hardening.stop_watchdog();
            break;
        }
        
        // Check backpressure
        let queue_size = 0; // Would be actual queue size in production
        backpressure.update_queue_size(queue_size);
        
        if backpressure.should_drop(queue_size) {
            backpressure.signal();
            continue;
        }
        
        // Check rate limit
        if !rate_limiter.allow()? {
            continue;
        }
        
        // Generate and send events (at least once per second)
        if event_count % 100 == 0 || event_count == 0 {
            // Simulate process exec event
            let process_event = process_monitor.record_exec(
                (1234 + (event_count % 10000)) as u32,
                Some(1000),
                1000,
                1000,
                "/usr/bin/test".to_string(),
                Some("test --arg".to_string()),
            )?;
            
            let features = feature_extractor.extract_from_process(&process_event)?;
            
            let envelope_data = serde_json::to_vec(&process_event)
                .map_err(|e| AgentError::EnvelopeCreationFailed(format!("{}", e)))?;
            
            let signature = security_signer.sign(&envelope_data)
                .map_err(|e| AgentError::SigningFailed(format!("{}", e)))?;
            
            let envelope = envelope_builder.build_from_process(&process_event, &features, signature)?;
            
            health_monitor.record_event();
            
            info!("Event envelope created: {} (sequence: {})", 
                envelope.event_id, envelope.sequence);
            
            // Step 1: Serialize EventEnvelope to canonical JSON bytes
            let canonical_bytes = serde_json::to_vec(&envelope)
                .map_err(|e| AgentError::EnvelopeCreationFailed(format!("Failed to serialize envelope: {}", e)))?;
            
            // Step 2: SHA-256 hash of canonical bytes
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&canonical_bytes);
            let hash_bytes = hasher.finalize();
            let payload_hash = hex::encode(hash_bytes);
            
            info!("Signing payload hash={} envelope_id={}", payload_hash, envelope.event_id);
            
            // Step 3: Sign the hash using Ed25519 (via SecurityEventSigner)
            // SecurityEventSigner.sign() includes sequence number, so we sign the hash directly
            info!("About to sign payload hash (length: {})", hash_bytes.len());
            let signature = security_signer.sign(&hash_bytes)
                .map_err(|e| {
                    error!("Signing failed with error: {}", e);
                    AgentError::SigningFailed(format!("Failed to sign hash with Ed25519: {}", e))
                })?;
            info!("Successfully signed payload hash");
            
            // Step 4: Create SignedEvent with new format
            use serde_json::json;
            let signed_event = json!({
                "envelope": serde_json::from_slice::<serde_json::Value>(&canonical_bytes)
                    .map_err(|e| AgentError::EnvelopeCreationFailed(format!("Failed to parse envelope JSON: {}", e)))?,
                "payload_hash": payload_hash,
                "signature": signature,
                "signer_id": component_id,
            });
            
            // Send directly via HTTP POST (async call in sync context)
            let url = format!("{}/ingest/linux", core_api_url);
            let url_clone = url.clone();
            let client_clone = http_client.clone();
            let envelope_id = envelope.event_id.clone();
            
            info!("POST /ingest/linux");
            
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
                        info!("POST {} -> {} OK | Telemetry delivered: {}", url_clone, res.status(), envelope_id);
                    } else {
                        error!("Failed to send event {}: HTTP {}", envelope_id, res.status());
                    }
                }
                Err(e) => {
                    error!("Failed to send event {}: {}", envelope_id, e);
                }
            }
        }
        
        event_count += 1;
        
        // Periodic stats
        if event_count % 10000 == 0 {
            let process_count = process_monitor.process_count();
            let connection_count = network_monitor.connection_count();
            let bp_stats = backpressure.stats();
            let health_stats = health_monitor.stats();
            
            info!("Stats: events={}, processes={}, connections={}, dropped={}, healthy={}", 
                event_count, process_count, connection_count, bp_stats.events_dropped, health_stats.healthy);
        }
    }
    
    syscall_monitor.stop();
    hardening.stop_watchdog();
    info!("Linux Agent stopped");
    Ok(())
}

