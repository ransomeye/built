// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/agent/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Linux Agent main entry point - standalone host telemetry sensor

use std::sync::Arc;
use tracing::{info, error, warn};
use tokio::runtime::Runtime;
use libsystemd::daemon::{notify, NotifyState};
use crossbeam_channel;
use process::{ProcessEvent, ProcessEventType};

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

#[path = "../../src/system_metrics.rs"]
mod system_metrics;

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
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Process information from /proc filesystem
struct ProcProcessInfo {
    pid: i32,
    ppid: i32,
    uid: u32,
    gid: u32,
    executable: String,
    command_line: String,
}

impl AgentConfig {
    /// Scan /proc filesystem for current process IDs
    fn scan_proc_processes() -> std::collections::HashSet<i32> {
        let mut pids = std::collections::HashSet::new();
        
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(pid_str) = entry.file_name().into_string() {
                    if let Ok(pid) = pid_str.parse::<i32>() {
                        pids.insert(pid);
                    }
                }
            }
        }
        
        pids
    }
    
    /// Read process information from /proc filesystem
    fn read_proc_process_info(pid: i32) -> Option<ProcProcessInfo> {
        let pid_dir = format!("/proc/{}", pid);
        let pid_path = Path::new(&pid_dir);
        
        if !pid_path.exists() {
            return None;
        }
        
        // Read process name from comm
        let executable = fs::read_to_string(pid_path.join("comm"))
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();
        
        // Read command line from cmdline
        let command_line = fs::read_to_string(pid_path.join("cmdline"))
            .unwrap_or_else(|_| String::new())
            .replace('\0', " ")
            .trim()
            .to_string();
        
        // Read stat for ppid
        let mut ppid = 0;
        if let Ok(stat_content) = fs::read_to_string(pid_path.join("stat")) {
            let fields: Vec<&str> = stat_content.split_whitespace().collect();
            if fields.len() > 3 {
                ppid = fields[3].parse().unwrap_or(0);
            }
        }
        
        // Read status for uid/gid
        let mut uid = 0;
        let mut gid = 0;
        if let Ok(status_content) = fs::read_to_string(pid_path.join("status")) {
            for line in status_content.lines() {
                if line.starts_with("Uid:") {
                    if let Some(uid_str) = line.split_whitespace().nth(1) {
                        uid = uid_str.parse().unwrap_or(0);
                    }
                }
                if line.starts_with("Gid:") {
                    if let Some(gid_str) = line.split_whitespace().nth(1) {
                        gid = gid_str.parse().unwrap_or(0);
                    }
                }
            }
        }
        
        Some(ProcProcessInfo {
            pid,
            ppid,
            uid,
            gid,
            executable,
            command_line,
        })
    }
}

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
    
    // Initialize system metrics collector
    let mut system_metrics_collector = system_metrics::SystemMetricsCollector::new();
    let system_metrics_state = Arc::new(std::sync::Mutex::new(None::<system_metrics::SystemMetrics>));
    
    // Start periodic system metrics collection (every 20 seconds)
    let metrics_state_clone = system_metrics_state.clone();
    std::thread::spawn(move || {
        info!("[SYS_METRICS] collection thread started (interval=20s)");
        let mut collector = system_metrics::SystemMetricsCollector::new();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(20));
            info!("[SYS_METRICS] tick");
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| collector.collect())) {
                Ok(metrics) => {
                    if metrics.has_real_metrics() {
                        if let Ok(mut state) = metrics_state_clone.lock() {
                            *state = Some(metrics);
                            info!("[SYS_METRICS] collected and stored successfully");
                        }
                    } else {
                        warn!("[SYS_METRICS][WARN] collected metrics but all values are null — discarding");
                    }
                }
                Err(_) => {
                    error!("[SYS_METRICS][PANIC] collector panicked, metrics collection stopped");
                    break;
                }
            }
        }
    });
    
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
    
    // Channel for real process events from monitoring
    use crossbeam_channel::{bounded, Receiver, Sender};
    let (event_tx, event_rx): (Sender<ProcessEvent>, Receiver<ProcessEvent>) = bounded(10000);
    
    // Start real process monitoring task (scans /proc filesystem)
    let running_monitor = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let monitor_tx = event_tx.clone();
    let monitor_running = running_monitor.clone();
    
    std::thread::spawn(move || {
        let mut last_pids = std::collections::HashSet::new();
        
        while monitor_running.load(std::sync::atomic::Ordering::Relaxed) {
            // Scan /proc for current processes
            let current_pids = AgentConfig::scan_proc_processes();
            
            // Detect new processes
            for pid in &current_pids {
                if !last_pids.contains(pid) {
                    if let Some(proc_info) = AgentConfig::read_proc_process_info(*pid) {
                        // Create ProcessEvent from real process data
                        let process_event = ProcessEvent {
                            event_type: ProcessEventType::Exec,
                            pid: *pid as u32,
                            ppid: Some(proc_info.ppid as u32),
                            uid: proc_info.uid,
                            gid: proc_info.gid,
                            executable: Some(proc_info.executable),
                            command_line: Some(proc_info.command_line),
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            mmap_address: None,
                            mmap_size: None,
                        };
                        
                        if monitor_tx.try_send(process_event).is_err() {
                            warn!("Process event queue full, dropping event");
                        }
                    }
                }
            }
            
            last_pids = current_pids;
            
            // Scan interval: 1 second
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    
    // Main processing loop - processes REAL events from monitoring
    let mut event_count = 0u64;
    loop {
        // Record watchdog heartbeat
        hardening.heartbeat();
        
        // Perform periodic runtime checks (every 1000 events)
        if event_count % 1000 == 0 {
            if let Err(e) = hardening.perform_runtime_checks() {
                error!("Runtime check failed: {}, stopping", e);
                hardening.stop_watchdog();
                running_monitor.store(false, std::sync::atomic::Ordering::Relaxed);
                return Err(AgentError::ConfigurationError(format!("Runtime hardening violation: {}", e)));
            }
            
            // Check for tamper detection
            if hardening.is_tampered() {
                error!("Tamper detected, stopping immediately");
                hardening.stop_watchdog();
                running_monitor.store(false, std::sync::atomic::Ordering::Relaxed);
                return Err(AgentError::ConfigurationError("Tamper detected - fail-closed".to_string()));
            }
        }
        
        // Check health
        if !health_monitor.check_health()? {
            error!("Health check failed, stopping");
            hardening.stop_watchdog();
            running_monitor.store(false, std::sync::atomic::Ordering::Relaxed);
            break;
        }
        
        // Check backpressure
        let queue_size = event_rx.len();
        backpressure.update_queue_size(queue_size);
        
        if backpressure.should_drop(queue_size) {
            backpressure.signal();
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }
        
        // Check rate limit
        if !rate_limiter.allow()? {
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }
        
        // Receive REAL process events from monitoring (non-blocking)
        match event_rx.try_recv() {
            Ok(process_event) => {
                // Record the real event in process monitor
                if let (Some(exec), Some(cmd)) = (process_event.executable.clone(), process_event.command_line.clone()) {
                    let _ = process_monitor.record_exec(
                        process_event.pid,
                        process_event.ppid,
                        process_event.uid,
                        process_event.gid,
                        exec,
                        Some(cmd),
                    );
                }
                
                let features = feature_extractor.extract_from_process(&process_event)?;
                
                let envelope_data = serde_json::to_vec(&process_event)
                    .map_err(|e| AgentError::EnvelopeCreationFailed(format!("{}", e)))?;
                
                let signature = security_signer.sign(&envelope_data)
                    .map_err(|e| AgentError::SigningFailed(format!("{}", e)))?;
                
                let mut envelope = envelope_builder.build_from_process(&process_event, &features, signature)?;
                
                // Inject system metrics into envelope.data.system (only if valid)
                let (system_json, has_metrics) = if let Ok(metrics_guard) = system_metrics_state.lock() {
                    if let Some(ref metrics) = *metrics_guard {
                        if metrics.has_real_metrics() {
                            (serde_json::to_value(metrics)
                                .unwrap_or_else(|_| serde_json::json!({})), true)
                        } else {
                            warn!("[ENVELOPE][WARN] system metrics present but invalid (all null), injecting empty object");
                            (serde_json::json!({
                                "cpu": {},
                                "memory": {},
                                "disk": {},
                                "filesystem": {"mounts": []},
                                "network": {},
                                "system_state": {}
                            }), false)
                        }
                    } else {
                        // Empty system metrics structure if not yet collected
                        (serde_json::json!({
                            "cpu": {},
                            "memory": {},
                            "disk": {},
                            "filesystem": {"mounts": []},
                            "network": {},
                            "system_state": {}
                        }), false)
                    }
                } else {
                    (serde_json::json!({
                        "cpu": {},
                        "memory": {},
                        "disk": {},
                        "filesystem": {"mounts": []},
                        "network": {},
                        "system_state": {}
                    }), false)
                };
                if has_metrics {
                    info!("[ENVELOPE] system metrics injected (non-empty)");
                } else {
                    warn!("[ENVELOPE][WARN] system metrics missing, injecting empty object");
                }
                envelope.data.system = Some(system_json);
                
                health_monitor.record_event();
                event_count += 1;
                
                info!("Event envelope created from REAL process: {} (sequence: {}, pid: {})", 
                    envelope.event_id, envelope.sequence, process_event.pid);
                
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
            Err(crossbeam_channel::TryRecvError::Empty) => {
                // No events available, sleep briefly
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                error!("Event channel disconnected");
                break;
            }
        }
        
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
    
    running_monitor.store(false, std::sync::atomic::Ordering::Relaxed);
    
    syscall_monitor.stop();
    hardening.stop_watchdog();
    info!("Linux Agent stopped");
    Ok(())
}

