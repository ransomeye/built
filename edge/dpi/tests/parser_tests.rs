// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/tests/parser_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Protocol parser correctness tests

use ransomeye_dpi_probe::parser::{ProtocolParser, Protocol};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_parse_ethernet_packet() {
    let parser = ProtocolParser::new();
    
    // Create minimal Ethernet packet (14 bytes header + payload)
    let mut packet = vec![0u8; 64];
    // Ethernet header: dst MAC (6) + src MAC (6) + EtherType (2)
    packet[12] = 0x08; // IPv4 EtherType
    packet[13] = 0x00;
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let result = parser.parse(&packet, timestamp);
    assert!(result.is_ok());
    
    let parsed = result.unwrap();
    assert_eq!(parsed.protocol, Protocol::Ethernet);
}

#[test]
fn test_parse_packet_too_short() {
    let parser = ProtocolParser::new();
    
    let packet = vec![0u8; 10]; // Too short for Ethernet header
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let result = parser.parse(&packet, timestamp);
    assert!(result.is_err());
}

#[test]
fn test_parser_determinism() {
    let parser = ProtocolParser::new();
    
    let mut packet = vec![0u8; 64];
    packet[12] = 0x08;
    packet[13] = 0x00;
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let result1 = parser.parse(&packet, timestamp);
    let result2 = parser.parse(&packet, timestamp);
    
    assert_eq!(result1.is_ok(), result2.is_ok());
    if result1.is_ok() && result2.is_ok() {
        let p1 = result1.unwrap();
        let p2 = result2.unwrap();
        assert_eq!(p1.protocol, p2.protocol);
        assert_eq!(p1.payload_len, p2.payload_len);
    }
}

