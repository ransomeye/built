// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/tests/flow_eviction_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Flow eviction and bounded memory tests

use ransomeye_dpi_probe::flow::{FlowTracker, FlowKey};
use ransomeye_dpi_probe::parser::{ParsedPacket, Protocol};
use std::time::{SystemTime, UNIX_EPOCH};

fn create_test_packet(src_ip: &str, dst_ip: &str, src_port: u16, dst_port: u16) -> ParsedPacket {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    ParsedPacket {
        timestamp,
        src_mac: Some([0; 6]),
        dst_mac: Some([0; 6]),
        src_ip: Some(src_ip.to_string()),
        dst_ip: Some(dst_ip.to_string()),
        src_port: Some(src_port),
        dst_port: Some(dst_port),
        protocol: Protocol::TCP,
        payload_len: 100,
        is_fragment: false,
    }
}

#[test]
fn test_flow_tracker_bounded_memory() {
    let tracker = FlowTracker::new(100); // Max 100 flows
    
    // Add flows up to limit
    for i in 0..100 {
        let packet = create_test_packet(
            &format!("192.168.1.{}", i % 255),
            "10.0.0.1",
            1000 + i as u16,
            80,
        );
        
        let result = tracker.update_flow(&packet);
        assert!(result.is_ok());
    }
    
    assert_eq!(tracker.flow_count(), 100);
}

#[test]
fn test_flow_eviction() {
    let tracker = FlowTracker::new(100);
    
    // Fill beyond eviction threshold (90% = 90 flows)
    for i in 0..95 {
        let packet = create_test_packet(
            &format!("192.168.1.{}", i % 255),
            "10.0.0.1",
            1000 + i as u16,
            80,
        );
        
        tracker.update_flow(&packet).unwrap();
    }
    
    // Eviction should have occurred (target: 80% = 80 flows)
    let count = tracker.flow_count();
    assert!(count <= 80, "Flow count {} should be <= 80 after eviction", count);
}

#[test]
fn test_flow_key_creation() {
    let packet = create_test_packet("192.168.1.1", "10.0.0.1", 1234, 80);
    
    let key = FlowKey::from_packet(&packet);
    assert!(key.is_some());
    
    let k = key.unwrap();
    assert_eq!(k.src_ip, "192.168.1.1");
    assert_eq!(k.dst_ip, "10.0.0.1");
    assert_eq!(k.src_port, 1234);
    assert_eq!(k.dst_port, 80);
    assert_eq!(k.protocol, 6); // TCP
}

