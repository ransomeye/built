// Path and File Name : /home/ransomeye/rebuild/edge/sentinel/src/hardening.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime hardening for Sentinel - watchdog, integrity, tamper detection, fail-closed

use std::path::Path;
use std::fs;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;
use sha2::{Sha256, Digest};
use hex;
use tracing::{error, warn, info};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HardeningError {
    #[error("Binary integrity check failed: {0}")]
    BinaryIntegrityFailed(String),
    #[error("Config integrity check failed: {0}")]
    ConfigIntegrityFailed(String),
    #[error("Tamper detected: {0}")]
    TamperDetected(String),
    #[error("Watchdog failure: {0}")]
    WatchdogFailure(String),
    #[error("Network isolation violation: {0}")]
    NetworkIsolationViolation(String),
    #[error("Process hardening failure: {0}")]
    ProcessHardeningFailure(String),
}

/// Runtime hardening manager for Sentinel
pub struct RuntimeHardening {
    binary_path: String,
    binary_hash: String,
    config_path: Option<String>,
    config_hash: Option<String>,
    watchdog_interval: Duration,
    last_heartbeat: Arc<AtomicU64>,
    crash_count: Arc<AtomicU64>,
    watchdog_running: Arc<AtomicBool>,
    tamper_detected: Arc<AtomicBool>,
}

impl RuntimeHardening {
    pub fn new(
        binary_path: String,
        config_path: Option<String>,
        watchdog_interval_secs: u64,
    ) -> Result<Self, HardeningError> {
        if !Path::new(&binary_path).exists() {
            return Err(HardeningError::BinaryIntegrityFailed(
                format!("Binary not found: {}", binary_path)
            ));
        }

        let binary_hash = Self::compute_file_hash(&binary_path)?;
        info!("Binary integrity verified: {} (hash: {})", binary_path, binary_hash);

        let config_hash = if let Some(ref cfg_path) = config_path {
            if !Path::new(cfg_path).exists() {
                return Err(HardeningError::ConfigIntegrityFailed(
                    format!("Config file not found: {}", cfg_path)
                ));
            }
            Some(Self::compute_file_hash(cfg_path)?)
        } else {
            None
        };

        if let Some(ref hash) = config_hash {
            info!("Config integrity verified: {} (hash: {})", 
                config_path.as_ref().unwrap(), hash);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            binary_path,
            binary_hash,
            config_path,
            config_hash,
            watchdog_interval: Duration::from_secs(watchdog_interval_secs),
            last_heartbeat: Arc::new(AtomicU64::new(now)),
            crash_count: Arc::new(AtomicU64::new(0)),
            watchdog_running: Arc::new(AtomicBool::new(false)),
            tamper_detected: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn verify_binary_integrity(&self) -> Result<(), HardeningError> {
        let current_hash = Self::compute_file_hash(&self.binary_path)?;
        
        if current_hash != self.binary_hash {
            error!("BINARY TAMPER DETECTED: Hash mismatch (expected: {}, got: {})", 
                self.binary_hash, current_hash);
            self.tamper_detected.store(true, Ordering::Release);
            return Err(HardeningError::TamperDetected(
                format!("Binary hash mismatch: expected {}, got {}", 
                    self.binary_hash, current_hash)
            ));
        }

        Ok(())
    }

    pub fn verify_config_integrity(&self) -> Result<(), HardeningError> {
        if let (Some(ref cfg_path), Some(ref expected_hash)) = (&self.config_path, &self.config_hash) {
            let current_hash = Self::compute_file_hash(cfg_path)?;
            
            if current_hash != *expected_hash {
                error!("CONFIG TAMPER DETECTED: Hash mismatch (expected: {}, got: {})", 
                    expected_hash, current_hash);
                self.tamper_detected.store(true, Ordering::Release);
                return Err(HardeningError::TamperDetected(
                    format!("Config hash mismatch: expected {}, got {}", 
                        expected_hash, current_hash)
                ));
            }
        }

        Ok(())
    }

    pub fn start_watchdog(&self) -> Result<(), HardeningError> {
        if self.watchdog_running.swap(true, Ordering::Acquire) {
            return Err(HardeningError::WatchdogFailure("Watchdog already running".to_string()));
        }

        let last_heartbeat = self.last_heartbeat.clone();
        let watchdog_interval = self.watchdog_interval;
        let crash_count = self.crash_count.clone();
        let watchdog_running = self.watchdog_running.clone();
        let tamper_detected = self.tamper_detected.clone();
        let binary_path = self.binary_path.clone();
        let binary_hash = self.binary_hash.clone();

        thread::spawn(move || {
            while watchdog_running.load(Ordering::Acquire) {
                thread::sleep(watchdog_interval);

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let last_beat = last_heartbeat.load(Ordering::Acquire);
                let elapsed = now.saturating_sub(last_beat);

                if elapsed > watchdog_interval.as_secs() * 2 {
                    error!("WATCHDOG TIMEOUT: No heartbeat for {} seconds", elapsed);
                    
                    if let Err(e) = Self::verify_binary_integrity_static(&binary_path, &binary_hash) {
                        error!("WATCHDOG: Binary tamper detected: {}", e);
                        tamper_detected.store(true, Ordering::Release);
                    }

                    let crashes = crash_count.fetch_add(1, Ordering::AcqRel) + 1;
                    
                    if crashes >= 3 {
                        error!("WATCHDOG: Repeated crashes detected ({}), escalating alert", crashes);
                    }
                }
            }
        });

        info!("Watchdog started (interval: {}s)", watchdog_interval.as_secs());
        Ok(())
    }

    pub fn heartbeat(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_heartbeat.store(now, Ordering::Release);
    }

    pub fn stop_watchdog(&self) {
        self.watchdog_running.store(false, Ordering::Release);
    }

    pub fn is_tampered(&self) -> bool {
        self.tamper_detected.load(Ordering::Acquire)
    }

    pub fn verify_network_isolation(&self) -> Result<(), HardeningError> {
        // Sentinel should not open arbitrary listeners
        let tcp_file = "/proc/net/tcp";
        if Path::new(tcp_file).exists() {
            let content = fs::read_to_string(tcp_file)
                .map_err(|e| HardeningError::NetworkIsolationViolation(
                    format!("Failed to read {}: {}", tcp_file, e)
                ))?;

            let listening_count = content.lines()
                .skip(1)
                .filter(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        parts[3] == "0A" // LISTEN state
                    } else {
                        false
                    }
                })
                .count();

            if listening_count > 0 {
                warn!("Network isolation check: {} listening sockets detected (expected: 0)", 
                    listening_count);
            }
        }

        Ok(())
    }

    pub fn detect_debugger(&self) -> Result<bool, HardeningError> {
        let status_file = "/proc/self/status";
        if Path::new(status_file).exists() {
            let content = fs::read_to_string(status_file)
                .map_err(|e| HardeningError::ProcessHardeningFailure(
                    format!("Failed to read {}: {}", status_file, e)
                ))?;

            for line in content.lines() {
                if line.starts_with("TracerPid:") {
                    let pid_str = line.split_whitespace().nth(1)
                        .unwrap_or("0");
                    let tracer_pid: u32 = pid_str.parse().unwrap_or(0);
                    
                    if tracer_pid != 0 {
                        error!("DEBUGGER DETECTED: TracerPid = {}", tracer_pid);
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    pub fn detect_ld_preload(&self) -> Result<bool, HardeningError> {
        if let Ok(preload) = std::env::var("LD_PRELOAD") {
            if !preload.is_empty() {
                error!("LD_PRELOAD INJECTION DETECTED: {}", preload);
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn perform_runtime_checks(&self) -> Result<(), HardeningError> {
        self.verify_binary_integrity()?;
        self.verify_config_integrity()?;

        if self.detect_debugger()? {
            return Err(HardeningError::TamperDetected(
                "Debugger attachment detected".to_string()
            ));
        }

        if self.detect_ld_preload()? {
            return Err(HardeningError::TamperDetected(
                "LD_PRELOAD injection detected".to_string()
            ));
        }

        self.verify_network_isolation()?;

        Ok(())
    }

    pub fn crash_count(&self) -> u64 {
        self.crash_count.load(Ordering::Acquire)
    }

    pub fn reset_crash_count(&self) {
        self.crash_count.store(0, Ordering::Release);
    }

    fn compute_file_hash(file_path: &str) -> Result<String, HardeningError> {
        let data = fs::read(file_path)
            .map_err(|e| HardeningError::BinaryIntegrityFailed(
                format!("Failed to read file {}: {}", file_path, e)
            ))?;

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hasher.finalize();

        Ok(hex::encode(hash))
    }

    fn verify_binary_integrity_static(binary_path: &str, expected_hash: &str) -> Result<(), HardeningError> {
        let current_hash = Self::compute_file_hash(binary_path)?;
        
        if current_hash != expected_hash {
            return Err(HardeningError::TamperDetected(
                format!("Binary hash mismatch: expected {}, got {}", 
                    expected_hash, current_hash)
            ));
        }

        Ok(())
    }
}

