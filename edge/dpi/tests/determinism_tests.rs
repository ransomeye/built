// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/tests/determinism_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Determinism tests for DPI Probe

use ransomeye_dpi_probe::parser::ProtocolParser;
use ransomeye_dpi_probe::extraction::FeatureExtractor;
use ransomeye_dpi_probe::parser::{ParsedPacket, Protocol};
use std::time::{SystemTime, UNIX_EPOCH};

fn create_test_packet() -> ParsedPacket {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    ParsedPacket {
        timestamp,
        src_mac: Some([0; 6]),
        dst_mac: Some([0; 6]),
        src_ip: Some("192.168.1.1".to_string()),
        dst_ip: Some("10.0.0.1".to_string()),
        src_port: Some(1234),
        dst_port: Some(80),
        protocol: Protocol::TCP,
        payload_len: 100,
        is_fragment: false,
    }
}

#[test]
fn test_parser_determinism() {
    let parser = ProtocolParser::new();
    let packet = create_test_packet();
    
    let result1 = parser.parse(&vec![0u8; 64], packet.timestamp);
    let result2 = parser.parse(&vec![0u8; 64], packet.timestamp);
    
    assert_eq!(result1.is_ok(), result2.is_ok());
    if result1.is_ok() && result2.is_ok() {
        let p1 = result1.unwrap();
        let p2 = result2.unwrap();
        assert_eq!(p1.protocol, p2.protocol);
    }
}

#[test]
fn test_feature_extraction_determinism() {
    let extractor = FeatureExtractor::new();
    let packet = create_test_packet();
    
    let result1 = extractor.extract(&packet, None);
    let result2 = extractor.extract(&packet, None);
    
    assert_eq!(result1.is_ok(), result2.is_ok());
    if result1.is_ok() && result2.is_ok() {
        let f1 = result1.unwrap();
        let f2 = result2.unwrap();
        assert_eq!(f1.packet_size, f2.packet_size);
        assert_eq!(f1.protocol, f2.protocol);
        assert_eq!(f1.is_fragment, f2.is_fragment);
    }
}

