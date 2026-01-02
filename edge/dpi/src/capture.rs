// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/src/capture.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Packet capture engine - high-throughput packet capture with zero packet modification

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task;
use pcap::{Capture, Device, Active};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use tracing::{error, warn, debug, info};
use crossbeam_channel::{bounded, Receiver, Sender};
use crate::config::Config;
use crate::identity::Identity;
use crate::flow::FlowAssembler;
use crate::feature::FeatureExtractor;
use crate::signing::{EventSigner, SignedEvent};
use crate::transport::TransportClient;
use crate::backpressure::BackpressureHandler;
use crate::buffer::DiskBuffer;

pub struct CaptureEngine {
    config: Config,
    identity: Arc<Identity>,
    running: Arc<AtomicBool>,
    packet_tx: Option<Sender<Vec<u8>>>,
    packet_rx: Option<Receiver<Vec<u8>>>,
}

impl CaptureEngine {
    pub fn new(config: Config, identity: Arc<Identity>) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let (tx, rx) = bounded::<Vec<u8>>(10000);
        
        Ok(Arc::new(Self {
            config,
            identity,
            running: Arc::new(AtomicBool::new(false)),
            packet_tx: Some(tx),
            packet_rx: Some(rx),
        }))
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.running.store(true, Ordering::Relaxed);
        info!("Starting DPI Probe capture engine");
        
        // Initialize components
        let flow_assembler = Arc::new(FlowAssembler::new(self.config.flow_timeout_seconds));
        let backpressure = Arc::new(BackpressureHandler::new(
            self.config.max_buffer_size_mb * 1024 * 1024,
            self.config.backpressure_threshold,
        ));
        let signer = Arc::new(EventSigner::new(
            self.identity.keypair(),
            self.identity.producer_id().to_string(),
        ));
        let transport = Arc::new(TransportClient::new(self.config.clone(), backpressure.clone())?);
        let disk_buffer = Arc::new(DiskBuffer::new(&self.config.buffer_dir, self.config.max_buffer_size_mb)?);
        
        // Start packet capture task
        let capture_handle = {
            let config = self.config.clone();
            let running = self.running.clone();
            let packet_tx = self.packet_tx.take().unwrap();
            
            task::spawn(async move {
                Self::capture_loop(config, running, packet_tx).await;
            })
        };
        
        // Start processing loop
        let process_handle = {
            let running = self.running.clone();
            let packet_rx = self.packet_rx.take().unwrap();
            let flow_assembler = flow_assembler.clone();
            let signer = signer.clone();
            let transport = transport.clone();
            let backpressure = backpressure.clone();
            let disk_buffer = disk_buffer.clone();
            
            task::spawn(async move {
                Self::process_loop(
                    running,
                    packet_rx,
                    flow_assembler,
                    signer,
                    transport,
                    backpressure,
                    disk_buffer,
                ).await;
            })
        };
        
        // Start cleanup task
        let cleanup_handle = {
            let running = self.running.clone();
            let flow_assembler = flow_assembler.clone();
            
            task::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
                while running.load(Ordering::Relaxed) {
                    interval.tick().await;
                    flow_assembler.cleanup_stale_flows();
                }
            })
        };
        
        // Wait for tasks
        tokio::select! {
            _ = capture_handle => {
                error!("Capture task exited");
            }
            _ = process_handle => {
                error!("Process task exited");
            }
            _ = cleanup_handle => {
                error!("Cleanup task exited");
            }
        }
        
        Ok(())
    }
    
    async fn capture_loop(
        config: Config,
        running: Arc<AtomicBool>,
        packet_tx: Sender<Vec<u8>>,
    ) {
        // Open capture device
        let device = Device::list()
            .unwrap_or_default()
            .into_iter()
            .find(|d| d.name == config.capture_interface)
            .unwrap_or_else(|| {
                warn!("Interface {} not found, using default", config.capture_interface);
                Device::list().unwrap_or_default().into_iter().next().unwrap()
            });
        
        let mut cap = Capture::from_device(device)
            .unwrap()
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()
            .unwrap();
        
        info!("Capturing packets on interface: {}", config.capture_interface);
        
        while running.load(Ordering::Relaxed) {
            match cap.next_packet() {
                Ok(packet) => {
                    // Send packet to processing queue (non-blocking)
                    if packet_tx.try_send(packet.data.to_vec()).is_err() {
                        warn!("Packet queue full, dropping packet");
                    }
                }
                Err(pcap::Error::TimeoutExpired) => {
                    // Timeout is expected, continue
                    continue;
                }
                Err(e) => {
                    error!("Capture error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    async fn process_loop(
        running: Arc<AtomicBool>,
        packet_rx: Receiver<Vec<u8>>,
        flow_assembler: Arc<FlowAssembler>,
        signer: Arc<EventSigner>,
        transport: Arc<TransportClient>,
        backpressure: Arc<BackpressureHandler>,
        disk_buffer: Arc<DiskBuffer>,
    ) {
        while running.load(Ordering::Relaxed) {
            // Receive packet
            let packet_data = match packet_rx.recv() {
                Ok(data) => data,
                Err(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    continue;
                }
            };
            
            // Parse Ethernet frame
            let ethernet = match EthernetPacket::new(&packet_data) {
                Some(p) => p,
                None => continue,
            };
            
            // Extract IP and transport info
            let (src_ip, dst_ip, src_port, dst_port, protocol) = match ethernet.get_ethertype() {
                EtherTypes::Ipv4 => {
                    let ipv4 = match Ipv4Packet::new(ethernet.payload()) {
                        Some(p) => p,
                        None => continue,
                    };
                    
                    let (src_port, dst_port, protocol) = match ipv4.get_next_level_protocol() {
                        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                            let tcp = match TcpPacket::new(ipv4.payload()) {
                                Some(p) => (p.get_source(), p.get_destination(), 6u8),
                                None => continue,
                            };
                            tcp
                        }
                        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                            let udp = match UdpPacket::new(ipv4.payload()) {
                                Some(p) => (p.get_source(), p.get_destination(), 17u8),
                                None => continue,
                            };
                            udp
                        }
                        _ => continue,
                    };
                    
                    (
                        std::net::IpAddr::V4(ipv4.get_source()),
                        std::net::IpAddr::V4(ipv4.get_destination()),
                        src_port,
                        dst_port,
                        protocol,
                    )
                }
                EtherTypes::Ipv6 => {
                    let ipv6 = match Ipv6Packet::new(ethernet.payload()) {
                        Some(p) => p,
                        None => continue,
                    };
                    
                    let (src_port, dst_port, protocol) = match ipv6.get_next_header() {
                        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                            let tcp = match TcpPacket::new(ipv6.payload()) {
                                Some(p) => (p.get_source(), p.get_destination(), 6u8),
                                None => continue,
                            };
                            tcp
                        }
                        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                            let udp = match UdpPacket::new(ipv6.payload()) {
                                Some(p) => (p.get_source(), p.get_destination(), 17u8),
                                None => continue,
                            };
                            udp
                        }
                        _ => continue,
                    };
                    
                    (
                        std::net::IpAddr::V6(ipv6.get_source()),
                        std::net::IpAddr::V6(ipv6.get_destination()),
                        src_port,
                        dst_port,
                        protocol,
                    )
                }
                _ => continue,
            };
            
            // Process packet in flow assembler
            if let Some(flow) = flow_assembler.process_packet(
                src_ip,
                dst_ip,
                src_port,
                dst_port,
                protocol,
                packet_data.len(),
            ) {
                // Extract features
                let features = FeatureExtractor::extract(&flow);
                let telemetry_data = FeatureExtractor::to_telemetry_data(&features);
                
                // Sign event
                let signed_event = match signer.sign_event(telemetry_data) {
                    Ok(e) => e,
                    Err(e) => {
                        error!("Failed to sign event: {}", e);
                        continue;
                    }
                };
                
                // Send to Core (with retry on backpressure or buffer to disk if Core unavailable)
                let mut retry_count = 0;
                let max_retries = 3;
                loop {
                    match transport.send_event(&signed_event).await {
                        Ok(_) => {
                            // Successfully sent - try to flush any buffered events
                            Self::try_flush_buffer(&disk_buffer, &transport).await;
                            break;
                        }
                        Err(crate::transport::TransportError::Backpressure) => {
                            // Backpressure - buffer to disk and retry
                            if let Err(e) = disk_buffer.write_event(&signed_event) {
                                warn!("Failed to buffer event to disk: {}", e);
                                // Drop event if buffer full
                            }
                            retry_count += 1;
                            if retry_count >= max_retries {
                                // Give up after max retries, event is buffered
                                break;
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(100 * retry_count)).await;
                        }
                        Err(e) => {
                            // Core unavailable - buffer to disk
                            warn!("Core unavailable, buffering event to disk: {}", e);
                            if let Err(e) = disk_buffer.write_event(&signed_event) {
                                error!("Failed to buffer event to disk: {}", e);
                                // Drop event if buffer full
                            }
                            break;
                        }
                    }
                }
            }
            
            // Periodic cleanup handled by separate task
        }
    }
    
    async fn try_flush_buffer(disk_buffer: &DiskBuffer, transport: &TransportClient) {
        // Try to flush buffered events when Core is available
        let mut flushed = 0;
        while let Ok(Some(event)) = disk_buffer.read_oldest_event() {
            match transport.send_event(&event).await {
                Ok(_) => {
                    if let Err(e) = disk_buffer.remove_event(&event) {
                        warn!("Failed to remove buffered event: {}", e);
                    }
                    flushed += 1;
                    // Limit flush rate to avoid overwhelming Core
                    if flushed >= 10 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        flushed = 0;
                    }
                }
                Err(_) => {
                    // Core unavailable again, stop flushing
                    break;
                }
            }
        }
    }
    
    pub async fn shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
        info!("DPI Probe capture engine shutdown");
    }
}

