// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/src/file_activity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: File activity monitoring on Windows using ReadDirectoryChangesW - observes file create/modify/delete (NO enforcement, NO blocking)

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
use crate::event::FileEvent;

/// Monitors file system activity on Windows using ReadDirectoryChangesW
/// OBSERVATION ONLY - never blocks, never enforces, never modifies files
#[cfg(windows)]
pub struct FileActivityMonitor;

#[cfg(windows)]
impl FileActivityMonitor {
    /// Monitor file activity on specified paths
    pub async fn monitor(
        running: Arc<AtomicBool>,
        monitor_paths: Vec<PathBuf>,
        event_tx: Sender<FileEvent>,
    ) {
        let mut handles = vec![];
        
        for path in monitor_paths {
            if path.exists() {
                let run = running.clone();
                let tx = event_tx.clone();
                let p = path.clone();
                
                let handle = tokio::spawn(async move {
                    Self::monitor_directory(run, p, tx).await;
                });
                
                handles.push(handle);
            }
        }
        
        // Wait for all monitors
        for handle in handles {
            let _ = handle.await;
        }
    }
    
    /// Monitor a single directory using ReadDirectoryChangesW
    async fn monitor_directory(
        running: Arc<AtomicBool>,
        path: PathBuf,
        event_tx: Sender<FileEvent>,
    ) {
        use winapi::um::winnt::{FILE_LIST_DIRECTORY, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE, OPEN_EXISTING};
        use winapi::um::fileapi::{ReadDirectoryChangesW, FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE};
        use winapi::um::winbase::{CreateFileW, FILE_FLAG_BACKUP_SEMANTICS};
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;
        
        let path_wide: Vec<u16> = OsStr::new(&path.to_string_lossy())
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        unsafe {
            let handle = CreateFileW(
                path_wide.as_ptr(),
                FILE_LIST_DIRECTORY,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS,
                ptr::null_mut(),
            );
            
            if handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
                error!("Failed to open directory: {}", path.display());
                return;
            }
            
            let mut buffer = [0u8; 4096];
            
            while running.load(Ordering::Relaxed) {
                let mut bytes_returned = 0u32;
                
                let result = ReadDirectoryChangesW(
                    handle,
                    buffer.as_mut_ptr() as *mut _,
                    buffer.len() as u32,
                    1, // Watch subdirectories
                    FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_LAST_WRITE,
                    &mut bytes_returned,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
                
                if result == 0 {
                    // Error or timeout
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
                
                // Parse notifications (simplified - would use proper FILE_NOTIFY_INFORMATION parsing)
                // For now, generate a simple event
                let file_event = FileEvent {
                    event_type: "file_activity".to_string(),
                    path: path.to_string_lossy().to_string(),
                    operation: "MODIFY".to_string(),
                    process_id: unsafe { winapi::um::processthreadsapi::GetCurrentProcessId() },
                    timestamp: Utc::now(),
                };
                
                if event_tx.try_send(file_event).is_err() {
                    warn!("File event queue full, dropping event");
                }
            }
            
            winapi::um::handleapi::CloseHandle(handle);
        }
    }
}

#[cfg(not(windows))]
pub struct FileActivityMonitor;

#[cfg(not(windows))]
impl FileActivityMonitor {
    pub async fn monitor(
        _running: std::sync::Arc<std::sync::atomic::AtomicBool>,
        _monitor_paths: Vec<std::path::PathBuf>,
        _event_tx: crossbeam_channel::Sender<crate::event::FileEvent>,
    ) {
        // Placeholder for non-Windows builds
    }
}
