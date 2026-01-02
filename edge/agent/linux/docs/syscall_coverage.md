# Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/docs/syscall_coverage.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Syscall coverage documentation

# Syscall Coverage

## Overview

Linux Agent monitors syscalls via eBPF (preferred) or auditd (fallback).

## Process Syscalls

### exec
- **Syscall**: `execve`, `execveat`
- **Event**: Process execution
- **Data**: PID, PPID, executable path, command line

### fork
- **Syscall**: `fork`, `clone`, `vfork`
- **Event**: Process forking
- **Data**: Parent PID, child PID, UID, GID

### mmap
- **Syscall**: `mmap`, `mmap2`
- **Event**: Memory mapping
- **Data**: PID, memory address, size

## Filesystem Syscalls

### rename
- **Syscall**: `rename`, `renameat`, `renameat2`
- **Event**: File rename
- **Data**: Old path, new path, PID, UID, GID

### unlink
- **Syscall**: `unlink`, `unlinkat`
- **Event**: File deletion
- **Data**: Path, PID, UID, GID

### chmod
- **Syscall**: `chmod`, `fchmod`, `fchmodat`
- **Event**: Permission change
- **Data**: Path, mode, PID, UID, GID

### mass writes
- **Syscall**: `write`, `writev`, `pwrite`, `pwritev`
- **Event**: Mass write detection (threshold-based)
- **Data**: Path, write count, PID, UID, GID

## Network Syscalls

### socket operations
- **Syscalls**: `socket`, `connect`, `bind`, `listen`, `accept`, `send`, `recv`
- **Event**: Socket operations (light monitoring)
- **Data**: Socket family, type, addresses, ports, bytes transferred

## Monitoring Methods

### eBPF (Preferred)
- High performance
- Low overhead
- Kernel-space filtering
- Optional (can be disabled)

### auditd (Fallback)
- Standard Linux audit framework
- User-space processing
- Available when eBPF not available
- Fallback when eBPF fails

## Abstraction Layer

The syscall monitor abstracts eBPF and auditd:
- Same interface for both methods
- Automatic fallback
- Transparent to event processing

