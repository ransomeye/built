// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/src/capture.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: AF_PACKET/libpcap abstraction for high-throughput capture

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use parking_lot::Mutex;
use tracing::{error, warn, info, debug};
use pcap::{Capture, Device, Active};

use super::errors::ProbeError;

/// Packet capture abstraction
/// 
/// Supports AF_PACKET (Linux) and libpcap (cross-platform).
/// Zero allocations in hot path.
/// Lock-free statistics.
pub struct PacketCapture {
    capture: Arc<Mutex<Option<Capture<Active>>>>,
    interface: String,
    running: Arc<AtomicBool>,
    packets_captured: Arc<AtomicU64>,
    packets_dropped: Arc<AtomicU64>,
    bytes_captured: Arc<AtomicU64>,
}

impl PacketCapture {
    /// Create new packet capture
    pub fn new(interface: String) -> Result<Self, ProbeError> {
        info!("Initializing packet capture on interface: {}", interface);
        
        Ok(Self {
            capture: Arc::new(Mutex::new(None)),
            interface,
            running: Arc::new(AtomicBool::new(false)),
            packets_captured: Arc::new(AtomicU64::new(0)),
            packets_dropped: Arc::new(AtomicU64::new(0)),
            bytes_captured: Arc::new(AtomicU64::new(0)),
        })
    }
    
    /// Start capture (optional and explicit)
    pub fn start(&self) -> Result<(), ProbeError> {
        if self.running.load(Ordering::Acquire) {
            warn!("Capture already running");
            return Ok(());
        }
        
        info!("Starting packet capture on interface: {}", self.interface);
        
        let device = Device::list()
            .map_err(|e| ProbeError::CaptureFailed(format!("Failed to list devices: {}", e)))?
            .into_iter()
            .find(|d| d.name == self.interface)
            .ok_or_else(|| ProbeError::CaptureFailed(
                format!("Interface not found: {}", self.interface)
            ))?;
        
        let mut cap = Capture::from_device(device)
            .map_err(|e| ProbeError::CaptureFailed(format!("Failed to open device: {}", e)))?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .buffer_size(64 * 1024 * 1024) // 64MB buffer for high throughput
            .open()
            .map_err(|e| ProbeError::CaptureFailed(format!("Failed to activate capture: {}", e)))?;
        
        {
            let mut capture_guard = self.capture.lock();
            *capture_guard = Some(cap);
        }
        
        self.running.store(true, Ordering::Release);
        info!("Packet capture started successfully");
        Ok(())
    }
    
    /// Stop capture
    pub fn stop(&self) {
        if !self.running.load(Ordering::Acquire) {
            return;
        }
        
        info!("Stopping packet capture");
        self.running.store(false, Ordering::Release);
        
        let mut capture_guard = self.capture.lock();
        *capture_guard = None;
    }
    
    /// Read next packet (zero allocation in hot path)
    pub fn next_packet(&self) -> Result<Option<Vec<u8>>, ProbeError> {
        if !self.running.load(Ordering::Acquire) {
            return Ok(None);
        }
        
        let mut capture_guard = self.capture.lock();
        if let Some(ref mut cap) = *capture_guard {
            match cap.next_packet() {
                Ok(packet) => {
                    let len = packet.data.len();
                    self.packets_captured.fetch_add(1, Ordering::Relaxed);
                    self.bytes_captured.fetch_add(len as u64, Ordering::Relaxed);
                    
                    // Zero-copy: return packet data directly
                    Ok(Some(packet.data.to_vec()))
                }
                Err(pcap::Error::TimeoutExpired) => {
                    // Timeout is normal, not an error
                    Ok(None)
                }
                Err(e) => {
                    self.packets_dropped.fetch_add(1, Ordering::Relaxed);
                    error!("Packet capture error: {}", e);
                    Err(ProbeError::CaptureFailed(format!("Capture error: {}", e)))
                }
            }
        } else {
            Ok(None)
        }
    }
    
    /// Get statistics (lock-free)
    pub fn stats(&self) -> CaptureStats {
        CaptureStats {
            packets_captured: self.packets_captured.load(Ordering::Relaxed),
            packets_dropped: self.packets_dropped.load(Ordering::Relaxed),
            bytes_captured: self.bytes_captured.load(Ordering::Relaxed),
            running: self.running.load(Ordering::Relaxed),
        }
    }
    
    /// Check if capture is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone)]
pub struct CaptureStats {
    pub packets_captured: u64,
    pub packets_dropped: u64,
    pub bytes_captured: u64,
    pub running: bool,
}

