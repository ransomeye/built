// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/src/file_activity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: File activity monitoring using inotify - observes file open/write/delete (NO enforcement, NO blocking)

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use inotify::{Inotify, WatchMask, EventMask};
use chrono::Utc;
use tracing::{warn, debug, error};
use crossbeam_channel::Sender;
use crate::event::FileEvent;

/// Monitors file system activity using inotify
/// OBSERVATION ONLY - never blocks, never enforces, never modifies files
pub struct FileActivityMonitor;

impl FileActivityMonitor {
    /// Monitor file activity on specified paths
    pub async fn monitor(
        running: Arc<AtomicBool>,
        monitor_paths: Vec<PathBuf>,
        event_tx: Sender<FileEvent>,
    ) {
        let mut inotify = match Inotify::init() {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to initialize inotify: {}", e);
                return;
            }
        };
        
        // Add watches for all monitor paths
        for path in &monitor_paths {
            if path.exists() {
                match inotify.add_watch(
                    path,
                    WatchMask::CREATE
                        | WatchMask::MODIFY
                        | WatchMask::DELETE
                        | WatchMask::DELETE_SELF
                        | WatchMask::MOVE,
                ) {
                    Ok(_) => {
                        debug!("Watching path: {}", path.display());
                    }
                    Err(e) => {
                        warn!("Failed to watch path {}: {}", path.display(), e);
                    }
                }
            } else {
                warn!("Monitor path does not exist: {}", path.display());
            }
        }
        
        let mut buffer = [0u8; 4096];
        
        while running.load(Ordering::Relaxed) {
            // Read events (non-blocking)
            match inotify.read_events_blocking(&mut buffer) {
                Ok(events) => {
                    for event in events {
                        Self::process_inotify_event(event, &event_tx);
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        error!("inotify read error: {}", e);
                    }
                    // Small delay to avoid tight loop
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    /// Process an inotify event and send FileEvent
    fn process_inotify_event(event: inotify::Event<&PathBuf>, event_tx: &Sender<FileEvent>) {
        let operation = if event.mask.contains(EventMask::CREATE) {
            "CREATE"
        } else if event.mask.contains(EventMask::MODIFY) {
            "MODIFY"
        } else if event.mask.contains(EventMask::DELETE) || event.mask.contains(EventMask::DELETE_SELF) {
            "DELETE"
        } else if event.mask.contains(EventMask::MOVED_FROM) {
            "MOVE_FROM"
        } else if event.mask.contains(EventMask::MOVED_TO) {
            "MOVE_TO"
        } else {
            "UNKNOWN"
        };
        
        let path = event.name
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        // Get current user and process ID (for context, not for enforcement)
        let user_id = unsafe { libc::getuid() };
        let process_id = unsafe { libc::getpid() };
        
        let file_event = FileEvent {
            event_type: "file_activity".to_string(),
            path,
            operation: operation.to_string(),
            user_id,
            process_id,
            timestamp: Utc::now(),
        };
        
        if event_tx.try_send(file_event).is_err() {
            warn!("File event queue full, dropping event");
        }
    }
}
