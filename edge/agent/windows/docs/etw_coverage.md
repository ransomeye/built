# ETW Coverage Documentation

**Path and File Name:** /home/ransomeye/rebuild/ransomeye_windows_agent/docs/etw_coverage.md  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** ETW provider coverage and event types monitored by Windows Agent

## Overview

The Windows Agent uses Event Tracing for Windows (ETW) as the primary telemetry source. ETW provides high-performance, kernel-level event collection with minimal overhead.

## ETW Providers Monitored

### 1. Microsoft-Windows-Kernel-Process
- **Purpose:** Process lifecycle events
- **Events Captured:**
  - Process Start (Create)
  - Process End (Terminate)
  - Command Line (via WMI fallback if needed)

### 2. Microsoft-Windows-Kernel-File
- **Purpose:** Filesystem activity
- **Events Captured:**
  - File Create
  - File Delete
  - File Rename
  - Permission Changes
  - Mass Write Detection (via write count tracking)

### 3. Microsoft-Windows-Kernel-Registry
- **Purpose:** Registry modifications
- **Events Captured:**
  - Key Create
  - Key Delete
  - Value Set
  - Autorun Detection
  - Persistence Key Detection

### 4. Microsoft-Windows-TCPIP
- **Purpose:** Network activity
- **Events Captured:**
  - Socket Connect
  - Socket Disconnect
  - Lightweight connection tracking

## WMI Fallback

When ETW is unavailable or insufficient, the agent falls back to WMI queries:
- Process information
- Network connections
- System configuration

## Performance Considerations

- ETW sessions are configured for minimal overhead
- Event filtering at kernel level reduces user-space processing
- Bounded memory for event tracking
- Rate limiting prevents event flooding

## Security

- ETW sessions run with minimal required privileges
- No kernel driver required
- Fail-closed on ETW initialization failure

