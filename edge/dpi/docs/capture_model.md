# Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/docs/capture_model.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Network capture model documentation

# Network Capture Model

## Overview

DPI Probe uses AF_PACKET (Linux) and libpcap (cross-platform) for high-throughput packet capture.

## Capture Interface

- **Interface Selection**: Via `CAPTURE_IFACE` environment variable
- **Promiscuous Mode**: Enabled for full packet capture
- **Buffer Size**: 64MB buffer for high throughput (≥10 Gbps)
- **Snaplen**: 65535 bytes (full packet capture)
- **Timeout**: 1000ms (non-blocking)

## Performance Characteristics

- **Zero Allocations**: Hot path uses zero-copy packet access
- **Lock-Free Statistics**: Atomic counters for performance
- **Non-Blocking**: Timeout-based packet reading

## Capture Lifecycle

1. **Initialization**: Device discovery and configuration
2. **Start**: Explicit start (optional and explicit)
3. **Capture Loop**: Non-blocking packet reading
4. **Stop**: Graceful shutdown

## Statistics

- Packets captured (atomic counter)
- Packets dropped (atomic counter)
- Bytes captured (atomic counter)
- Running status (atomic boolean)

## Error Handling

- Device not found → Error
- Capture activation failure → Error
- Timeout → Normal (not an error)
- Packet read error → Drop + log

