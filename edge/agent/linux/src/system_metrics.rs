// Path and File Name : /home/ransomeye/rebuild/edge/agent/linux/src/system_metrics.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: System metrics collection for Linux Agent health monitoring

use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;

/// System metrics structure matching dashboard API expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub filesystem: FilesystemMetrics,
    pub network: NetworkMetrics,
    pub system_state: SystemStateMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub utilization: Option<f64>,
    pub load_avg_1m: Option<f64>,
    pub load_avg_5m: Option<f64>,
    pub load_avg_15m: Option<f64>,
    pub core_count: Option<u32>,
    pub agent_process_cpu: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total: Option<u64>,
    pub used: Option<u64>,
    pub free: Option<u64>,
    pub swap_used: Option<u64>,
    pub agent_process_rss: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub read_bytes: Option<u64>,
    pub write_bytes: Option<u64>,
    pub read_iops: Option<u64>,
    pub write_iops: Option<u64>,
    pub utilization: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemMetrics {
    pub mounts: Vec<MountMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountMetrics {
    pub mount_point: String,
    pub total_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
    pub usage_percent: Option<f64>,
    pub inode_total: Option<u64>,
    pub inode_used: Option<u64>,
    pub inode_free: Option<u64>,
    pub inode_usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub bytes_in: Option<u64>,
    pub bytes_out: Option<u64>,
    pub packets_in: Option<u64>,
    pub packets_out: Option<u64>,
    pub errors: Option<u64>,
    pub drops: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStateMetrics {
    pub host_uptime: Option<u64>,
    pub process_count: Option<u32>,
    pub agent_process_status: Option<String>,
}

pub struct SystemMetricsCollector {
    pid: u32,
    start_time: u64,
    last_cpu_time: Option<CpuTime>,
    last_disk_stats: Option<DiskStats>,
    last_disk_time: u64,
    last_network_stats: Option<NetworkStats>,
    last_network_time: u64,
}

#[derive(Clone)]
struct CpuTime {
    total: u64,
    idle: u64,
}

#[derive(Clone)]
struct DiskStats {
    read_bytes: u64,
    write_bytes: u64,
    read_ios: u64,
    write_ios: u64,
}

#[derive(Clone)]
struct NetworkStats {
    bytes_in: u64,
    bytes_out: u64,
    packets_in: u64,
    packets_out: u64,
    drops: u64,
    errors: u64,
}

impl SystemMetricsCollector {
    pub fn new() -> Self {
        let pid = std::process::id();
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            pid,
            start_time,
            last_cpu_time: None,
            last_disk_stats: None,
            last_disk_time: 0,
            last_network_stats: None,
            last_network_time: 0,
        }
    }
    
    pub fn collect(&mut self) -> SystemMetrics {
        SystemMetrics {
            cpu: self.collect_cpu(),
            memory: self.collect_memory(),
            disk: self.collect_disk(),
            filesystem: self.collect_filesystem(),
            network: self.collect_network(),
            system_state: self.collect_system_state(),
        }
    }
    
    fn collect_cpu(&mut self) -> CpuMetrics {
        let mut metrics = CpuMetrics {
            utilization: None,
            load_avg_1m: None,
            load_avg_5m: None,
            load_avg_15m: None,
            core_count: None,
            agent_process_cpu: None,
        };
        
        // Read load average from /proc/loadavg
        if let Ok(loadavg) = fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = loadavg.split_whitespace().collect();
            if parts.len() >= 3 {
                metrics.load_avg_1m = parts[0].parse().ok();
                metrics.load_avg_5m = parts[1].parse().ok();
                metrics.load_avg_15m = parts[2].parse().ok();
            }
        }
        
        // Read CPU stats from /proc/stat
        if let Ok(stat) = fs::read_to_string("/proc/stat") {
            if let Some(first_line) = stat.lines().next() {
                if first_line.starts_with("cpu ") {
                    let parts: Vec<&str> = first_line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        let user: u64 = parts[1].parse().unwrap_or(0);
                        let nice: u64 = parts[2].parse().unwrap_or(0);
                        let system: u64 = parts[3].parse().unwrap_or(0);
                        let idle: u64 = parts[4].parse().unwrap_or(0);
                        let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
                        let irq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
                        let softirq: u64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
                        
                        let total = user + nice + system + idle + iowait + irq + softirq;
                        let cpu_time = CpuTime { total, idle };
                        
                        if let Some(ref last) = self.last_cpu_time {
                            let total_diff = cpu_time.total.saturating_sub(last.total);
                            let idle_diff = cpu_time.idle.saturating_sub(last.idle);
                            
                            if total_diff > 0 {
                                let used = total_diff.saturating_sub(idle_diff);
                                metrics.utilization = Some((used as f64 / total_diff as f64) * 100.0);
                            }
                        }
                        
                        self.last_cpu_time = Some(cpu_time);
                    }
                }
            }
        }
        
        // Count CPU cores from /proc/cpuinfo
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            let core_count = cpuinfo.lines()
                .filter(|line| line.starts_with("processor"))
                .count() as u32;
            if core_count > 0 {
                metrics.core_count = Some(core_count);
            }
        }
        
        // Get agent process CPU from /proc/{pid}/stat
        let proc_stat_path = format!("/proc/{}/stat", self.pid);
        if let Ok(stat_line) = fs::read_to_string(&proc_stat_path) {
            let parts: Vec<&str> = stat_line.split_whitespace().collect();
            if parts.len() >= 15 {
                let utime: u64 = parts[13].parse().unwrap_or(0);
                let stime: u64 = parts[14].parse().unwrap_or(0);
                let total_time = utime + stime;
                
                // Get system uptime to calculate percentage
                if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
                    if let Some(uptime_secs) = uptime_str.split_whitespace().next() {
                        if let Ok(uptime) = uptime_secs.parse::<f64>() {
                            if uptime > 0.0 {
                                let clock_ticks = total_time as f64 / 100.0; // Assuming 100 Hz clock
                                metrics.agent_process_cpu = Some((clock_ticks / uptime) * 100.0);
                            }
                        }
                    }
                }
            }
        }
        
        metrics
    }
    
    fn collect_memory(&self) -> MemoryMetrics {
        let mut metrics = MemoryMetrics {
            total: None,
            used: None,
            free: None,
            swap_used: None,
            agent_process_rss: None,
        };
        
        // Read memory info from /proc/meminfo
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            let mut mem_total = None;
            let mut mem_free = None;
            let mut mem_available = None;
            let mut swap_total = None;
            let mut swap_free = None;
            
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        mem_total = val.parse::<u64>().ok().map(|kb| kb * 1024);
                    }
                } else if line.starts_with("MemFree:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        mem_free = val.parse::<u64>().ok().map(|kb| kb * 1024);
                    }
                } else if line.starts_with("MemAvailable:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        mem_available = val.parse::<u64>().ok().map(|kb| kb * 1024);
                    }
                } else if line.starts_with("SwapTotal:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        swap_total = val.parse::<u64>().ok().map(|kb| kb * 1024);
                    }
                } else if line.starts_with("SwapFree:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        swap_free = val.parse::<u64>().ok().map(|kb| kb * 1024);
                    }
                }
            }
            
            metrics.total = mem_total;
            metrics.free = mem_free.or(mem_available);
            if let (Some(total), Some(free)) = (mem_total, metrics.free) {
                metrics.used = Some(total.saturating_sub(free));
            }
            
            if let (Some(swap_t), Some(swap_f)) = (swap_total, swap_free) {
                metrics.swap_used = Some(swap_t.saturating_sub(swap_f));
            }
        }
        
        // Get agent process RSS from /proc/{pid}/status
        let proc_status_path = format!("/proc/{}/status", self.pid);
        if let Ok(status) = fs::read_to_string(&proc_status_path) {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = val.parse::<u64>() {
                            metrics.agent_process_rss = Some(kb * 1024);
                        }
                    }
                    break;
                }
            }
        }
        
        metrics
    }
    
    fn collect_disk(&mut self) -> DiskMetrics {
        let mut metrics = DiskMetrics {
            read_bytes: None,
            write_bytes: None,
            read_iops: None,
            write_iops: None,
            utilization: None,
        };
        
        // Read disk stats from /proc/diskstats
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if let Ok(diskstats) = fs::read_to_string("/proc/diskstats") {
            let mut total_read_bytes = 0u64;
            let mut total_write_bytes = 0u64;
            let mut total_read_ios = 0u64;
            let mut total_write_ios = 0u64;
            
            for line in diskstats.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                // Format: major minor name reads reads_merged reads_sectors reads_ms writes writes_merged writes_sectors writes_ms ...
                if parts.len() >= 14 {
                    if let (Ok(reads), Ok(writes), Ok(read_sectors), Ok(write_sectors)) = (
                        parts[3].parse::<u64>(),
                        parts[7].parse::<u64>(),
                        parts[5].parse::<u64>(),
                        parts[9].parse::<u64>(),
                    ) {
                        total_read_ios += reads;
                        total_write_ios += writes;
                        total_read_bytes += read_sectors * 512; // sectors to bytes
                        total_write_bytes += write_sectors * 512;
                    }
                }
            }
            
            let current_stats = DiskStats {
                read_bytes: total_read_bytes,
                write_bytes: total_write_bytes,
                read_ios: total_read_ios,
                write_ios: total_write_ios,
            };
            
            // Calculate rates if we have previous stats
            if let Some(ref last_stats) = self.last_disk_stats {
                let time_diff = now.saturating_sub(self.last_disk_time);
                if time_diff > 0 {
                    let read_bytes_diff = current_stats.read_bytes.saturating_sub(last_stats.read_bytes);
                    let write_bytes_diff = current_stats.write_bytes.saturating_sub(last_stats.write_bytes);
                    let read_ios_diff = current_stats.read_ios.saturating_sub(last_stats.read_ios);
                    let write_ios_diff = current_stats.write_ios.saturating_sub(last_stats.write_ios);
                    
                    metrics.read_bytes = Some(read_bytes_diff);
                    metrics.write_bytes = Some(write_bytes_diff);
                    metrics.read_iops = Some(read_ios_diff);
                    metrics.write_iops = Some(write_ios_diff);
                }
            }
            
            self.last_disk_stats = Some(current_stats);
            self.last_disk_time = now;
        }
        
        metrics
    }
    
    fn collect_filesystem(&self) -> FilesystemMetrics {
        let mut mounts = Vec::new();
        
        // Read mount info from /proc/mounts
        if let Ok(mounts_content) = fs::read_to_string("/proc/mounts") {
            for line in mounts_content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let mount_point = parts[1].to_string();
                    
                    // Skip special filesystems
                    if mount_point.starts_with("/proc") || 
                       mount_point.starts_with("/sys") ||
                       mount_point.starts_with("/dev") ||
                       mount_point == "/" {
                        // Get filesystem stats using statvfs (via command or direct syscall)
                        // For simplicity, we'll use a basic approach
                        let mount_metrics = self.get_mount_stats(&mount_point);
                        mounts.push(mount_metrics);
                    }
                }
            }
        }
        
        FilesystemMetrics { mounts }
    }
    
    fn get_mount_stats(&self, mount_point: &str) -> MountMetrics {
        let mut metrics = MountMetrics {
            mount_point: mount_point.to_string(),
            total_bytes: None,
            used_bytes: None,
            free_bytes: None,
            usage_percent: None,
            inode_total: None,
            inode_used: None,
            inode_free: None,
            inode_usage_percent: None,
        };
        
        // Use df command to get filesystem stats (fail-soft if unavailable)
        if let Ok(output) = std::process::Command::new("df")
            .arg("-B1")  // Block size 1 byte
            .arg(mount_point)
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                let lines: Vec<&str> = output_str.lines().collect();
                if lines.len() >= 2 {
                    let parts: Vec<&str> = lines[1].split_whitespace().collect();
                    if parts.len() >= 4 {
                        if let (Ok(total), Ok(used), Ok(available)) = (
                            parts[1].parse::<u64>(),
                            parts[2].parse::<u64>(),
                            parts[3].parse::<u64>(),
                        ) {
                            metrics.total_bytes = Some(total);
                            metrics.used_bytes = Some(used);
                            metrics.free_bytes = Some(available);
                            if total > 0 {
                                metrics.usage_percent = Some((used as f64 / total as f64) * 100.0);
                            }
                        }
                    }
                }
            }
        }
        
        // Get inode stats using df -i
        if let Ok(output) = std::process::Command::new("df")
            .arg("-i")
            .arg(mount_point)
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                let lines: Vec<&str> = output_str.lines().collect();
                if lines.len() >= 2 {
                    let parts: Vec<&str> = lines[1].split_whitespace().collect();
                    if parts.len() >= 4 {
                        if let (Ok(total), Ok(used), Ok(available)) = (
                            parts[1].parse::<u64>(),
                            parts[2].parse::<u64>(),
                            parts[3].parse::<u64>(),
                        ) {
                            metrics.inode_total = Some(total);
                            metrics.inode_used = Some(used);
                            metrics.inode_free = Some(available);
                            if total > 0 {
                                metrics.inode_usage_percent = Some((used as f64 / total as f64) * 100.0);
                            }
                        }
                    }
                }
            }
        }
        
        metrics
    }
    
    fn collect_network(&mut self) -> NetworkMetrics {
        let mut metrics = NetworkMetrics {
            bytes_in: None,
            bytes_out: None,
            packets_in: None,
            packets_out: None,
            errors: None,
            drops: None,
        };
        
        // Read network stats from /proc/net/dev
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if let Ok(netdev) = fs::read_to_string("/proc/net/dev") {
            let mut total_bytes_in = 0u64;
            let mut total_bytes_out = 0u64;
            let mut total_packets_in = 0u64;
            let mut total_packets_out = 0u64;
            let mut total_drops = 0u64;
            let mut total_errors = 0u64;
            
            for line in netdev.lines().skip(2) {
                // Format: interface: bytes_in packets_in errs_in drops_in bytes_out packets_out errs_out drops_out
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    let stats: Vec<&str> = parts[1].trim().split_whitespace().collect();
                    if stats.len() >= 16 {
                        if let (Ok(bytes_in), Ok(packets_in), Ok(errs_in), Ok(drops_in),
                                Ok(bytes_out), Ok(packets_out), Ok(errs_out), Ok(drops_out)) = (
                            stats[0].parse::<u64>(),
                            stats[1].parse::<u64>(),
                            stats[2].parse::<u64>(),
                            stats[3].parse::<u64>(),
                            stats[8].parse::<u64>(),
                            stats[9].parse::<u64>(),
                            stats[10].parse::<u64>(),
                            stats[11].parse::<u64>(),
                        ) {
                            total_bytes_in += bytes_in;
                            total_bytes_out += bytes_out;
                            total_packets_in += packets_in;
                            total_packets_out += packets_out;
                            total_drops += drops_in + drops_out;
                            total_errors += errs_in + errs_out;
                        }
                    }
                }
            }
            
            let current_stats = NetworkStats {
                bytes_in: total_bytes_in,
                bytes_out: total_bytes_out,
                packets_in: total_packets_in,
                packets_out: total_packets_out,
                drops: total_drops,
                errors: total_errors,
            };
            
            metrics.drops = Some(total_drops);
            metrics.errors = Some(total_errors);
            
            // Calculate rates if we have previous stats
            if let Some(ref last_stats) = self.last_network_stats {
                let time_diff = now.saturating_sub(self.last_network_time);
                if time_diff > 0 {
                    let bytes_in_diff = current_stats.bytes_in.saturating_sub(last_stats.bytes_in);
                    let bytes_out_diff = current_stats.bytes_out.saturating_sub(last_stats.bytes_out);
                    let packets_in_diff = current_stats.packets_in.saturating_sub(last_stats.packets_in);
                    let packets_out_diff = current_stats.packets_out.saturating_sub(last_stats.packets_out);
                    
                    metrics.bytes_in = Some(bytes_in_diff);
                    metrics.bytes_out = Some(bytes_out_diff);
                    metrics.packets_in = Some(packets_in_diff);
                    metrics.packets_out = Some(packets_out_diff);
                }
            }
            
            self.last_network_stats = Some(current_stats);
            self.last_network_time = now;
        }
        
        metrics
    }
    
    fn collect_system_state(&self) -> SystemStateMetrics {
        let mut metrics = SystemStateMetrics {
            host_uptime: None,
            process_count: None,
            agent_process_status: None,
        };
        
        // Read host uptime from /proc/uptime
        if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
            if let Some(uptime_secs) = uptime_str.split_whitespace().next() {
                if let Ok(uptime) = uptime_secs.parse::<f64>() {
                    metrics.host_uptime = Some(uptime as u64);
                }
            }
        }
        
        // Count processes from /proc
        if let Ok(entries) = fs::read_dir("/proc") {
            let count = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.path().file_name()
                        .and_then(|n| n.to_str())
                        .and_then(|s| s.parse::<u32>().ok())
                        .is_some()
                })
                .count() as u32;
            metrics.process_count = Some(count);
        }
        
        // Check agent process status
        let proc_stat_path = format!("/proc/{}/stat", self.pid);
        if fs::metadata(&proc_stat_path).is_ok() {
            metrics.agent_process_status = Some("running".to_string());
        } else {
            metrics.agent_process_status = Some("stopped".to_string());
        }
        
        metrics
    }
}

