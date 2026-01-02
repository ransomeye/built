// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/src/buffer.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Disk-based buffer for telemetry when Core is unavailable - bounded buffer with oldest-first drop policy

use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Write, Read};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tracing::{error, warn, debug, info};
use crate::signing::SignedEvent;

#[derive(Debug, Error)]
pub enum BufferError {
    #[error("Buffer directory error: {0}")]
    DirectoryError(String),
    #[error("File I/O error: {0}")]
    IoError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Buffer full - oldest events dropped")]
    BufferFull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BufferedEvent {
    timestamp: u64,
    sequence: u64,
    event: SignedEvent,
}

pub struct DiskBuffer {
    buffer_dir: PathBuf,
    max_size_bytes: usize,
    current_size: Arc<AtomicUsize>,
    sequence: Arc<AtomicUsize>,
}

impl DiskBuffer {
    pub fn new(buffer_dir: impl AsRef<Path>, max_size_mb: usize) -> Result<Self, BufferError> {
        let buffer_dir = buffer_dir.as_ref().to_path_buf();
        
        // Create buffer directory
        fs::create_dir_all(&buffer_dir)
            .map_err(|e| BufferError::DirectoryError(format!("Failed to create buffer directory: {}", e)))?;
        
        // Calculate current size
        let current_size = Self::calculate_directory_size(&buffer_dir)?;
        
        info!("Disk buffer initialized at {} (current size: {} bytes, max: {} MB)", 
              buffer_dir.display(), current_size, max_size_mb);
        
        Ok(Self {
            buffer_dir,
            max_size_bytes: max_size_mb * 1024 * 1024,
            current_size: Arc::new(AtomicUsize::new(current_size)),
            sequence: Arc::new(AtomicUsize::new(0)),
        })
    }
    
    pub fn write_event(&self, event: &SignedEvent) -> Result<(), BufferError> {
        // Check if buffer is full
        let current = self.current_size.load(Ordering::Relaxed);
        if current >= self.max_size_bytes {
            // Drop oldest events to make space
            self.drop_oldest_events(1024 * 1024)?; // Drop at least 1MB
        }
        
        // Generate filename from timestamp and sequence
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let filename = format!("{:020}_{:010}.event", timestamp, sequence);
        let filepath = self.buffer_dir.join(&filename);
        
        // Serialize event
        let buffered = BufferedEvent {
            timestamp,
            sequence,
            event: event.clone(),
        };
        
        let data = serde_json::to_vec(&buffered)
            .map_err(|e| BufferError::SerializationError(format!("Failed to serialize event: {}", e)))?;
        
        // Write to disk
        let mut file = fs::File::create(&filepath)
            .map_err(|e| BufferError::IoError(format!("Failed to create buffer file: {}", e)))?;
        
        file.write_all(&data)
            .map_err(|e| BufferError::IoError(format!("Failed to write buffer file: {}", e)))?;
        
        file.sync_all()
            .map_err(|e| BufferError::IoError(format!("Failed to sync buffer file: {}", e)))?;
        
        // Update size
        let data_size = data.len();
        self.current_size.fetch_add(data_size, Ordering::Relaxed);
        
        debug!("Buffered event to disk: {} ({} bytes)", filename, data_size);
        Ok(())
    }
    
    pub fn read_oldest_event(&self) -> Result<Option<SignedEvent>, BufferError> {
        // List all event files sorted by name (which includes timestamp)
        let mut entries: Vec<PathBuf> = fs::read_dir(&self.buffer_dir)
            .map_err(|e| BufferError::DirectoryError(format!("Failed to read buffer directory: {}", e)))?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.extension() == Some(std::ffi::OsStr::new("event")) {
                        Some(path)
                    } else {
                        None
                    }
                })
            })
            .collect();
        
        if entries.is_empty() {
            return Ok(None);
        }
        
        // Sort by filename (timestamp-first, then sequence)
        entries.sort();
        
        // Read oldest file
        let oldest_path = &entries[0];
        let mut file = fs::File::open(oldest_path)
            .map_err(|e| BufferError::IoError(format!("Failed to open buffer file: {}", e)))?;
        
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| BufferError::IoError(format!("Failed to read buffer file: {}", e)))?;
        
        // Deserialize
        let buffered: BufferedEvent = serde_json::from_slice(&data)
            .map_err(|e| BufferError::SerializationError(format!("Failed to deserialize event: {}", e)))?;
        
        Ok(Some(buffered.event))
    }
    
    pub fn remove_event(&self, event: &SignedEvent) -> Result<(), BufferError> {
        // Find and remove the event file
        let entries: Vec<PathBuf> = fs::read_dir(&self.buffer_dir)
            .map_err(|e| BufferError::DirectoryError(format!("Failed to read buffer directory: {}", e)))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| p.extension() == Some(std::ffi::OsStr::new("event")))
            .collect();
        
        for entry in entries {
            let mut file = match fs::File::open(&entry) {
                Ok(f) => f,
                Err(_) => continue,
            };
            
            let mut data = Vec::new();
            if file.read_to_end(&mut data).is_err() {
                continue;
            }
            
            let buffered: BufferedEvent = match serde_json::from_slice(&data) {
                Ok(b) => b,
                Err(_) => continue,
            };
            
            // Check if this is the event we're looking for
            if buffered.event.message_id == event.message_id {
                // Get file size before deletion
                let file_size = data.len();
                
                // Delete file
                fs::remove_file(&entry)
                    .map_err(|e| BufferError::IoError(format!("Failed to remove buffer file: {}", e)))?;
                
                // Update size
                self.current_size.fetch_sub(file_size, Ordering::Relaxed);
                
                debug!("Removed buffered event: {}", entry.display());
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    pub fn drop_oldest_events(&self, target_bytes_to_free: usize) -> Result<usize, BufferError> {
        let mut bytes_freed = 0;
        let mut events_dropped = 0;
        
        // List and sort all event files
        let mut entries: Vec<PathBuf> = fs::read_dir(&self.buffer_dir)
            .map_err(|e| BufferError::DirectoryError(format!("Failed to read buffer directory: {}", e)))?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.extension() == Some(std::ffi::OsStr::new("event")) {
                        Some(path)
                    } else {
                        None
                    }
                })
            })
            .collect();
        
        entries.sort();
        
        // Delete oldest files until we've freed enough space
        for entry in entries {
            if bytes_freed >= target_bytes_to_free {
                break;
            }
            
            // Get file size
            let metadata = fs::metadata(&entry)
                .map_err(|e| BufferError::IoError(format!("Failed to get file metadata: {}", e)))?;
            let file_size = metadata.len() as usize;
            
            // Delete file
            fs::remove_file(&entry)
                .map_err(|e| BufferError::IoError(format!("Failed to remove buffer file: {}", e)))?;
            
            bytes_freed += file_size;
            events_dropped += 1;
            
            self.current_size.fetch_sub(file_size, Ordering::Relaxed);
        }
        
        if events_dropped > 0 {
            warn!("Dropped {} oldest events ({} bytes) due to buffer full", events_dropped, bytes_freed);
        }
        
        Ok(bytes_freed)
    }
    
    pub fn get_current_size(&self) -> usize {
        self.current_size.load(Ordering::Relaxed)
    }
    
    pub fn get_max_size(&self) -> usize {
        self.max_size_bytes
    }
    
    pub fn count_buffered_events(&self) -> Result<usize, BufferError> {
        let count = fs::read_dir(&self.buffer_dir)
            .map_err(|e| BufferError::DirectoryError(format!("Failed to read buffer directory: {}", e)))?
            .filter(|entry| {
                entry.as_ref()
                    .map(|e| e.path().extension() == Some(std::ffi::OsStr::new("event")))
                    .unwrap_or(false)
            })
            .count();
        
        Ok(count)
    }
    
    fn calculate_directory_size(dir: &Path) -> Result<usize, BufferError> {
        let mut total = 0;
        
        let entries = fs::read_dir(dir)
            .map_err(|e| BufferError::DirectoryError(format!("Failed to read directory: {}", e)))?;
        
        for entry in entries {
            let entry = entry
                .map_err(|e| BufferError::IoError(format!("Failed to read directory entry: {}", e)))?;
            
            let metadata = entry.metadata()
                .map_err(|e| BufferError::IoError(format!("Failed to get metadata: {}", e)))?;
            
            if metadata.is_file() {
                total += metadata.len() as usize;
            }
        }
        
        Ok(total)
    }
}
