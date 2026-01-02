// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/src/scanner.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Active scanner - CIDR discovery, host liveness, port enumeration, service fingerprinting, rate-limited

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use ipnetwork::IpNetwork;
use chrono::Utc;
use uuid::Uuid;
use tokio::time::{timeout, Duration};
use tracing::{error, warn, info, debug};

use crate::errors::ScannerError;
use crate::result::{ScanResult, ScannerMode, Asset, PortInfo, PortState, Service, ScanMetadata};
use crate::rate_limit::RateLimiter;
use crate::security::ScanResultSigner;

pub struct ActiveScanner {
    rate_limiter: Arc<RateLimiter>,
    signer: Arc<ScanResultSigner>,
    max_ports: usize,
    scan_timeout: Duration,
    cidrs: Vec<IpNetwork>,
}

impl ActiveScanner {
    /// Create a new active scanner from environment variables
    pub fn new() -> Result<Self, ScannerError> {
        // Get configuration from environment (fail-closed on missing/unsafe values)
        let cidrs_str = std::env::var("SCAN_CIDRS")
            .map_err(|_| ScannerError::InvalidConfiguration(
                "SCAN_CIDRS environment variable is required".to_string()
            ))?;
        
        let max_ports = std::env::var("MAX_PORTS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<usize>()
            .map_err(|e| ScannerError::InvalidConfiguration(
                format!("Invalid MAX_PORTS: {}", e)
            ))?;
        
        if max_ports == 0 || max_ports > 65535 {
            return Err(ScannerError::InvalidConfiguration(
                format!("MAX_PORTS must be between 1 and 65535, got {}", max_ports)
            ));
        }
        
        let max_rate = std::env::var("MAX_RATE")
            .unwrap_or_else(|_| "10.0".to_string())
            .parse::<f64>()
            .map_err(|e| ScannerError::InvalidConfiguration(
                format!("Invalid MAX_RATE: {}", e)
            ))?;
        
        if max_rate <= 0.0 {
            return Err(ScannerError::InvalidConfiguration(
                format!("MAX_RATE must be > 0, got {}", max_rate)
            ));
        }
        
        let scan_timeout_secs = std::env::var("SCAN_TIMEOUT")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .map_err(|e| ScannerError::InvalidConfiguration(
                format!("Invalid SCAN_TIMEOUT: {}", e)
            ))?;
        
        if scan_timeout_secs == 0 {
            return Err(ScannerError::InvalidConfiguration(
                "SCAN_TIMEOUT must be > 0".to_string()
            ));
        }
        
        // Parse CIDRs
        let cidrs: Result<Vec<IpNetwork>, _> = cidrs_str
            .split(',')
            .map(|s| s.trim().parse())
            .collect();
        
        let cidrs = cidrs.map_err(|e| ScannerError::InvalidCidr(
            format!("Failed to parse CIDR: {}", e)
        ))?;
        
        if cidrs.is_empty() {
            return Err(ScannerError::InvalidConfiguration(
                "At least one CIDR must be specified".to_string()
            ));
        }
        
        // Get signing key path
        let signing_key_path = std::env::var("RANSOMEYE_SCANNER_PRIVATE_KEY_PATH")
            .unwrap_or_else(|_| "/etc/ransomeye/keys/scanner_private_key.pem".to_string());
        
        let signer = Arc::new(ScanResultSigner::new(&signing_key_path)?);
        
        // Create rate limiter
        let max_concurrent = std::env::var("MAX_CONCURRENT_SCANS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<usize>()
            .map_err(|e| ScannerError::InvalidConfiguration(
                format!("Invalid MAX_CONCURRENT_SCANS: {}", e)
            ))?;
        
        let rate_limiter = Arc::new(RateLimiter::new(max_rate, max_concurrent));
        
        Ok(Self {
            rate_limiter,
            signer,
            max_ports,
            scan_timeout: Duration::from_secs(scan_timeout_secs),
            cidrs,
        })
    }
    
    /// Scan a CIDR range
    pub async fn scan_cidr(&self, cidr: &IpNetwork) -> Result<Vec<ScanResult>, ScannerError> {
        info!("Starting active scan of CIDR: {}", cidr);
        
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        
        // Enumerate hosts in CIDR (bounded)
        let hosts: Vec<IpAddr> = cidr.iter().take(10000).collect(); // Safety limit
        
        for host in hosts {
            // Rate limit: acquire token for host scan
            self.rate_limiter.acquire(1.0).await?;
            
            match self.scan_host(host).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    warn!("Failed to scan host {}: {}", host, e);
                    // Continue with next host
                }
            }
            
            self.rate_limiter.release();
        }
        
        let duration = start_time.elapsed();
        info!("Completed CIDR scan in {:?}, found {} hosts", duration, results.len());
        
        Ok(results)
    }
    
    /// Scan a single host
    async fn scan_host(&self, ip: IpAddr) -> Result<ScanResult, ScannerError> {
        debug!("Scanning host: {}", ip);
        
        // Check host liveness (ICMP ping or TCP SYN)
        let liveness_check = self.check_liveness(ip).await?;
        
        if !liveness_check {
            return Err(ScannerError::InternalError(
                format!("Host {} is not alive", ip)
            ));
        }
        
        // Enumerate ports (bounded by max_ports)
        let ports = self.enumerate_ports(ip).await?;
        
        // Fingerprint services (banner-based, no exploit)
        let services = self.fingerprint_services(ip, &ports).await?;
        
        // Build scan result
        let mut result = ScanResult {
            scan_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            scanner_mode: ScannerMode::Active,
            asset: Asset {
                ip: ip.to_string(),
                hostname: self.resolve_hostname(ip).await.ok(),
                mac: None, // MAC not available in active scan
                vendor: None,
            },
            open_ports: ports.iter().map(|p| PortInfo {
                port: *p,
                protocol: "tcp".to_string(),
                state: PortState::Open,
                discovered_at: Utc::now(),
            }).collect(),
            services,
            confidence_score: 0.8, // Default confidence for active scan
            hash: String::new(),
            signature: String::new(),
            metadata: Some(ScanMetadata {
                scan_duration_ms: 0,
                ports_scanned: ports.len(),
                hosts_scanned: 1,
                rate_limit_applied: true,
                cidr: Some(ip.to_string()),
            }),
        };
        
        // Sign result
        result = self.signer.sign_result(result)?;
        
        Ok(result)
    }
    
    /// Check host liveness (ICMP or TCP SYN)
    async fn check_liveness(&self, ip: IpAddr) -> Result<bool, ScannerError> {
        // Use TCP SYN to port 80 or 443 as liveness check (more reliable than ICMP)
        // This is a simplified implementation - production would use proper TCP SYN scan
        
        let check_timeout = Duration::from_secs(2);
        
        // Try TCP connection to common ports
        for port in [80, 443, 22, 3389] {
            match timeout(check_timeout, self.tcp_connect(ip, port)).await {
                Ok(Ok(_)) => return Ok(true),
                Ok(Err(_)) => continue,
                Err(_) => continue, // Timeout
            }
        }
        
        // If no TCP connection works, try ICMP ping (if available)
        // For now, return false if no TCP connection succeeded
        Ok(false)
    }
    
    /// TCP connect (simplified - production would use proper TCP SYN)
    async fn tcp_connect(&self, ip: IpAddr, port: u16) -> Result<(), ScannerError> {
        use tokio::net::TcpStream;
        
        let addr = format!("{}:{}", ip, port);
        match timeout(Duration::from_secs(1), TcpStream::connect(&addr)).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(ScannerError::InternalError(
                format!("TCP connect failed: {}", e)
            )),
            Err(_) => Err(ScannerError::ScanTimeout(
                format!("TCP connect timeout to {}:{}", ip, port)
            )),
        }
    }
    
    /// Enumerate ports (bounded by max_ports)
    async fn enumerate_ports(&self, ip: IpAddr) -> Result<Vec<u16>, ScannerError> {
        // Common ports to scan (bounded by max_ports)
        let common_ports = vec![
            22, 23, 25, 53, 80, 110, 143, 443, 445, 993, 995,
            1433, 3306, 3389, 5432, 5900, 8080, 8443,
        ];
        
        let ports_to_scan: Vec<u16> = common_ports
            .into_iter()
            .take(self.max_ports.min(common_ports.len()))
            .collect();
        
        let mut open_ports = Vec::new();
        
        for port in ports_to_scan {
            // Rate limit per port
            self.rate_limiter.acquire(0.1).await?;
            
            match timeout(Duration::from_secs(1), self.tcp_connect(ip, port)).await {
                Ok(Ok(_)) => {
                    open_ports.push(port);
                }
                _ => {
                    // Port is closed or filtered
                }
            }
            
            self.rate_limiter.release();
        }
        
        if open_ports.len() > self.max_ports {
            return Err(ScannerError::PortLimitExceeded(
                format!("Found {} open ports, exceeds MAX_PORTS={}", open_ports.len(), self.max_ports)
            ));
        }
        
        Ok(open_ports)
    }
    
    /// Fingerprint services (banner-based, no exploit)
    async fn fingerprint_services(&self, ip: IpAddr, ports: &[u16]) -> Result<Vec<Service>, ScannerError> {
        let mut services = Vec::new();
        
        for port in ports {
            // Rate limit per service fingerprint
            self.rate_limiter.acquire(0.5).await?;
            
            // Try to get banner (simplified - production would use proper banner grabbing)
            let banner = self.get_banner(ip, *port).await.ok();
            
            let service_name = self.infer_service(*port, &banner);
            let version = self.infer_version(&banner);
            
            services.push(Service {
                port: *port,
                protocol: "tcp".to_string(),
                service_name,
                version,
                banner,
                confidence: 0.7, // Default confidence for banner-based detection
            });
            
            self.rate_limiter.release();
        }
        
        Ok(services)
    }
    
    /// Get banner from service (banner-based, no exploit)
    async fn get_banner(&self, ip: IpAddr, port: u16) -> Result<String, ScannerError> {
        use tokio::net::TcpStream;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        
        let addr = format!("{}:{}", ip, port);
        let mut stream = timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await
            .map_err(|_| ScannerError::ScanTimeout(
                format!("Banner grab timeout for {}:{}", ip, port)
            ))?
            .map_err(|e| ScannerError::InternalError(
                format!("Banner grab connection failed: {}", e)
            ))?;
        
        // Send a simple probe (no exploit, just banner grab)
        let _ = stream.write(b"\n").await;
        
        // Read banner (limited to 1024 bytes for safety)
        let mut buffer = vec![0u8; 1024];
        match timeout(Duration::from_secs(1), stream.read(&mut buffer)).await {
            Ok(Ok(n)) if n > 0 => {
                let banner = String::from_utf8_lossy(&buffer[..n]).to_string();
                Ok(banner.trim().to_string())
            }
            _ => Err(ScannerError::InternalError(
                "Failed to read banner".to_string()
            )),
        }
    }
    
    /// Infer service name from port and banner
    fn infer_service(&self, port: u16, banner: &Option<String>) -> String {
        // Port-based inference
        let port_service = match port {
            22 => "ssh",
            23 => "telnet",
            25 => "smtp",
            53 => "dns",
            80 => "http",
            110 => "pop3",
            143 => "imap",
            443 => "https",
            445 => "smb",
            993 => "imaps",
            995 => "pop3s",
            1433 => "mssql",
            3306 => "mysql",
            3389 => "rdp",
            5432 => "postgresql",
            5900 => "vnc",
            8080 => "http-proxy",
            8443 => "https-alt",
            _ => "unknown",
        };
        
        // Banner-based refinement (if available)
        if let Some(ref b) = banner {
            let banner_lower = b.to_lowercase();
            if banner_lower.contains("ssh") {
                return "ssh".to_string();
            }
            if banner_lower.contains("http") {
                return "http".to_string();
            }
            if banner_lower.contains("apache") {
                return "apache".to_string();
            }
            if banner_lower.contains("nginx") {
                return "nginx".to_string();
            }
        }
        
        port_service.to_string()
    }
    
    /// Infer version from banner
    fn infer_version(&self, banner: &Option<String>) -> Option<String> {
        banner.as_ref().and_then(|b| {
            // Extract version patterns (simplified)
            if let Some(cap) = regex::Regex::new(r"(\d+\.\d+\.\d+)")
                .ok()
                .and_then(|re| re.captures(b))
            {
                return Some(cap.get(1).unwrap().as_str().to_string());
            }
            None
        })
    }
    
    /// Resolve hostname from IP
    async fn resolve_hostname(&self, ip: IpAddr) -> Result<String, ScannerError> {
        use dns_lookup::lookup_addr;
        
        match lookup_addr(&ip) {
            Ok(hostname) => Ok(hostname),
            Err(_) => Err(ScannerError::InternalError(
                format!("Failed to resolve hostname for {}", ip)
            )),
        }
    }
    
    /// Run full scan of all configured CIDRs
    pub async fn scan_all(&self) -> Result<Vec<ScanResult>, ScannerError> {
        let mut all_results = Vec::new();
        
        for cidr in &self.cidrs {
            match self.scan_cidr(cidr).await {
                Ok(mut results) => {
                    all_results.append(&mut results);
                }
                Err(e) => {
                    error!("Failed to scan CIDR {}: {}", cidr, e);
                    // Continue with next CIDR
                }
            }
        }
        
        Ok(all_results)
    }
}

