// Path and File Name : /home/ransomeye/rebuild/edge/dpi/probe/src/l7_parser.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: L7 protocol parsers (SMB, DNS, HTTP, HTTPS, RDP) - metadata only, no decryption

use std::collections::HashMap;
use tracing::debug;

use super::errors::ProbeError;

/// L7 protocol metadata (passive, metadata-only, no decryption)
#[derive(Debug, Clone)]
pub struct L7Metadata {
    pub protocol: L7Protocol,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum L7Protocol {
    SMB,
    DNS,
    HTTP,
    HTTPS,
    RDP,
    Unknown,
}

pub struct L7Parser;

impl L7Parser {
    pub fn new() -> Self {
        Self
    }

    /// Parse L7 protocol from TCP/UDP payload
    /// Returns metadata only (no decryption)
    pub fn parse(&self, payload: &[u8], src_port: Option<u16>, dst_port: Option<u16>) -> Result<L7Metadata, ProbeError> {
        if payload.is_empty() {
            return Ok(L7Metadata {
                protocol: L7Protocol::Unknown,
                metadata: HashMap::new(),
            });
        }

        // Determine protocol by port
        let port = dst_port.or(src_port);
        
        match port {
            Some(53) => self.parse_dns(payload),
            Some(80) | Some(8080) => self.parse_http(payload),
            Some(443) | Some(8443) => self.parse_https(payload),
            Some(445) => self.parse_smb(payload),
            Some(3389) => self.parse_rdp(payload),
            _ => {
                // Try to detect by content
                if payload.len() >= 4 && &payload[0..4] == b"SMB" {
                    self.parse_smb(payload)
                } else if payload.len() >= 2 && payload[0] == 0x00 && payload[1] == 0x00 {
                    // DNS query/response
                    self.parse_dns(payload)
                } else if payload.len() >= 4 && payload[0] == 0x03 && payload[1] == 0x00 {
                    // RDP
                    self.parse_rdp(payload)
                } else if payload.starts_with(b"GET ") || payload.starts_with(b"POST ") || payload.starts_with(b"HTTP/") {
                    self.parse_http(payload)
                } else if payload.len() >= 5 && &payload[0..3] == b"\x16\x03" {
                    // TLS handshake
                    self.parse_https(payload)
                } else {
                    Ok(L7Metadata {
                        protocol: L7Protocol::Unknown,
                        metadata: HashMap::new(),
                    })
                }
            }
        }
    }

    /// Parse DNS protocol (metadata only)
    fn parse_dns(&self, payload: &[u8]) -> Result<L7Metadata, ProbeError> {
        let mut metadata = HashMap::new();
        
        if payload.len() < 12 {
            return Ok(L7Metadata {
                protocol: L7Protocol::DNS,
                metadata,
            });
        }

        // DNS header: ID (2), Flags (2), Questions (2), Answers (2), Authority (2), Additional (2)
        let flags = u16::from_be_bytes([payload[2], payload[3]]);
        let is_query = (flags & 0x8000) == 0;
        let is_response = !is_query;
        
        metadata.insert("dns_type".to_string(), if is_query { "query".to_string() } else { "response".to_string() });
        
        if is_query && payload.len() >= 13 {
            // Try to extract query name (simplified)
            let mut pos = 12;
            let mut qname = String::new();
            let mut label_len = payload[pos] as usize;
            
            while label_len > 0 && label_len < 64 && pos + label_len < payload.len() {
                if !qname.is_empty() {
                    qname.push('.');
                }
                qname.push_str(&String::from_utf8_lossy(&payload[pos+1..pos+1+label_len]));
                pos += label_len + 1;
                if pos < payload.len() {
                    label_len = payload[pos] as usize;
                } else {
                    break;
                }
            }
            
            if !qname.is_empty() {
                metadata.insert("dns_query".to_string(), qname);
            }
            
            // Query type (2 bytes after name)
            if pos + 2 < payload.len() {
                let qtype = u16::from_be_bytes([payload[pos], payload[pos+1]]);
                let qtype_str = match qtype {
                    1 => "A",
                    2 => "NS",
                    5 => "CNAME",
                    15 => "MX",
                    28 => "AAAA",
                    _ => "OTHER",
                };
                metadata.insert("dns_qtype".to_string(), qtype_str.to_string());
            }
        }
        
        if is_response && payload.len() >= 13 {
            // Response code (bits 0-3 of flags byte 1)
            let rcode = (flags & 0x0F) as u8;
            metadata.insert("dns_response_code".to_string(), rcode.to_string());
        }

        Ok(L7Metadata {
            protocol: L7Protocol::DNS,
            metadata,
        })
    }

    /// Parse HTTP protocol (metadata only)
    fn parse_http(&self, payload: &[u8]) -> Result<L7Metadata, ProbeError> {
        let mut metadata = HashMap::new();
        
        let payload_str = String::from_utf8_lossy(payload);
        let lines: Vec<&str> = payload_str.lines().take(20).collect();
        
        if lines.is_empty() {
            return Ok(L7Metadata {
                protocol: L7Protocol::HTTP,
                metadata,
            });
        }

        // Parse request line or status line
        let first_line = lines[0];
        if first_line.starts_with("GET ") || first_line.starts_with("POST ") || first_line.starts_with("PUT ") ||
           first_line.starts_with("DELETE ") || first_line.starts_with("HEAD ") || first_line.starts_with("OPTIONS ") {
            // HTTP Request
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() >= 1 {
                metadata.insert("http_method".to_string(), parts[0].to_string());
            }
            if parts.len() >= 2 {
                metadata.insert("http_path".to_string(), parts[1].to_string());
            }
        } else if first_line.starts_with("HTTP/") {
            // HTTP Response
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() >= 2 {
                metadata.insert("http_status".to_string(), parts[1].to_string());
            }
        }

        // Parse headers
        for line in lines.iter().skip(1) {
            if line.is_empty() {
                break;
            }
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos+1..].trim();
                
                match key.as_str() {
                    "host" => {
                        metadata.insert("http_host".to_string(), value.to_string());
                    }
                    "user-agent" => {
                        metadata.insert("http_user_agent".to_string(), value.to_string());
                    }
                    "content-type" => {
                        metadata.insert("http_content_type".to_string(), value.to_string());
                    }
                    _ => {}
                }
            }
        }

        Ok(L7Metadata {
            protocol: L7Protocol::HTTP,
            metadata,
        })
    }

    /// Parse HTTPS/TLS protocol (SNI, JA3 metadata only, no decryption)
    fn parse_https(&self, payload: &[u8]) -> Result<L7Metadata, ProbeError> {
        let mut metadata = HashMap::new();
        
        if payload.len() < 5 {
            return Ok(L7Metadata {
                protocol: L7Protocol::HTTPS,
                metadata,
            });
        }

        // TLS record header: ContentType (1), Version (2), Length (2)
        let content_type = payload[0];
        
        if content_type != 0x16 { // Handshake
            return Ok(L7Metadata {
                protocol: L7Protocol::HTTPS,
                metadata,
            });
        }

        if payload.len() < 43 {
            return Ok(L7Metadata {
                protocol: L7Protocol::HTTPS,
                metadata,
            });
        }

        // TLS version
        let tls_version = u16::from_be_bytes([payload[1], payload[2]]);
        let version_str = match tls_version {
            0x0301 => "TLS 1.0",
            0x0302 => "TLS 1.1",
            0x0303 => "TLS 1.2",
            0x0304 => "TLS 1.3",
            _ => "UNKNOWN",
        };
        metadata.insert("tls_version".to_string(), version_str.to_string());

        // Handshake message type (byte 5)
        if payload.len() > 5 && payload[5] == 0x01 { // ClientHello
            // Try to extract SNI (Server Name Indication)
            // SNI is in the extensions section of ClientHello
            let mut pos = 43; // Skip to extensions
            
            if payload.len() > pos + 2 {
                let ext_len = u16::from_be_bytes([payload[pos], payload[pos+1]]) as usize;
                pos += 2;
                
                // Search for SNI extension (type 0x0000)
                let mut ext_pos = pos;
                while ext_pos + 4 < payload.len() && ext_pos < pos + ext_len {
                    let ext_type = u16::from_be_bytes([payload[ext_pos], payload[ext_pos+1]]);
                    let ext_len = u16::from_be_bytes([payload[ext_pos+2], payload[ext_pos+3]]) as usize;
                    
                    if ext_type == 0x0000 && ext_pos + 4 + ext_len <= payload.len() {
                        // SNI extension found
                        let sni_data = &payload[ext_pos+4..ext_pos+4+ext_len];
                        if sni_data.len() > 3 {
                            let name_type = sni_data[0];
                            let name_len = u16::from_be_bytes([sni_data[1], sni_data[2]]) as usize;
                            if name_type == 0x00 && name_len > 0 && name_len < 256 && 3 + name_len <= sni_data.len() {
                                if let Ok(sni) = String::from_utf8(sni_data[3..3+name_len].to_vec()) {
                                    metadata.insert("tls_sni".to_string(), sni);
                                }
                            }
                        }
                    }
                    
                    ext_pos += 4 + ext_len;
                }
            }
        }

        Ok(L7Metadata {
            protocol: L7Protocol::HTTPS,
            metadata,
        })
    }

    /// Parse SMB protocol (metadata only)
    fn parse_smb(&self, payload: &[u8]) -> Result<L7Metadata, ProbeError> {
        let mut metadata = HashMap::new();
        
        if payload.len() < 4 {
            return Ok(L7Metadata {
                protocol: L7Protocol::SMB,
                metadata,
            });
        }

        // SMB signature: "SMB" or "\xFE\x53\x4D\x42"
        if &payload[0..4] == b"SMB" || (payload[0] == 0xFE && &payload[1..4] == b"SMB") {
            metadata.insert("smb_version".to_string(), "SMB".to_string());
            
            if payload.len() >= 9 {
                // SMB command (byte 4 for SMB1, byte 8 for SMB2)
                if payload[0] == 0xFE {
                    // SMB2
                    if payload.len() >= 12 {
                        let command = u16::from_le_bytes([payload[8], payload[9]]);
                        let cmd_str = match command {
                            0x0000 => "NEGOTIATE",
                            0x0001 => "SESSION_SETUP",
                            0x0002 => "LOGOFF",
                            0x0003 => "TREE_CONNECT",
                            0x0004 => "TREE_DISCONNECT",
                            0x0005 => "CREATE",
                            0x0006 => "CLOSE",
                            0x0008 => "READ",
                            0x0009 => "WRITE",
                            _ => "OTHER",
                        };
                        metadata.insert("smb_command".to_string(), cmd_str.to_string());
                    }
                } else {
                    // SMB1
                    let command = payload[4];
                    let cmd_str = match command {
                        0x72 => "NEGOTIATE",
                        0x73 => "SESSION_SETUP",
                        0x74 => "LOGOFF",
                        0x75 => "TREE_CONNECT",
                        0x71 => "TREE_DISCONNECT",
                        0xa2 => "CREATE",
                        0x04 => "CLOSE",
                        0x2e => "READ",
                        0x2f => "WRITE",
                        _ => "OTHER",
                    };
                    metadata.insert("smb_command".to_string(), cmd_str.to_string());
                }
            }
        }

        Ok(L7Metadata {
            protocol: L7Protocol::SMB,
            metadata,
        })
    }

    /// Parse RDP protocol (metadata only)
    fn parse_rdp(&self, payload: &[u8]) -> Result<L7Metadata, ProbeError> {
        let mut metadata = HashMap::new();
        
        if payload.len() < 4 {
            return Ok(L7Metadata {
                protocol: L7Protocol::RDP,
                metadata,
            });
        }

        // RDP TPKT header: 0x03 0x00
        if payload[0] == 0x03 && payload[1] == 0x00 {
            metadata.insert("rdp_version".to_string(), "RDP".to_string());
            
            if payload.len() >= 6 {
                // TPKT length
                let tpkt_len = u16::from_be_bytes([payload[2], payload[3]]) as usize;
                
                if payload.len() >= 11 && tpkt_len >= 11 {
                    // X.224 Connection Request/Confirm
                    let li = payload[5]; // Length indicator
                    let code = payload[6]; // Connection code
                    
                    match code {
                        0xE0 => metadata.insert("rdp_type".to_string(), "CONNECTION_REQUEST".to_string()),
                        0xD0 => metadata.insert("rdp_type".to_string(), "CONNECTION_CONFIRM".to_string()),
                        _ => {}
                    }
                }
            }
        }

        Ok(L7Metadata {
            protocol: L7Protocol::RDP,
            metadata,
        })
    }
}

