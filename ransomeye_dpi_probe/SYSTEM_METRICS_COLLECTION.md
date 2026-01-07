# Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/SYSTEM_METRICS_COLLECTION.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Documentation for DPI Probe system metrics collection implementation

# DPI Probe System Metrics Collection

## Overview

This document describes how to implement system metrics collection in the DPI Probe to match Core Engine system health monitoring. The metrics are collected on the DPI Probe host and included in the telemetry payload sent to the ingestion server.

## Metrics to Collect

### CPU Metrics
- CPU utilization (%)
- Load average (1m / 5m / 15m)
- Core count
- Context switches
- DPI Probe process CPU usage (%)

### Memory Metrics
- Total memory
- Used memory
- Free memory
- Swap usage
- DPI Probe process RSS (Resident Set Size)

### Disk I/O Metrics
- Read IOPS
- Write IOPS
- Read throughput (bytes/sec)
- Write throughput (bytes/sec)
- Disk utilization (% busy)

### Filesystem Metrics
- Root filesystem usage (%)
- Critical mountpoints usage (%)
- Inode usage

### Network I/O Metrics (CRITICAL)
- Bytes in / out (per interface)
- Packets in / out
- Drops
- Errors
- Ring buffer drops (if available)

### Process State Metrics
- DPI Probe process CPU %
- DPI Probe process memory %
- DPI Probe process thread count
- DPI Probe process uptime
- Packet processing rate

### System State Metrics
- Host uptime
- Process count
- DPI Probe process status (running/degraded)

## Implementation Approach

### 1. Create System Metrics Collector Module

Create a new module `edge/dpi/probe/src/system_metrics.rs`:

```rust
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub filesystem: FilesystemMetrics,
    pub network: NetworkMetrics,
    pub processing: ProcessingMetrics,
    pub system_state: SystemStateMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub utilization: Option<f64>,
    pub load_avg_1m: Option<f64>,
    pub load_avg_5m: Option<f64>,
    pub load_avg_15m: Option<f64>,
    pub core_count: Option<u32>,
    pub context_switches: Option<u64>,
    pub dpi_process_cpu: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total: Option<u64>,
    pub used: Option<u64>,
    pub free: Option<u64>,
    pub swap_used: Option<u64>,
    pub dpi_process_rss: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub read_iops: Option<u64>,
    pub write_iops: Option<u64>,
    pub read_throughput: Option<u64>,
    pub write_throughput: Option<u64>,
    pub utilization: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemMetrics {
    pub root_usage: Option<f64>,
    pub critical_mounts: Option<serde_json::Value>,
    pub inode_usage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub bytes_in: Option<u64>,
    pub bytes_out: Option<u64>,
    pub packets_in: Option<u64>,
    pub packets_out: Option<u64>,
    pub drops: Option<u64>,
    pub errors: Option<u64>,
    pub ring_buffer_drops: Option<u64>,
    pub packets_per_sec: Option<f64>,
    pub bytes_per_sec: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetrics {
    pub packet_processing_rate: Option<f64>,
    pub packet_drops: Option<u64>,
    pub probe_uptime: Option<u64>,
    pub probe_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStateMetrics {
    pub host_uptime: Option<u64>,
    pub process_count: Option<u32>,
    pub dpi_process_status: Option<String>,
}

pub struct SystemMetricsCollector {
    pid: u32,
    start_time: u64,
}

impl SystemMetricsCollector {
    pub fn new() -> Self {
        let pid = std::process::id();
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self { pid, start_time }
    }
    
    pub fn collect(&self) -> SystemMetrics {
        SystemMetrics {
            cpu: self.collect_cpu(),
            memory: self.collect_memory(),
            disk: self.collect_disk(),
            filesystem: self.collect_filesystem(),
            network: self.collect_network(),
            processing: self.collect_processing(),
            system_state: self.collect_system_state(),
        }
    }
    
    fn collect_cpu(&self) -> CpuMetrics {
        // Read from /proc/stat and /proc/loadavg
        // Parse CPU utilization, load averages
        // Get core count from /proc/cpuinfo
        // Get DPI process CPU from /proc/{pid}/stat
        // Fail-soft if any metric unavailable
        CpuMetrics {
            utilization: None, // TODO: Implement
            load_avg_1m: None, // TODO: Implement
            load_avg_5m: None, // TODO: Implement
            load_avg_15m: None, // TODO: Implement
            core_count: None, // TODO: Implement
            context_switches: None, // TODO: Implement
            dpi_process_cpu: None, // TODO: Implement
        }
    }
    
    fn collect_memory(&self) -> MemoryMetrics {
        // Read from /proc/meminfo
        // Get DPI process RSS from /proc/{pid}/status
        // Fail-soft if any metric unavailable
        MemoryMetrics {
            total: None, // TODO: Implement
            used: None, // TODO: Implement
            free: None, // TODO: Implement
            swap_used: None, // TODO: Implement
            dpi_process_rss: None, // TODO: Implement
        }
    }
    
    fn collect_disk(&self) -> DiskMetrics {
        // Read from /proc/diskstats
        // Calculate IOPS and throughput
        // Fail-soft if any metric unavailable
        DiskMetrics {
            read_iops: None, // TODO: Implement
            write_iops: None, // TODO: Implement
            read_throughput: None, // TODO: Implement
            write_throughput: None, // TODO: Implement
            utilization: None, // TODO: Implement
        }
    }
    
    fn collect_filesystem(&self) -> FilesystemMetrics {
        // Read from /proc/mounts and df command
        // Calculate usage percentages
        // Fail-soft if any metric unavailable
        FilesystemMetrics {
            root_usage: None, // TODO: Implement
            critical_mounts: None, // TODO: Implement
            inode_usage: None, // TODO: Implement
        }
    }
    
    fn collect_network(&self) -> NetworkMetrics {
        // Read from /proc/net/dev
        // Aggregate across all interfaces (no hardcoded names)
        // Calculate per-second rates
        // Fail-soft if any metric unavailable
        NetworkMetrics {
            bytes_in: None, // TODO: Implement
            bytes_out: None, // TODO: Implement
            packets_in: None, // TODO: Implement
            packets_out: None, // TODO: Implement
            drops: None, // TODO: Implement
            errors: None, // TODO: Implement
            ring_buffer_drops: None, // TODO: Implement
            packets_per_sec: None, // TODO: Implement
            bytes_per_sec: None, // TODO: Implement
        }
    }
    
    fn collect_processing(&self) -> ProcessingMetrics {
        // Get from HealthMonitor stats
        // Calculate packet processing rate
        // Get probe uptime
        // Fail-soft if any metric unavailable
        ProcessingMetrics {
            packet_processing_rate: None, // TODO: Implement
            packet_drops: None, // TODO: Implement
            probe_uptime: None, // TODO: Implement
            probe_status: None, // TODO: Implement
        }
    }
    
    fn collect_system_state(&self) -> SystemStateMetrics {
        // Read host uptime from /proc/uptime
        // Count processes from /proc
        // Get DPI process status
        // Fail-soft if any metric unavailable
        SystemStateMetrics {
            host_uptime: None, // TODO: Implement
            process_count: None, // TODO: Implement
            dpi_process_status: None, // TODO: Implement
        }
    }
}
```

### 2. Integrate into Main Loop

In `edge/dpi/probe/src/main.rs`, add periodic system metrics collection:

```rust
use system_metrics::SystemMetricsCollector;

// In main(), after initializing components:
let metrics_collector = Arc::new(SystemMetricsCollector::new());
let mut last_metrics_time = SystemTime::now();

// In main loop, collect metrics every 5-10 seconds:
if last_metrics_time.elapsed().unwrap().as_secs() >= 8 {
    let metrics = metrics_collector.collect();
    
    // Include in telemetry payload
    // This should be added to the envelope payload when sending telemetry
    last_metrics_time = SystemTime::now();
}
```

### 3. Include in Telemetry Payload

Modify the envelope builder or telemetry sending logic to include system metrics in the `payload` JSONB field:

```rust
// When building telemetry for ingestion
let system_metrics = metrics_collector.collect();
let payload = serde_json::json!({
    "system": {
        "cpu": system_metrics.cpu,
        "memory": system_metrics.memory,
        "disk": system_metrics.disk,
        "filesystem": system_metrics.filesystem,
        "network": system_metrics.network,
        "processing": system_metrics.processing,
        "system_state": system_metrics.system_state,
    }
});

// Include in the signed_event payload that gets stored in dpi_probe_telemetry.payload
```

## Collection Rules

1. **Sampling Interval**: 5-10 seconds (suitable for high-throughput systems)
2. **Fail-Soft**: All metrics must fail gracefully if unavailable
3. **No Hardcoded Values**: No hardcoded interface names, paths, or thresholds
4. **Environment-Based**: Use environment variables for configuration
5. **Performance**: Metrics collection must not impact packet processing performance

## Data Flow

1. DPI Probe collects system metrics periodically (every 5-10 seconds)
2. Metrics are included in telemetry payload when sending to ingestion server
3. Ingestion server stores payload in `dpi_probe_telemetry.payload` JSONB field
4. Dashboard API endpoint (`/api/dashboards/dpi-probe-health`) reads latest telemetry with system metrics
5. Dashboard displays metrics in panels

## Testing

1. Verify metrics collection doesn't impact packet processing
2. Verify fail-soft behavior when metrics unavailable
3. Verify payload structure matches expected schema
4. Verify dashboard displays metrics correctly

## Notes

- This is a summary document for implementation guidance
- Actual implementation should be done in Rust following the DPI Probe codebase patterns
- All metrics must be collected on the DPI Probe host itself
- No synthetic data or placeholders allowed
- Must maintain backward compatibility with existing telemetry consumers

