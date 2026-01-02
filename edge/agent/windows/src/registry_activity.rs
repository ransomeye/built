// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/src/registry_activity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Registry activity monitoring on Windows using RegNotifyChangeKeyValue - observes registry create/modify/delete (NO enforcement, NO blocking)

#[cfg(windows)]
use std::sync::Arc;
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use chrono::Utc;
#[cfg(windows)]
use tracing::{warn, debug, error};
#[cfg(windows)]
use crossbeam_channel::Sender;
#[cfg(windows)]
use crate::event::RegistryEvent;

/// Monitors Windows Registry activity using RegNotifyChangeKeyValue
/// OBSERVATION ONLY - never enforces, never blocks, never modifies registry
#[cfg(windows)]
pub struct RegistryActivityMonitor;

#[cfg(windows)]
impl RegistryActivityMonitor {
    /// Monitor registry activity on specified keys
    pub async fn monitor(
        running: Arc<AtomicBool>,
        monitor_keys: Vec<String>,
        event_tx: Sender<RegistryEvent>,
    ) {
        let mut handles = vec![];
        
        for key_path in monitor_keys {
            let run = running.clone();
            let tx = event_tx.clone();
            let k = key_path.clone();
            
            let handle = tokio::spawn(async move {
                Self::monitor_registry_key(run, k, tx).await;
            });
            
            handles.push(handle);
        }
        
        // Wait for all monitors
        for handle in handles {
            let _ = handle.await;
        }
    }
    
    /// Monitor a single registry key
    async fn monitor_registry_key(
        running: Arc<AtomicBool>,
        key_path: String,
        event_tx: Sender<RegistryEvent>,
    ) {
        use winapi::um::winreg::{RegOpenKeyExW, RegNotifyChangeKeyValue, RegCloseKey, HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER, KEY_NOTIFY, REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_CHANGE_LAST_SET};
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;
        
        // Parse key path (simplified - would properly parse HKEY_CURRENT_USER\..., etc.)
        let (hkey, subkey) = if key_path.starts_with("HKCU\\") {
            (HKEY_CURRENT_USER, &key_path[5..])
        } else if key_path.starts_with("HKLM\\") {
            (HKEY_LOCAL_MACHINE, &key_path[5..])
        } else {
            error!("Invalid registry key path: {}", key_path);
            return;
        };
        
        let subkey_wide: Vec<u16> = OsStr::new(subkey)
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        unsafe {
            let mut h_key = ptr::null_mut();
            
            let result = RegOpenKeyExW(
                hkey,
                subkey_wide.as_ptr(),
                0,
                KEY_NOTIFY,
                &mut h_key,
            );
            
            if result != 0 {
                error!("Failed to open registry key: {}", key_path);
                return;
            }
            
            while running.load(Ordering::Relaxed) {
                let notify_result = RegNotifyChangeKeyValue(
                    h_key,
                    1, // Watch subtree
                    REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_CHANGE_LAST_SET,
                    ptr::null_mut(),
                    0,
                );
                
                if notify_result == 0 {
                    // Change detected
                    let registry_event = RegistryEvent {
                        event_type: "registry_activity".to_string(),
                        key_path: key_path.clone(),
                        operation: "MODIFY".to_string(),
                        value_name: "unknown".to_string(),
                        process_id: winapi::um::processthreadsapi::GetCurrentProcessId(),
                        timestamp: Utc::now(),
                    };
                    
                    if event_tx.try_send(registry_event).is_err() {
                        warn!("Registry event queue full, dropping event");
                    }
                } else {
                    // Error or timeout
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
            
            RegCloseKey(h_key);
        }
    }
}

#[cfg(not(windows))]
pub struct RegistryActivityMonitor;

#[cfg(not(windows))]
impl RegistryActivityMonitor {
    pub async fn monitor(
        _running: std::sync::Arc<std::sync::atomic::AtomicBool>,
        _monitor_keys: Vec<String>,
        _event_tx: crossbeam_channel::Sender<crate::event::RegistryEvent>,
    ) {
        // Placeholder for non-Windows builds
    }
}
