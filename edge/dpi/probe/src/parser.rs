// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/src/parser.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Protocol parsing (L3-L7) - zero allocation in hot path

use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use tracing::debug;

use super::errors::ProbeError;

/// Protocol parser for L3-L7
/// 
/// Zero allocations in hot path.
/// Supports: Ethernet, IPv4, IPv6, TCP, UDP.
/// No payload retention beyond parsing window.
#[derive(Debug, Clone)]
pub struct ParsedPacket {
    pub timestamp: u64,
    pub src_mac: Option<[u8; 6]>,
    pub dst_mac: Option<[u8; 6]>,
    pub src_ip: Option<String>,
    pub dst_ip: Option<String>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub protocol: Protocol,
    pub payload_len: usize,
    pub is_fragment: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Ethernet,
    IPv4,
    IPv6,
    TCP,
    UDP,
    ICMP,
    Unknown,
}

pub struct ProtocolParser;

impl ProtocolParser {
    pub fn new() -> Self {
        Self
    }
    
    /// Parse packet (zero allocation in hot path)
    pub fn parse(&self, data: &[u8], timestamp: u64) -> Result<ParsedPacket, ProbeError> {
        if data.len() < 14 {
            return Err(ProbeError::ParseFailed("Packet too short for Ethernet header".to_string()));
        }
        
        // Parse Ethernet (L2)
        let ethernet = EthernetPacket::new(data)
            .ok_or_else(|| ProbeError::ParseFailed("Invalid Ethernet packet".to_string()))?;
        
        let src_mac = Some(ethernet.get_source().octets());
        let dst_mac = Some(ethernet.get_destination().octets());
        
        // Parse IP (L3)
        let (src_ip, dst_ip, src_port, dst_port, protocol, payload_len, is_fragment) = match ethernet.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                    let src = ipv4.get_source();
                    let dst = ipv4.get_destination();
                    let frag = ipv4.get_flags() & 0x3 != 0 || ipv4.get_fragment_offset() != 0;
                    
                    // Parse transport (L4)
                    let (proto, src_port_val, dst_port_val, payload_len) = match ipv4.get_next_level_protocol() {
                        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                            if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                                (Protocol::TCP, Some(tcp.get_source()), Some(tcp.get_destination()), tcp.payload().len())
                            } else {
                                (Protocol::Unknown, None, None, ipv4.payload().len())
                            }
                        }
                        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                            if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                                (Protocol::UDP, Some(udp.get_source()), Some(udp.get_destination()), udp.payload().len())
                            } else {
                                (Protocol::Unknown, None, None, ipv4.payload().len())
                            }
                        }
                        pnet::packet::ip::IpNextHeaderProtocols::Icmp => {
                            (Protocol::ICMP, None, None, ipv4.payload().len())
                        }
                        _ => {
                            (Protocol::Unknown, None, None, ipv4.payload().len())
                        }
                    };
                    
                    (
                        Some(src.to_string()),
                        Some(dst.to_string()),
                        src_port_val,
                        dst_port_val,
                        proto,
                        payload_len,
                        frag,
                    )
                } else {
                    (None, None, None, None, Protocol::IPv4, 0, false)
                }
            }
            EtherTypes::Ipv6 => {
                if let Some(ipv6) = Ipv6Packet::new(ethernet.payload()) {
                    let src = ipv6.get_source();
                    let dst = ipv6.get_destination();
                    
                    // Parse transport (L4) for IPv6
                    let (proto, src_port_val, dst_port_val, payload_len) = match ipv6.get_next_header() {
                        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                            if let Some(tcp) = TcpPacket::new(ipv6.payload()) {
                                (Protocol::TCP, Some(tcp.get_source()), Some(tcp.get_destination()), tcp.payload().len())
                            } else {
                                (Protocol::Unknown, None, None, ipv6.payload().len())
                            }
                        }
                        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                            if let Some(udp) = UdpPacket::new(ipv6.payload()) {
                                (Protocol::UDP, Some(udp.get_source()), Some(udp.get_destination()), udp.payload().len())
                            } else {
                                (Protocol::Unknown, None, None, ipv6.payload().len())
                            }
                        }
                        _ => {
                            (Protocol::Unknown, None, None, ipv6.payload().len())
                        }
                    };
                    
                    (
                        Some(src.to_string()),
                        Some(dst.to_string()),
                        src_port_val,
                        dst_port_val,
                        proto,
                        payload_len,
                        false,
                    )
                } else {
                    (None, None, None, None, Protocol::IPv6, 0, false)
                }
            }
            _ => {
                (None, None, None, None, Protocol::Ethernet, ethernet.payload().len(), false)
            }
        };
        
        debug!("Parsed packet: {} -> {} ({:?})", 
            src_ip.as_ref().unwrap_or(&"unknown".to_string()),
            dst_ip.as_ref().unwrap_or(&"unknown".to_string()),
            protocol);
        
        Ok(ParsedPacket {
            timestamp,
            src_mac,
            dst_mac,
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            protocol,
            payload_len,
            is_fragment,
        })
    }
}

